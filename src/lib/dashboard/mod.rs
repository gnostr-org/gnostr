//pub mod prelude {
//pub use std::result::Result;
pub use std::convert::{TryFrom, TryInto};
pub use std::fmt::{self, Debug, Display};
pub use std::iter::{IntoIterator, Iterator};
pub use std::option::Option;

pub use once_cell::sync::Lazy;
pub use once_cell::sync::OnceCell;

// Add more items as needed.
pub mod app;
pub mod p2p;
pub mod handlers;
pub mod ui;
pub use clap::parser::ValueSource;
pub use clap::{Arg, ArgAction, ArgMatches, Command, Parser, Subcommand};
pub use anyhow::{Result};
pub use handlers::config::CompleteConfig;

//
//
//}
