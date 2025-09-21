use crate::constants::STATIONS;
use crate::stations_models::{ApiResponse, Station, StationId, StationNames};
use std::env;

#[allow(dead_code)]
pub fn pick_station(query: &str) -> Result<Station, Box<dyn std::error::Error>> {
    let encoded_query = urlencoding::encode(query);
    let url = format!(
        "https://gateway.apiportal.ns.nl/nsapp-stations/v3?q={}&includeNonPlannableStations=false&limit=10",
        encoded_query
    );

    let ns_api_token = env::var("NS_API_TOKEN").map_err(|_| "NS_API_TOKEN not found")?;

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
            println!(
                "Your query `{}` was ambiguous, multiple stations matched:",
                query
            );
            for s in &response.payload {
                println!("{} - {}", s.id.uic_code, s.names.long);
            }
            Err("⚠️ Multiple stations matched. Please refine your query.".into())
        }
    }
}

#[allow(dead_code)]
pub fn get_all_stations() -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://gateway.apiportal.ns.nl/nsapp-stations/v3",);

    let ns_api_token = env::var("NS_API_TOKEN").map_err(|_| "NS_API_TOKEN not found")?;

    let body: String = ureq::get(url)
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &ns_api_token)
        .call()?
        .body_mut()
        .read_to_string()?;

    let response: ApiResponse = serde_json::from_str(&body).unwrap();
    for s in &response.payload {
        println!("(\"{}\", {}),", s.names.long, s.id.uic_code);
    }
    return Ok(());
}
pub fn pick_station_local(query: &str) -> Result<Station, Box<dyn std::error::Error>> {
    let q = query.to_lowercase();

    // 1️⃣ Exact (case-insensitive) match first
    if let Some((name, code)) = STATIONS.iter().find(|(key, _)| key.to_lowercase() == q) {
        return Ok(Station {
            id: StationId {
                uic_code: code.to_string(),
            },
            names: StationNames {
                long: name.to_string(),
            },
        });
    }

    // 2️⃣ Fall back to case-insensitive substring matches
    let matches: Vec<&(&str, i32)> = STATIONS
        .iter()
        .filter(|(key, _)| key.to_lowercase().contains(&q))
        .collect();

    match matches.len() {
        0 => Err("❌ No stations found for your query".into()),
        1 => {
            let (name, code) = *matches[0];
            Ok(Station {
                id: StationId {
                    uic_code: code.to_string(),
                },
                names: StationNames {
                    long: name.to_string(),
                },
            })
        }
        _ => {
            println!(
                "Your query `{}` was ambiguous, multiple stations matched:",
                query
            );
            for m in matches {
                let (name, code) = *m;
                println!("{} - {}", code, name);
            }
            Err("⚠️ Multiple stations matched. Please refine your query.".into())
        }
    }
}
