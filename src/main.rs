use clap::Parser;

mod cli;
mod config;
mod utils;

use cli::args::Cli;

fn main() {
    // Parse command line arguments
    let cli = Cli::parse();

    // Handle the result of running the CLI
    if let Err(err) = cli.run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
