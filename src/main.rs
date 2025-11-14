mod commands;
mod constants;
mod stations;
mod trips;

use clap::{Parser, Subcommand};
use dotenv::dotenv;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Find train trips between two stations
    Trip {
        /// Start station name to search for
        from: String,
        /// Destination station name to search for
        to: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let args = Args::parse();

    match args.command {
        Commands::Trip { from, to } => commands::trip::execute(&from, &to)?,
    }

    Ok(())
}
