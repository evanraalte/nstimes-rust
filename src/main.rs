use clap::Parser;
use dotenv::dotenv;
use std::env;

use serde::Deserialize;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Station name to search for
    from: String,
    to: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    payload: Vec<Station>,
}

#[derive(Debug, Deserialize)]
struct Station {
    id: StationId,
    names: StationNames,
}

#[derive(Debug, Deserialize)]
struct StationId {
    #[serde(rename = "uicCode")]
    uic_code: String,
}

#[derive(Debug, Deserialize)]
struct StationNames {
    long: String,
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

fn pick_station(query: &str) -> Result<Station, Box<dyn std::error::Error>> {
    let encoded_query = urlencoding::encode(query);
    let url = format!(
        "https://gateway.apiportal.ns.nl/nsapp-stations/v3?q={}&includeNonPlannableStations=false&limit=10",
        encoded_query
    );

    // Get the NS_API_TOKEN from the environment
    let ns_api_token = env::var("NS_API_TOKEN")?;

    let body: String = ureq::get(url)
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &ns_api_token)
        .call()?
        .body_mut()
        .read_to_string()?;

    let response: ApiResponse = serde_json::from_str(&body).unwrap();

    match response.payload.len() {
        0 => Err("❌ No stations found for your query".into()),
        1 => Ok(response.payload.into_iter().next().unwrap()),
        _ => {
            println!("Your query was ambiguous, multiple stations matched:");
            for s in &response.payload {
                println!("{} - {}", s.id.uic_code, s.names.long);
            }
            Err("⚠️ Multiple stations matched. Please refine your query.".into())
        }
    }
}
