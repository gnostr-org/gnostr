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
pub mod chat;
mod commands;
pub mod cube;
pub mod evt_loop;
pub mod global_rt;
pub mod handlers;
pub mod local_git;
pub mod system_command;
pub mod terminal;
pub mod ui;
pub mod utils;
pub use clap::parser::ValueSource;
pub use clap::{Arg, ArgAction, ArgMatches, Command, Parser, Subcommand};
pub use color_eyre::eyre::{Result, WrapErr};
pub use handlers::config::CompleteConfig;

//
//
//}
