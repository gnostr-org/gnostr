use std::convert::TryFrom;

/// A way that a user uses a Relay
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum RelayUsage {
    /// User seeks events here if they are not otherwise found
    FallbackRead = 1 << 0,

    /// User writes here but does not advertise it
    Archive = 1 << 1,

    // was a relay usage flag in gossip, but was retired
    // Advertise = 1 << 2,
    /// User accepts posts here from the public that tag them
    Inbox = 1 << 3,

    /// User posts here for the public
    Outbox = 1 << 4,

    /// User seeks relay lists here (index, discover)
    Directory = 1 << 5,

    // is used as SPAMSAFE bit in gossip so reserved, but isn't a relay usage
    // ReservedSpamsafe = 1 << 6,
    /// User accepts DMs here
    Dm = 1 << 7,

    /// user stores and reads back their own configurations here
    Config = 1 << 8,

    /// User does NIP-50 SEARCH here
    Search = 1 << 9,
}

impl TryFrom<u32> for RelayUsage {
    type Error = ();

    fn try_from(u: u32) -> Result<RelayUsage, ()> {
        match u {
            1 => Ok(RelayUsage::FallbackRead),
            2 => Ok(RelayUsage::Archive),
            8 => Ok(RelayUsage::Inbox),
            16 => Ok(RelayUsage::Outbox),
            32 => Ok(RelayUsage::Directory),
            128 => Ok(RelayUsage::Dm),
            256 => Ok(RelayUsage::Config),
            512 => Ok(RelayUsage::Search),
            _ => Err(()),
        }
    }
}

/// The ways that a user uses a Relay
// See also https://github.com/mikedilger/gossip/blob/master/gossip-lib/src/storage/types/relay3.rs
// See also https://github.com/nostr-protocol/nips/issues/1282 for possible future entries
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct RelayUsageSet(u32);

impl RelayUsageSet {
    const MASK: u32 = RelayUsage::FallbackRead as u32
        | RelayUsage::Archive as u32
        | RelayUsage::Inbox as u32
        | RelayUsage::Outbox as u32
        | RelayUsage::Directory as u32
        | RelayUsage::Dm as u32
        | RelayUsage::Config as u32
        | RelayUsage::Search as u32;

    /// Create a new empty RelayUsageSet
    pub const fn new_empty() -> Self {
        RelayUsageSet(0)
    }

    /// Create a new RelayUsageSet with all usages
    pub const fn new_all() -> Self {
        Self(Self::MASK)
    }

    /// Get the u32 bitflag representation
    pub const fn bits(&self) -> u32 {
        self.0
    }

    /// Set from a u32 bitflag representation. If any unknown bits are set,
    /// this will return None
    pub const fn from_bits(bits: u32) -> Option<RelayUsageSet> {
        if bits & !Self::MASK != 0 {
            None
        } else {
            Some(RelayUsageSet(bits))
        }
    }

    /// Set from a u32 bitflag representation. If any unknown bits are set,
    /// they will be cleared
    pub const fn from_bits_truncate(bits: u32) -> RelayUsageSet {
        RelayUsageSet(bits & Self::MASK)
    }

    /// Whether all bits are unset
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Whether all defined bits are set
    pub const fn is_all(&self) -> bool {
        self.0 & Self::MASK == Self::MASK
    }

