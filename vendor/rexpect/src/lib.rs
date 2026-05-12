//! The main crate of Rexpect
//!
//! # Overview
//!
//! Rexpect is a loose port of [pexpect](https://pexpect.readthedocs.io/en/stable/)
//! which itself is inspired by Don Libe's expect.
//!
//! It's main components (depending on your need you can use either of those)
//!
//! - [session](session/index.html): automate stuff in Rust
//! - [reader](reader/index.html): a non-blocking reader with buffering, matching on
//!   strings/regex/...
//! - [process](process/index.html): spawn a process in a pty
//!
//! # Basic example
//!
//! ```no_run
//!
//! use rexpect::spawn;
//! use rexpect::error::Error;
//!
//! fn main() -> Result<(), Error> {
//!     let mut p = spawn("ftp speedtest.tele2.net", Some(2000))?;
//!     p.exp_regex("Name \\(.*\\):")?;
//!     p.send_line("anonymous")?;
//!     p.exp_string("Password")?;
//!     p.send_line("test")?;
//!     p.exp_string("ftp>")?;
//!     p.send_line("cd upload")?;
//!     p.exp_string("successfully changed.\r\nftp>")?;
//!     p.send_line("pwd")?;
//!     p.exp_regex("[0-9]+ \"/upload\"")?;
//!     p.send_line("exit")?;
//!     p.exp_eof()?;
//!     Ok(())
//! }
//! ```
//!
//! # Example with bash
//!
//! Tip: try the chain of commands first in a bash session.
//! The tricky thing is to get the wait_for_prompt right.
//! What `wait_for_prompt` actually does is seeking to the next
//! visible prompt. If you forgot to call this once your next call to
//! `wait_for_prompt` comes out of sync and you're seeking to a prompt
//! printed "above" the last `execute()`.
//!
//! ```no_run
//! use rexpect::spawn_bash;
//! use rexpect::error::Error;
//!
//! fn main() -> Result<(), Error> {
//!     let mut p = spawn_bash(Some(30_000))?;
//!     p.execute("ping 8.8.8.8", "bytes of data")?;
//!     p.send_control('z')?;
//!     p.wait_for_prompt()?;
//!     p.execute("bg", "suspended")?;
//!     p.send_line("sleep 1")?;
//!     p.wait_for_prompt()?;
//!     p.execute("fg", "continued")?;
//!     p.send_control('c')?;
//!     p.exp_string("packet loss")?;
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod reader;

#[cfg(not(windows))]
pub mod process;

#[cfg(windows)]
pub mod process {
    use crate::error::Error;
    use std::process::Command;

    #[derive(Debug)]
    pub struct PtyProcess;

    impl PtyProcess {
        pub fn new(_command: Command) -> Result<Self, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn set_kill_timeout(&mut self, _timeout_ms: Option<u64>) {}
    }
}

#[cfg(not(windows))]
pub mod session;

#[cfg(windows)]
pub mod session {
    use crate::error::Error;
    use std::io::{Read, Write};
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};
    use std::process::Command;

    pub use crate::reader::{Options, ReadUntil};

    #[derive(Debug)]
    pub struct StreamSession<W: Write> {
        _marker: PhantomData<W>,
    }

    impl<W: Write> StreamSession<W> {
        pub fn new<R: Read + Send + 'static>(_reader: R, _writer: W, _options: Options) -> Self {
            Self {
                _marker: PhantomData,
            }
        }

        pub fn send_line(&mut self, _line: &str) -> Result<usize, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn send(&mut self, _s: &str) -> Result<usize, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn send_control(&mut self, _c: char) -> Result<(), Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn flush(&mut self) -> Result<(), Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn read_line(&mut self) -> Result<String, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn try_read(&mut self) -> Option<char> {
            None
        }

        pub fn exp_eof(&mut self) -> Result<String, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn exp_regex(&mut self, _regex: &str) -> Result<(String, String), Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn exp_string(&mut self, _needle: &str) -> Result<String, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn exp_char(&mut self, _needle: char) -> Result<String, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn exp_any(&mut self, _needles: Vec<ReadUntil>) -> Result<(String, String), Error> {
            Err(Error::UnsupportedPlatform)
        }
    }

    #[derive(Debug)]
    pub struct PtySession;

    impl PtySession {
        pub fn new(_process: (), _options: Options) -> Result<Self, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn send_line(&mut self, _line: &str) -> Result<usize, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn send(&mut self, _s: &str) -> Result<usize, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn flush(&mut self) -> Result<(), Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn read_line(&mut self) -> Result<String, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn try_read(&mut self) -> Option<char> {
            None
        }

        pub fn exp_eof(&mut self) -> Result<String, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn exp_regex(&mut self, _regex: &str) -> Result<(String, String), Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn exp_string(&mut self, _needle: &str) -> Result<String, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn exp_char(&mut self, _needle: char) -> Result<String, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn exp_any(&mut self, _needles: Vec<ReadUntil>) -> Result<(String, String), Error> {
            Err(Error::UnsupportedPlatform)
        }
    }

    #[derive(Debug)]
    pub struct PtyReplSession {
        pub prompt: String,
        pub pty_session: PtySession,
        pub quit_command: Option<String>,
        pub echo_on: bool,
    }

    impl PtyReplSession {
        pub fn wait_for_prompt(&mut self) -> Result<String, Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn execute(&mut self, _cmd: &str, _ready_regex: &str) -> Result<(), Error> {
            Err(Error::UnsupportedPlatform)
        }

        pub fn send_line(&mut self, _line: &str) -> Result<usize, Error> {
            Err(Error::UnsupportedPlatform)
        }
    }

    impl Deref for PtyReplSession {
        type Target = PtySession;

        fn deref(&self) -> &PtySession {
            &self.pty_session
        }
    }

    impl DerefMut for PtyReplSession {
        fn deref_mut(&mut self) -> &mut PtySession {
            &mut self.pty_session
        }
    }

    impl Drop for PtyReplSession {
        fn drop(&mut self) {}
    }

    pub fn spawn(_program: &str, _timeout_ms: Option<u64>) -> Result<PtySession, Error> {
        Err(Error::UnsupportedPlatform)
    }

    pub fn spawn_with_options(
        _command: Command,
        _options: Options,
    ) -> Result<PtySession, Error> {
        Err(Error::UnsupportedPlatform)
    }

    pub fn spawn_bash(_timeout: Option<u64>) -> Result<PtyReplSession, Error> {
        Err(Error::UnsupportedPlatform)
    }

    pub fn spawn_python(_timeout: Option<u64>) -> Result<PtyReplSession, Error> {
        Err(Error::UnsupportedPlatform)
    }

    pub fn spawn_stream<R: Read + Send + 'static, W: Write>(
        _reader: R,
        _writer: W,
        _timeout_ms: Option<u64>,
    ) -> StreamSession<W> {
        StreamSession {
            _marker: PhantomData,
        }
    }

}

pub use reader::ReadUntil;
pub use session::{spawn, spawn_bash, spawn_python, spawn_stream, spawn_with_options};

// include the README.md here to test its doc
#[doc = include_str!("../README.md")]
mod test {}
