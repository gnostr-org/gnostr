//! Ratatui widget wrappers for asyncgit Nostr types.
//!
//! Each widget renders the wrapped value with debug formatting so the TUI can
//! show the structure without requiring custom formatting logic for every type.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph, Widget},
};

use crate::types::*;

macro_rules! debug_widget {
    ($name:ident, $ty:ty) => {
        /// A simple debug-rendering widget for a Nostr type.
        pub struct $name<'a> {
            value: &'a $ty,
            block: Option<Block<'static>>,
            style: Style,
        }

        impl<'a> $name<'a> {
            /// Create a widget for the given value.
            pub fn new(value: &'a $ty) -> Self {
                Self {
                    value,
                    block: None,
                    style: Style::default(),
                }
            }

            /// Attach a block to the widget.
            pub fn block(mut self, block: Block<'static>) -> Self {
                self.block = Some(block);
                self
            }

            /// Override the paragraph style used for rendering.
            pub fn style(mut self, style: Style) -> Self {
                self.style = style;
                self
            }
        }

        impl Widget for $name<'_> {
            fn render(self, area: Rect, buf: &mut Buffer) {
                let mut widget = Paragraph::new(format!("{:#?}", self.value)).style(self.style);
                if let Some(block) = self.block {
                    widget = widget.block(block);
                }
                widget.render(area, buf);
            }
        }
    };
}

