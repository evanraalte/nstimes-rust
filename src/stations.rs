use crate::response_models::{ApiResponse, Station};
use std::env;

pub fn pick_station(query: &str) -> Result<Station, Box<dyn std::error::Error>> {
    let encoded_query = urlencoding::encode(query);
    let url = format!(
        "https://gateway.apiportal.ns.nl/nsapp-stations/v3?q={}&includeNonPlannableStations=false&limit=10",
        encoded_query
    );

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
