use crate::stations_models::Station;
use crate::trips_models::{TripRaw, TripsResponse};
use chrono::{DateTime, FixedOffset};
use std::{env, fmt};

#[derive(Debug)]
pub struct Trip {
    pub origin_name: String,
    pub destination_name: String,
    pub track: String,
    pub cancelled: bool,
    pub departure_time: DateTime<FixedOffset>,
    pub arrival_time: DateTime<FixedOffset>,
    pub train_type: String,
}

impl From<TripRaw> for Trip {
    fn from(raw: TripRaw) -> Self {
        // we only care about the first leg
        let leg = raw.legs.into_iter().next().expect("No legs in trip");

        let track = leg
            .origin
            .actual_track
            .or(leg.origin.planned_track)
            .unwrap_or_else(|| "?".to_string());

        let parse_time = |txt: String| {
            DateTime::parse_from_str(&txt, "%Y-%m-%dT%H:%M:%S%z").expect("Invalid datetime format")
        };

        Trip {
            origin_name: leg.origin.name,
            destination_name: leg.destination.name,
            track,
            cancelled: leg.cancelled,
            departure_time: parse_time(leg.origin.planned_date_time),
            arrival_time: parse_time(leg.destination.planned_date_time),
            train_type: leg.product.category_code,
        }
    }
}

impl fmt::Display for Trip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {} [{}] tr.{} {}->{} {}",
            self.origin_name,
            self.destination_name,
            self.train_type,
            self.track,
            self.departure_time.format("%H:%M"),
            self.arrival_time.format("%H:%M"),
            if self.cancelled { "(cancelled)" } else { "" }
        )
    }
}

pub fn trips(from: Station, to: Station) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://gateway.apiportal.ns.nl/reisinformatie-api/api/v3/trips");

    let ns_api_token = env::var("NS_API_TOKEN").map_err(|_| "NS_API_TOKEN not found")?;

    let body: String = ureq::get(url)
        .header("Cache-Control", "no-cache")
        .header("Ocp-Apim-Subscription-Key", &ns_api_token)
        .query("originUicCode", from.id.uic_code)
        .query("destinationUicCode", to.id.uic_code)
        .call()?
        .body_mut()
        .read_to_string()?;

    let resp: TripsResponse = serde_json::from_str(&body)?;
    let trips: Vec<Trip> = resp.trips.into_iter().map(Trip::from).collect();

    for t in &trips {
        println!("{}", t);
    }
    Ok(())
}
