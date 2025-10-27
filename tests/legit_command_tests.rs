#![cfg(test)]

use std::error::Error as StdError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use gnostr::cli::LegitCommands;
use gnostr::sub_commands::legit::{legit, LegitSubCommand};

static LOGIN_CALLED: AtomicBool = AtomicBool::new(false);
static INIT_CALLED: AtomicBool = AtomicBool::new(false);
static SEND_CALLED: AtomicBool = AtomicBool::new(false);
static LIST_CALLED: AtomicBool = AtomicBool::new(false);
static PULL_CALLED: AtomicBool = AtomicBool::new(false);
static PUSH_CALLED: AtomicBool = AtomicBool::new(false);
static FETCH_CALLED: AtomicBool = AtomicBool::new(false);
static MINE_CALLED: AtomicBool = AtomicBool::new(false);

// Mock implementations of the launch functions
mod login {
    use super::*;
    use gnostr::sub_commands::login::LoginSubCommand;
    pub async fn launch(_args: &LoginSubCommand) -> Result<(), Box<dyn StdError>> {
        LOGIN_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

mod init {
    use super::*;
    use gnostr::sub_commands::init::InitSubCommand;
    pub async fn launch(_args: &InitSubCommand) -> Result<(), Box<dyn StdError>> {
        INIT_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

mod send {
    use super::*;
    use gnostr::sub_commands::send::SendSubCommand;
    pub async fn launch(_args: &SendSubCommand, _is_main_command: bool) -> Result<(), Box<dyn StdError>> {
        SEND_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

mod push {
    use super::*;
    use gnostr::sub_commands::push::PushSubCommand;
    pub async fn launch(_args: &PushSubCommand) -> Result<(), Box<dyn StdError>> {
        PUSH_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

mod fetch {
    use super::*;
    use gnostr::sub_commands::fetch::FetchSubCommand;
    pub async fn launch(_args: &FetchOnHandCommand) -> Result<(), Box<dyn StdError>> {
        FETCH_CALLED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

// Mock for gnostr_legit::command
mod mock_gnostr_legit {
    pub mod command {
        use super::super::*;
        use mock_gnostr_legit::gitminer::Options;
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
fn reset_mocks() {
    LOGIN_CALLED.store(false, Ordering::SeqCst);
    INIT_CALLED.store(false, Ordering::SeqCst);
    SEND_CALLED.store(false, Ordering::SeqCst);
    LIST_CALLED.store(false, Ordering::SeqCst);
    PULL_CALLED.store(false, Ordering::SeqCst);
    PUSH_CALLED.store(false, Ordering::SeqCst);
    FETCH_CALLED.store(false, Ordering::SeqCst);
    MINE_CALLED.store(false, Ordering::SeqCst);
}

#[tokio::test]
async fn test_legit_login_command() -> Result<(), Box<dyn StdError>> {
    reset_mocks();
    let args = LegitSubCommand {
        command: Some(LegitCommands::Login(gnostr::sub_commands::login::LoginSubCommand {
            nsec: None,
            password: None,
            repo: None,
            pow: None,
        })),
        repository_path: None,
        nsec: None,
        password: None,
        repo: None,
        pow: None,
        prefix: None,
        threads: 1,
        message: "test".to_string(),
        disable_cli_spinners: false,
    };
    legit(&args).await?;
    assert!(LOGIN_CALLED.load(Ordering::SeqCst));
    Ok(())
}

#[tokio::test]
async fn test_legit_init_command() -> Result<(), Box<dyn StdError>> {
    reset_mocks();
    let args = LegitSubCommand {
        command: Some(LegitCommands::Init(gnostr::sub_commands::init::InitSubCommand {
            nsec: None,
            password: None,
            repo: None,
            pow: None,
        })),
        repository_path: None,
        nsec: None,
        password: None,
        repo: None,
        pow: None,
        prefix: None,
        threads: 1,
        message: "test".to_string(),
        disable_cli_spinners: false,
    };
    legit(&args).await?;
    assert!(INIT_CALLED.load(Ordering::SeqCst));
    Ok(())
}

#[tokio::test]
async fn test_legit_send_command() -> Result<(), Box<dyn StdError>> {
    reset_mocks();
    let args = LegitSubCommand {
        command: Some(LegitCommands::Send(gnostr::sub_commands::send::SendSubCommand {
            nsec: None,
            password: None,
            repo: None,
            pow: None,
            message: None,
            disable_cli_spinners: false,
        })),
        repository_path: None,
        nsec: None,
        password: None,
        repo: None,
        pow: None,
        prefix: None,
        threads: 1,
        message: "test".to_string(),
        disable_cli_spinners: false,
    };
    legit(&args).await?;
    assert!(SEND_CALLED.load(Ordering::SeqCst));
    Ok(())
}

#[tokio::test]
async fn test_legit_list_command() -> Result<(), Box<dyn StdError>> {
    reset_mocks();
    let args = LegitSubCommand {
        command: Some(LegitCommands::List),
        repository_path: None,
        nsec: None,
        password: None,
        repo: None,
        pow: None,
        prefix: None,
        threads: 1,
        message: "test".to_string(),
        disable_cli_spinners: false,
    };
    legit(&args).await?;
    assert!(LIST_CALLED.load(Ordering::SeqCst));
    Ok(())
}

#[tokio::test]
async fn test_legit_pull_command() -> Result<(), Box<dyn StdError>> {
    reset_mocks();
    let args = LegitSubCommand {
        command: Some(LegitCommands::Pull),
        repository_path: None,
        nsec: None,
        password: None,
        repo: None,
        pow: None,
        prefix: None,
        threads: 1,
        message: "test".to_string(),
        disable_cli_spinners: false,
    };
    legit(&args).await?;
    assert!(PULL_CALLED.load(Ordering::SeqCst));
    Ok(())
}

#[tokio::test]
async fn test_legit_push_command() -> Result<(), Box<dyn StdError>> {
    reset_mocks();
    let args = LegitSubCommand {
        command: Some(LegitCommands::Push(gnostr::sub_commands::push::PushSubCommand {
            nsec: None,
            password: None,
            repo: None,
            pow: None,
            message: None,
            disable_cli_spinners: false,
        })),
        repository_path: None,
        nsec: None,
        password: None,
        repo: None,
        pow: None,
        prefix: None,
        threads: 1,
        message: "test".to_string(),
        disable_cli_spinners: false,
    };
    legit(&args).await?;
    assert!(PUSH_CALLED.load(Ordering::SeqCst));
    Ok(())
}

#[tokio::test]
async fn test_legit_fetch_command() -> Result<(), Box<dyn StdError>> {
    reset_mocks();
    let args = LegitSubCommand {
        command: Some(LegitCommands::Fetch(gnostr::sub_commands::fetch::FetchSubCommand {
            nsec: None,
            password: None,
            repo: None,
            pow: None,
        })),
        repository_path: None,
        nsec: None,
        password: None,
        repo: None,
        pow: None,
        prefix: None,
        threads: 1,
        message: "test".to_string(),
        disable_cli_spinners: false,
    };
    legit(&args).await?;
    assert!(FETCH_CALLED.load(Ordering::SeqCst));
    Ok(())
}

#[tokio::test]
async fn test_legit_mine_command() -> Result<(), Box<dyn StdError>> {
    reset_mocks();
    let args = LegitSubCommand {
        command: Some(LegitCommands::Mine),
        repository_path: None,
        nsec: None,
        password: None,
        repo: None,
        pow: None,
        prefix: None,
        threads: 1,
        message: "test".to_string(),
        disable_cli_spinners: false,
    };
    legit(&args).await?;
    assert!(MINE_CALLED.load(Ordering::SeqCst));
    Ok(())
}

#[tokio::test]
async fn test_legit_default_command() -> Result<(), Box<dyn StdError>> {
    reset_mocks();
    let args = LegitSubCommand {
        command: None, // Default case
        repository_path: None,
        nsec: None,
        password: None,
        repo: None,
        pow: None,
        prefix: None,
        threads: 1,
        message: "test".to_string(),
        disable_cli_spinners: false,
    };
    legit(&args).await?;
    assert!(MINE_CALLED.load(Ordering::SeqCst));
    Ok(())
}

#[tokio::test]
async fn test_legit_mine_command_with_custom_message_and_repo() -> Result<(), Box<dyn StdError>> {
    reset_mocks();
    let args = LegitSubCommand {
        command: Some(LegitCommands::Mine),
        repository_path: Some("/tmp/my_repo".to_string()),
        nsec: None,
        password: None,
        repo: None,
        pow: None,
        prefix: Some("abc".to_string()),
        threads: 4,
        message: "Custom commit message".to_string(),
        disable_cli_spinners: false,
    };
    legit(&args).await?;
    assert!(MINE_CALLED.load(Ordering::SeqCst));
    // In a real scenario, you would also assert that the `gnostr_legit::command::run_legit_command`
    // was called with the correct `Options` struct. This would require a more sophisticated mock
    // that captures arguments. For now, just checking if it was called is sufficient.
    Ok(())
}