use clap::Parser;
use dotenv::dotenv;
mod response_models;
mod stations;
use stations::pick_station;

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
        "Finding journey from {} to {}",
        station_from.names.long, station_to.names.long
    );
    Ok(())
}
