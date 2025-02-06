use clap::Parser;

mod cli;
mod config;
mod utils;
mod api;
mod context;

use cli::args::Cli;

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let cli = Cli::parse();

    // Handle the result of running the CLI
    if let Err(err) = cli.run().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
