#[cfg(feature = "gnostr-web")]
use clap::Parser;

#[cfg(feature = "gnostr-web")]
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 3030)]
    port: u16,
}

#[cfg(feature = "gnostr-web")]
#[tokio::main]
async fn main() {
    let args = Args::parse();
    gnostr_js::web_app::run(args.port).await.expect("run web app");
}

#[cfg(not(feature = "gnostr-web"))]
fn main() {
    eprintln!("needs gnostr-web feature enabled");
    std::process::exit(1);
}
