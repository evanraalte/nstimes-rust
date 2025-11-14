use crate::cache::PriceCache;
use crate::prices::models::PriceApiResponse;
use crate::stations::models::Station;
use std::env;

pub fn get_prices(
    from: &Station,
    to: &Station,
    travel_class: Option<&str>,
    travel_type: Option<&str>,
    cache: Option<&PriceCache>,
) -> Result<PriceApiResponse, Box<dyn std::error::Error>> {
    // Only use cache for single trips (not return trips)
    let use_cache = cache.is_some() && travel_type.unwrap_or("single") == "single";

    // Convert travel_class string to u8 for cache lookup
    let class_num = match travel_class.unwrap_or("SECOND_CLASS") {
        "FIRST_CLASS" => 1,
        _ => 2,
    };

    // Check cache first
    if use_cache {
        if let Some(cached_price) = cache.unwrap().get(&from.names.long, &to.names.long, class_num) {
            // Return a mock response with the cached price
            return Ok(create_cached_response(
                cached_price,
                travel_class.unwrap_or("SECOND_CLASS"),
            ));
        }
    }

    // Cache miss or caching disabled - fetch from API
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

    // Update cache with the first price if available
    if use_cache {
        if let Some(first_price) = response.payload.prices.first() {
            let _ = cache
                .unwrap()
                .set(&from.names.long, &to.names.long, class_num, first_price.total_price_in_cents as u32);
        }
    }

    Ok(response)
}

/// Create a cached response with minimal data
fn create_cached_response(price_cents: u32, travel_class: &str) -> PriceApiResponse {
    use crate::prices::models::{Price, PricesResponse};

    PriceApiResponse {
        payload: PricesResponse {
            prices: vec![Price {
                travel_class: travel_class.to_string(),
                total_price_in_cents: price_cents as i32,
                price_per_adult_in_cents: price_cents as i32,
                discount_in_cents: None,
                discount_type: "NONE".to_string(),
                is_best_option: true,
                display_name: "Cached Price".to_string(),
                operator_name: None,
            }],
        },
    }
}
