use crate::prices::models::PriceApiResponse;
use crate::stations::models::Station;
use std::env;

pub fn get_prices(
    from: &Station,
    to: &Station,
    travel_class: Option<&str>,
    travel_type: Option<&str>,
) -> Result<PriceApiResponse, Box<dyn std::error::Error>> {
    let url = "https://gateway.apiportal.ns.nl/reisinformatie-api/api/v3/price";

    let ns_api_token = env::var("NS_API_TOKEN").map_err(|_| "NS_API_TOKEN not found")?;

    let request = ureq::get(url)
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &ns_api_token)
        .query("fromStation", &from.id.uic_code)
        .query("toStation", &to.id.uic_code)
        .query("travelClass", travel_class.unwrap_or("SECOND_CLASS"))
        .query("travelType", travel_type.unwrap_or("single"))
        .query("isJointJourney", "false")
        .query("adults", "1")
        .query("children", "0");

    let body: String = request.call()?.body_mut().read_to_string()?;

    let response: PriceApiResponse = serde_json::from_str(&body)?;

    Ok(response)
}
