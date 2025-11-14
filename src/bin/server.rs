use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use nstimes::{prices, stations};

#[derive(Deserialize)]
struct PriceQuery {
    from: String,
    to: String,
    #[serde(default = "default_class")]
    class: u8,
}

fn default_class() -> u8 {
    2
}

#[derive(Serialize)]
struct PriceResponse {
    from: String,
    to: String,
    price_cents: i32,
    travel_class: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn get_price(Query(params): Query<PriceQuery>) -> impl IntoResponse {
    // Validate class parameter
    if params.class != 1 && params.class != 2 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "class must be 1 or 2".to_string(),
            }),
        )
            .into_response();
    }

    // Lookup stations
    let station_from = match stations::pick_station_local(&params.from) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("From station error: {}", e),
                }),
            )
                .into_response();
        }
    };

    let station_to = match stations::pick_station_local(&params.to) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("To station error: {}", e),
                }),
            )
                .into_response();
        }
    };

    // Get travel class
    let travel_class = if params.class == 1 {
        Some("FIRST_CLASS")
    } else {
        Some("SECOND_CLASS")
    };

    // Fetch price
    let response = match prices::get_prices(&station_from, &station_to, travel_class, Some("single")) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to fetch prices: {}", e),
                }),
            )
                .into_response();
        }
    };

    // Extract first price
    if let Some(price) = response.payload.prices.first() {
        (
            StatusCode::OK,
            Json(PriceResponse {
                from: station_from.names.long,
                to: station_to.names.long,
                price_cents: price.total_price_in_cents,
                travel_class: if params.class == 1 {
                    "1st class".to_string()
                } else {
                    "2nd class".to_string()
                },
            }),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "No prices found for this route".to_string(),
            }),
        )
            .into_response()
    }
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let app = Router::new()
        .route("/price", get(get_price))
        .route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 Server running on http://localhost:3000");
    println!("   GET /price?from=Amsterdam+Centraal&to=Utrecht+Centraal&class=2");
    println!("   GET /health");

    axum::serve(listener, app).await.unwrap();
}
