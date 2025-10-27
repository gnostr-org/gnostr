#![cfg(test)]

use std::error::Error as StdError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

// Mock flags to check if functions were called
pub static LOGIN_CALLED: AtomicBool = AtomicBool::new(false);
pub static INIT_CALLED: AtomicBool = AtomicBool::new(false);
pub static SEND_CALLED: AtomicBool = AtomicBool::new(false);
pub static LIST_CALLED: AtomicBool = AtomicBool::new(false);
pub static PULL_CALLED: AtomicBool = AtomicBool::new(false);
pub static PUSH_CALLED: AtomicBool = AtomicBool::new(false);
pub static FETCH_CALLED: AtomicBool = AtomicBool::new(false);
pub static MINE_CALLED: AtomicBool = AtomicBool::new(false);

// Mock implementations of the launch functions
pub mod login {
    use super::*;
    use crate::sub_commands::login::LoginSubCommand; // Use the actual struct
    pub async fn launch(_args: &LoginSubCommand) -> Result<(), Box<dyn StdError>> {
        LOGIN_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

pub mod init {
    use super::*;
    use crate::sub_commands::init::InitSubCommand; // Use the actual struct
    pub async fn launch(_args: &InitSubCommand) -> Result<(), Box<dyn StdError>> {
        INIT_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

pub mod send {
    use super::*;
    use crate::sub_commands::send::SendSubCommand; // Use the actual struct
    pub async fn launch(_args: &SendSubCommand, _is_main_command: bool) -> Result<(), Box<dyn StdError>> {
        SEND_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

pub mod list {
    use super::*;
    pub async fn launch() -> Result<(), Box<dyn StdError>> {
        LIST_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

pub mod pull {
    use super::*;
    pub async fn launch() -> Result<(), Box<dyn StdError>> {
        PULL_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

pub mod push {
    use super::*;
    use crate::sub_commands::push::PushSubCommand; // Use the actual struct
    pub async fn launch(_args: &PushSubCommand) -> Result<(), Box<dyn StdError>> {
        PUSH_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

pub mod fetch {
    use super::*;
    use crate::sub_commands::fetch::FetchSubCommand; // Use the actual struct
    pub async fn launch(_args: &FetchSubCommand) -> Result<(), Box<dyn StdError>> {
        FETCH_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

// Mock for gnostr_legit::command
pub mod mock_gnostr_legit {
    pub mod command {
        use super::super::*;
        use mock_gnostr_legit::gitminer::Options; // Use the actual struct
        pub fn run_legit_command(_opts: Options) -> Result<(), Box<dyn StdError>> {
            MINE_CALLED.store(true, Ordering::SeqCst);
            Ok(())
        }
    }
    // Re-export gitminer::Options if needed by the main code or other mocks
    pub mod gitminer {
        pub use mock_gnostr_legit::gitminer::Options;
    }
}

// Reset all flags before each test
pub fn reset_mocks() {
    LOGIN_CALLED.store(false, Ordering::SeqCst);
    INIT_CALLED.store(false, Ordering::SeqCst);
    SEND_CALLED.store(false, Ordering::SeqCst);
    LIST_CALLED.store(false, Ordering::SeqCst);
    PULL_CALLED.store(false, Ordering::SeqCst);
    PUSH_CALLED.store(false, Ordering::SeqCst);
    FETCH_CALLED.store(false, Ordering::SeqCst);
    MINE_CALLED.store(false, Ordering::SeqCst);
}