    /// Whether any usage in other is also in Self
    pub const fn intersects(&self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Whether all usages in other are in Self
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Has a RelayUsage set
    pub fn has_usage(&mut self, ru: RelayUsage) -> bool {
        self.0 & ru as u32 == ru as u32
    }

    /// Add a RelayUsage to Self
    pub fn add_usage(&mut self, ru: RelayUsage) {
        self.0 |= ru as u32
    }

    /// Remove a RelayUsage to Self
    pub fn remove_usage(&mut self, ru: RelayUsage) {
        self.0 &= !(ru as u32)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::convert::TryFrom;

    const ALL_VARIANTS: [RelayUsage; 8] = [
        RelayUsage::FallbackRead,
        RelayUsage::Archive,
        RelayUsage::Inbox,
        RelayUsage::Outbox,
        RelayUsage::Directory,
        RelayUsage::Dm,
        RelayUsage::Config,
        RelayUsage::Search,
    ];

    #[test]
    fn try_from_accepts_all_valid_values() {
        // Arrange
        let expected: [(u32, RelayUsage); 8] = [
            (1, RelayUsage::FallbackRead),
            (2, RelayUsage::Archive),
            (8, RelayUsage::Inbox),
            (16, RelayUsage::Outbox),
            (32, RelayUsage::Directory),
            (128, RelayUsage::Dm),
            (256, RelayUsage::Config),
            (512, RelayUsage::Search),
        ];

        // Act / Assert
        for (bits, usage) in expected {
            assert_eq!(RelayUsage::try_from(bits), Ok(usage));
        }
    }

    #[test]
    fn try_from_rejects_unknown_bits() {
        // Arrange: 0, retired/reserved bits (4, 64), combos, and out-of-range
        let invalid = [0_u32, 4, 64, 3, 1024, u32::MAX];

        // Act / Assert
        for bits in invalid {
            assert!(
                RelayUsage::try_from(bits).is_err(),
                "bits {bits} must be rejected"
            );
        }
    }

    #[test]
    fn variants_are_distinct_single_bits() {
        // Arrange / Act / Assert
        let mut seen = 0_u32;
        for usage in ALL_VARIANTS {
            let bits = usage as u32;
            assert_eq!(bits.count_ones(), 1, "each usage must be a single bit");
            assert_eq!(seen & bits, 0, "usage bits must not overlap");
            seen |= bits;
        }
    }

    #[test]
    fn new_empty_and_new_all() {
        // Arrange
        let empty = RelayUsageSet::new_empty();
        let all = RelayUsageSet::new_all();

        // Act / Assert
        assert!(empty.is_empty());
        assert!(!empty.is_all());
        assert!(!all.is_empty());
        assert!(all.is_all());
        assert_eq!(all.bits(), empty.bits() | all.bits());
    }

    #[test]
    fn bits_roundtrip_through_from_bits() {
        // Arrange
        let bits = RelayUsage::Inbox as u32 | RelayUsage::Dm as u32;

        // Act
        let mut set = RelayUsageSet::from_bits(bits).unwrap();

        // Assert
        assert_eq!(set.bits(), bits);
        assert!(set.has_usage(RelayUsage::Inbox));
        assert!(set.has_usage(RelayUsage::Dm));
        assert!(!set.has_usage(RelayUsage::Outbox));
    }

    #[test]
    fn from_bits_rejects_unknown_bits() {
        // Arrange
        let bits_with_unknown = RelayUsage::Inbox as u32 | (1 << 2); // retired Advertise bit
        let bits_with_reserved = RelayUsage::Inbox as u32 | (1 << 6); // reserved Spamsafe bit

        // Act / Assert
        assert!(RelayUsageSet::from_bits(bits_with_unknown).is_none());
        assert!(RelayUsageSet::from_bits(bits_with_reserved).is_none());
        assert!(RelayUsageSet::from_bits(1 << 30).is_none());
    }

    #[test]
    fn from_bits_truncate_clears_unknown_bits() {
        // Arrange
        let bits = RelayUsage::Search as u32 | (1 << 6) | (1 << 30);

        // Act
        let mut set = RelayUsageSet::from_bits_truncate(bits);

        // Assert
        assert_eq!(set.bits(), RelayUsage::Search as u32);
        assert!(set.has_usage(RelayUsage::Search));
    }

    #[test]
    fn set_operations_intersects_and_contains() {
        // Arrange
        let mut inbox_outbox = RelayUsageSet::new_empty();
        inbox_outbox.add_usage(RelayUsage::Inbox);
        inbox_outbox.add_usage(RelayUsage::Outbox);
        let mut outbox_only = RelayUsageSet::new_empty();
        outbox_only.add_usage(RelayUsage::Outbox);

        // Act / Assert
        assert!(inbox_outbox.intersects(outbox_only));
        assert!(inbox_outbox.contains(outbox_only));
        assert!(!outbox_only.contains(inbox_outbox));
        assert!(!outbox_only.intersects(RelayUsageSet::new_empty()));

        // Act: remove
        inbox_outbox.remove_usage(RelayUsage::Inbox);
        assert!(!inbox_outbox.has_usage(RelayUsage::Inbox));
        assert!(inbox_outbox.has_usage(RelayUsage::Outbox));
        assert_eq!(inbox_outbox, outbox_only);
    }

    #[test]
    fn add_usage_is_idempotent() {
        // Arrange
        let mut set = RelayUsageSet::new_empty();

        // Act
        set.add_usage(RelayUsage::Dm);
        let once = set.bits();
        set.add_usage(RelayUsage::Dm);

        // Assert
        assert_eq!(set.bits(), once);
    }
}
