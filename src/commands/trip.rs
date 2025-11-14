use crate::stations::pick_station_local;
use crate::trips::trips;

pub fn execute(from: &str, to: &str) -> Result<(), Box<dyn std::error::Error>> {
    let station_from = pick_station_local(from)?;
    let station_to = pick_station_local(to)?;
    println!(
        "Finding journey from {} to {}",
        station_from.names.long, station_to.names.long,
    );
    trips(station_from, station_to)?;
    Ok(())
}
