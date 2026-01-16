extern crate russh;
pub mod config;
pub mod git;
pub mod site;
pub mod ssh;
pub mod state;
pub mod utils;
pub mod vars;

#[cfg(test)]
#[path = "./utils_test.rs"]
mod utils_test;

#[cfg(test)]
#[path = "./git_test.rs"]
mod git_test;

#[cfg(test)]
#[path = "./config_test.rs"]
mod config_test;

#[cfg(test)]
#[path = "./state_test.rs"]
mod state_test;
