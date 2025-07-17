use clap::Parser;
use gnostr::remote::host;
//use gnostr::remote::message_stream;
//use gnostr::remote::messages;
//use gnostr::remote::options;
use gnostr::remote::remote_runner;
//use gnostr::remote::tests;

use anyhow::Result;
use gnostr::remote::options::Opt;
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;

fn main() -> Result<()> {
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
        info!("Starting remote-runner");
        remote_runner::update(&opt);
    } else {
        info!("Starting host");
        host::host_loop(&opt, opt.target.as_ref().map_or("127.0.0.1", |v| v))?;
        //host::host_loop(&opt, opt.target.as_ref().unwrap())?;
    }

    Ok(())
}
