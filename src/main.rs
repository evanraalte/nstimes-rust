use clap::Parser;
use dotenv::dotenv;
mod stations;
mod stations_models;
mod trips;
mod trips_models;

use stations::pick_station;
use trips::trips;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Station name to search for
    from: String,
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
    let station_from = pick_station(&args.from)?;

    let station_to = pick_station(&args.to)?;
    println!(
        "Finding journey from {} ({}) to {} ({})",
        station_from.names.long,
        station_from.id.uic_code,
        station_to.names.long,
        station_to.id.uic_code
    );
    let _ = trips(station_from, station_to);
    Ok(())
}
