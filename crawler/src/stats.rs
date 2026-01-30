/// Keep track of some stats: count encountered event, time of last event, etc.
pub struct Stats {
    /// Count of Contact events seen
    pub count_contacts: u64,
    /// Count of Relay events seen
    pub count_relays: u64,
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            count_contacts: 0,
            count_relays: 0,
        }
    }

    pub fn add_contacts(&mut self) {
        self.count_contacts += 1;
    }

    pub fn add_relays(&mut self) {
        self.count_relays += 1;
    }

    // pub fn print_summary(&self) {
    //     println!("ev_cnts {} {} \t ", self.count_contacts, self.count_relays);
    // }
}
