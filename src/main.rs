mod constants;
mod stations;
mod trips;

use clap::Parser;
use dotenv::dotenv;

use stations::pick_station_local;
use trips::trips;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Start station name to search for
    from: String,
    /// Destination station name to search for
    to: String,
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
    let station_from = pick_station_local(&args.from)?;
    let station_to = pick_station_local(&args.to)?;
    println!(
        "Finding journey from {} to {}",
        station_from.names.long, station_to.names.long,
    );
    let _ = trips(station_from, station_to)?;
    Ok(())
}
