#![allow(unused)]
#![allow(dead_code)]

use std::io::{Result};

mod command;

fn main() -> io::Result<()> {
    command::run_legit_command()
}