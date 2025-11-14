use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use clap::Parser;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use nstimes::{cache::PriceCache, prices, stations::{self, StationLookupResult}};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Enable Swagger UI documentation at /swagger-ui
    #[arg(long)]
    docs: bool,

    /// Enable price caching with specified file path
    #[arg(long)]
    cache: Option<String>,
}

// Application state shared across handlers
#[derive(Clone)]
struct AppState {
    cache: Option<Arc<PriceCache>>,
}

#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
struct PriceQuery {
    /// Origin station name (e.g., "Amsterdam Centraal")
    from: String,
    /// Destination station name (e.g., "Utrecht Centraal")
    to: String,
    /// Travel class: 1 for first class, 2 for second class (default: 2)
    #[serde(default = "default_class")]
    #[param(default = 2, minimum = 1, maximum = 2)]
    class: u8,
}

fn default_class() -> u8 {
    2
}

#[derive(Serialize, utoipa::ToSchema)]
struct PriceResponse {
    /// Full name of the origin station
    from: String,
    /// Full name of the destination station
    to: String,
    /// Price in cents
    #[schema(example = 940)]
    price_cents: i32,
    /// Travel class description
    #[schema(example = "2nd class")]
    travel_class: String,
}

#[derive(Serialize, utoipa::ToSchema)]
struct StationMatch {
    /// Station name
    #[schema(example = "Amsterdam Centraal")]
    name: String,
    /// UIC station code
    #[schema(example = 8400058)]
    uic_code: i32,
}

#[derive(Serialize, utoipa::ToSchema)]
struct ErrorResponse {
    /// Error message
    error: String,
    /// List of matching stations (if query was ambiguous)
    #[serde(skip_serializing_if = "Option::is_none")]
    matches: Option<Vec<StationMatch>>,
}

#[utoipa::path(
    get,
    path = "/price",
    params(PriceQuery),
    responses(
        (status = 200, description = "Price information retrieved successfully", body = PriceResponse),
        (status = 400, description = "Invalid input or ambiguous station name", body = ErrorResponse),
        (status = 404, description = "No prices found for this route", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "prices"
)]
async fn get_price(
    State(state): State<AppState>,
    Query(params): Query<PriceQuery>,
) -> impl IntoResponse {
    // Validate class parameter
    if params.class != 1 && params.class != 2 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "class must be 1 or 2".to_string(),
                matches: None,
            }),
        )
            .into_response();
    }

    // Lookup stations
    let station_from = match stations::lookup_station_local(&params.from) {
        StationLookupResult::Single(s) => s,
        StationLookupResult::None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("No stations found for 'from' query: {}", params.from),
                    matches: None,
                }),
            )
                .into_response();
        }
        StationLookupResult::Multiple(matches) => {
            let match_list = matches
                .into_iter()
                .map(|(name, uic_code)| StationMatch { name, uic_code })
                .collect();
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Multiple stations matched for 'from' query: {}. Please refine your query.", params.from),
                    matches: Some(match_list),
                }),
            )
                .into_response();
        }
    };

    let station_to = match stations::lookup_station_local(&params.to) {
        StationLookupResult::Single(s) => s,
        StationLookupResult::None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("No stations found for 'to' query: {}", params.to),
                    matches: None,
                }),
            )
                .into_response();
        }
        StationLookupResult::Multiple(matches) => {
            let match_list = matches
                .into_iter()
                .map(|(name, uic_code)| StationMatch { name, uic_code })
                .collect();
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Multiple stations matched for 'to' query: {}. Please refine your query.", params.to),
                    matches: Some(match_list),
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

    // Fetch price (with cache if available)
    let cache_ref = state.cache.as_ref().map(|arc| arc.as_ref());

    let response = match prices::get_prices(
        &station_from,
        &station_to,
        travel_class,
        Some("single"),
        cache_ref,
    ) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to fetch prices: {}", e),
                    matches: None,
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
                matches: None,
            }),
        )
            .into_response()
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = inline(Object))
    ),
    tag = "health"
)]
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}

#[derive(OpenApi)]
#[openapi(
    paths(get_price, health_check),
    components(schemas(PriceResponse, ErrorResponse, StationMatch)),
    tags(
        (name = "prices", description = "Train ticket price endpoints"),
        (name = "health", description = "Health check endpoint")
    ),
    info(
        title = "NSTimes API",
        version = "0.1.0",
        description = "Dutch railway (NS) travel information API - get train ticket prices",
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let args = Args::parse();

    // Initialize cache if --cache flag is provided
    let cache = if let Some(cache_path) = &args.cache {
        match PriceCache::new(cache_path) {
            Ok(c) => {
                println!("💾 Cache enabled: {}", cache_path);
                Some(Arc::new(c))
            }
            Err(e) => {
                eprintln!("⚠️  Failed to initialize cache: {}", e);
                None
            }
        }
    } else {
        None
    };

    let state = AppState { cache };

    let mut app = Router::new()
        .route("/price", get(get_price))
        .route("/health", get(health_check))
        .with_state(state);

    if args.docs {
        let swagger_ui = SwaggerUi::new("/docs")
            .url("/docs/openapi.json", ApiDoc::openapi());
        app = app.merge(swagger_ui);
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 Server running on http://localhost:3000");
    if args.docs {
        println!("   📚 Docs: http://localhost:3000/docs");
        println!("   📄 OpenAPI spec: http://localhost:3000/docs/openapi.json");
    }

    axum::serve(listener, app).await.unwrap();
}
