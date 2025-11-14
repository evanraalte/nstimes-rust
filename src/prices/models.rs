use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PriceApiResponse {
    pub payload: PricesResponse,
}

#[derive(Debug, Deserialize)]
pub struct PricesResponse {
    pub prices: Vec<Price>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub total_price_in_cents: i32,
    pub price_per_adult_in_cents: i32,
    #[serde(default)]
    pub discount_in_cents: Option<i32>,
    #[serde(default)]
    pub operator_name: Option<String>,
    pub discount_type: String,
    pub travel_class: String,
    pub display_name: String,
    #[serde(default)]
    pub is_best_option: bool,
}
