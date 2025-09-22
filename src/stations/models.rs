use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub payload: Vec<Station>,
}

#[derive(Debug, Deserialize)]
pub struct Station {
    pub id: StationId,
    pub names: StationNames,
}

#[derive(Debug, Deserialize)]
pub struct StationId {
    #[serde(rename = "uicCode")]
    pub uic_code: String,
}

#[derive(Debug, Deserialize)]
pub struct StationNames {
    pub long: String,
}
