mod constants;
mod stations;
mod trips;

use clap::{Parser, Subcommand};
use dotenv::dotenv;

use stations::pick_station_local;
use trips::trips;

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
        Commands::Trip { from, to } => {
            let station_from = pick_station_local(&from)?;
            let station_to = pick_station_local(&to)?;
            println!(
                "Finding journey from {} to {}",
                station_from.names.long, station_to.names.long,
            );
            trips(station_from, station_to)?;
        }
    }

    Ok(())
}
