use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, num_args = 0..=1)]
    tag: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    println!("Parsed tag: {:?}", cli.tag);
}