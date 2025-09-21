use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TripsResponse {
    pub trips: Vec<TripRaw>,
}

#[derive(Debug, Deserialize)]
pub struct TripRaw {
    pub legs: Vec<LegRaw>,
}

#[derive(Debug, Deserialize)]
pub struct LegRaw {
    pub origin: StopRaw,
    pub destination: StopRaw,

    #[serde(default)]
    pub cancelled: bool,

    pub product: ProductRaw,
}

#[derive(Debug, Deserialize)]
pub struct StopRaw {
    pub name: String,

    #[serde(rename = "actualTrack")]
    pub actual_track: Option<String>,

    #[serde(rename = "plannedTrack")]
    pub planned_track: Option<String>,

    #[serde(rename = "plannedDateTime")]
    pub planned_date_time: String,

    #[serde(rename = "actualDateTime")]
    pub actual_date_time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProductRaw {
    #[serde(rename = "categoryCode")]
    pub category_code: String,
}