debug_widget!(EventWidget, Event);
debug_widget!(EventBuilderWidget, EventBuilder);
debug_widget!(EventKindWidget, EventKind);
debug_widget!(EventKindIteratorWidget, EventKindIterator);
debug_widget!(EventKindOrRangeWidget, EventKindOrRange);
debug_widget!(EventReferenceWidget, EventReference);
debug_widget!(ClientMessageWidget, ClientMessage);
debug_widget!(ContentSegmentWidget, ContentSegment);
debug_widget!(DelegationConditionsWidget, DelegationConditions);
debug_widget!(EventDelegationWidget, EventDelegation);
debug_widget!(IdWidget, Id);
debug_widget!(IdHexWidget, IdHex);
debug_widget!(IdentityWidget, Identity);
debug_widget!(KeySignerWidget, KeySigner);
debug_widget!(ChannelCreationEventWidget, ChannelCreationEvent);
debug_widget!(ChannelMetadataEventWidget, ChannelMetadataEvent);
debug_widget!(ChannelMessageEventWidget, ChannelMessageEvent);
debug_widget!(HideMessageEventWidget, HideMessageEvent);
debug_widget!(MuteUserEventWidget, MuteUserEvent);
debug_widget!(MetadataWidget, Metadata);
debug_widget!(NAddrWidget, NAddr);
debug_widget!(NEventWidget, NEvent);
debug_widget!(Nip05Widget, Nip05);
debug_widget!(Nip19Widget, Nip19);
debug_widget!(Nip19ProfileWidget, Nip19Profile);
debug_widget!(Nip19EventWidget, Nip19Event);
debug_widget!(Nip19AddressWidget, Nip19Address);
debug_widget!(NostrBech32Widget, NostrBech32);
debug_widget!(NostrUrlWidget, NostrUrl);
debug_widget!(PayRequestDataWidget, PayRequestData);
debug_widget!(ContentEncryptionAlgorithmWidget, ContentEncryptionAlgorithm);
debug_widget!(EncryptedPrivateKeyWidget, EncryptedPrivateKey);
debug_widget!(KeySecurityWidget, KeySecurity);
debug_widget!(PrivateKeyWidget, PrivateKey);
debug_widget!(ProfileWidget, Profile);
debug_widget!(PublicKeyWidget, PublicKey);
debug_widget!(PublicKeyHexWidget, PublicKeyHex);
debug_widget!(RelayFeesWidget, RelayFees);
debug_widget!(RelayInformationDocumentWidget, RelayInformationDocument);
debug_widget!(RelayLimitationWidget, RelayLimitation);
debug_widget!(RelayRetentionWidget, RelayRetention);
debug_widget!(RelayListWidget, RelayList);
debug_widget!(RelayListUsageWidget, RelayListUsage);
debug_widget!(RelayMessageWidget, RelayMessage);
debug_widget!(RelayOriginWidget, RelayOrigin);
debug_widget!(RelayUrlWidget, RelayUrl);
debug_widget!(RelayUsageWidget, RelayUsage);
debug_widget!(RelayUsageSetWidget, RelayUsageSet);
debug_widget!(MilliSatoshiWidget, MilliSatoshi);
debug_widget!(SignatureWidget, Signature);
debug_widget!(SignatureHexWidget, SignatureHex);
debug_widget!(SimpleRelayListWidget, SimpleRelayList);
debug_widget!(SimpleRelayUsageWidget, SimpleRelayUsage);
debug_widget!(SubscriptionIdWidget, SubscriptionId);
debug_widget!(TagWidget, Tag);
debug_widget!(UnixtimeWidget, Unixtime);
debug_widget!(UncheckedUrlWidget, UncheckedUrl);
debug_widget!(UrlWidget, Url);
debug_widget!(KeysWidget, Keys);
debug_widget!(ClientWidget, Client);
debug_widget!(FilterWidget, Filter);
debug_widget!(FilterOptionsWidget, FilterOptions);
debug_widget!(OptionsWidget, Options);
debug_widget!(ImageDimensionsWidget, ImageDimensions);
debug_widget!(RelayMessageV1Widget, RelayMessageV1);
debug_widget!(RelayMessageV2Widget, RelayMessageV2);
debug_widget!(RelayMessageV3Widget, RelayMessageV3);
debug_widget!(RelayMessageV4Widget, RelayMessageV4);
debug_widget!(RelayMessageV5Widget, RelayMessageV5);
debug_widget!(WhyWidget, Why);
debug_widget!(ClientMessageV1Widget, ClientMessageV1);
debug_widget!(ClientMessageV2Widget, ClientMessageV2);
debug_widget!(ClientMessageV3Widget, ClientMessageV3);
debug_widget!(EventV1Widget, EventV1);
debug_widget!(EventV2Widget, EventV2);
debug_widget!(EventV3Widget, EventV3);
debug_widget!(PreEventV1Widget, PreEventV1);
debug_widget!(PreEventV2Widget, PreEventV2);
debug_widget!(PreEventV3Widget, PreEventV3);
debug_widget!(RumorV1Widget, RumorV1);
debug_widget!(RumorV2Widget, RumorV2);
debug_widget!(RumorV3Widget, RumorV3);
debug_widget!(MetadataV1Widget, MetadataV1);
debug_widget!(Nip05V1Widget, Nip05V1);
debug_widget!(FeeV1Widget, FeeV1);
debug_widget!(RelayFeesV1Widget, RelayFeesV1);
debug_widget!(RelayInformationDocumentV1Widget, RelayInformationDocumentV1);
debug_widget!(RelayInformationDocumentV2Widget, RelayInformationDocumentV2);
debug_widget!(RelayLimitationV1Widget, RelayLimitationV1);
debug_widget!(RelayLimitationV2Widget, RelayLimitationV2);
debug_widget!(RelayRetentionV1Widget, RelayRetentionV1);
debug_widget!(TagV1Widget, TagV1);
debug_widget!(TagV2Widget, TagV2);
debug_widget!(TagV3Widget, TagV3);
debug_widget!(ZapDataV1Widget, ZapDataV1);
debug_widget!(ZapDataV2Widget, ZapDataV2);
debug_widget!(EventRefTypeWidget, EventRefType);
debug_widget!(RepoRefWidget, RepoRef);
debug_widget!(RepoStateWidget, RepoState);
debug_widget!(Nip34EventWidget, Nip34Event);
debug_widget!(Nip34KindWidget, Nip34Kind);
debug_widget!(Nip34UnsignedEventWidget, Nip34UnsignedEvent);
