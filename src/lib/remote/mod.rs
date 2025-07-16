pub mod host;
pub mod message_stream;
pub mod messages;
pub mod options;
pub mod remote_runner;
pub mod tests;
use clap::Parser;

use crate::remote::options::Opt;
use anyhow::Result;
use log::LevelFilter;
use simple_logger::SimpleLogger;

pub fn remote() -> Result<()> {
    let opt = Opt::parse();

    let level = match opt.log_level.as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Warn,
    };

    SimpleLogger::new()
        .with_level(level)
        .with_colors(true)
        .init()?;

    if opt.remote_runner {
        println!("Starting remote-runner");
        remote_runner::update(&opt);
    } else {
        println!("Starting host");
        host::host_loop(&opt, opt.target.as_ref().unwrap())?;
    }

    Ok(())
}
