# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NSTimes is a Rust project providing Dutch railway (NS) travel information through two interfaces:
1. **CLI tool** - Command-line interface for querying train schedules and prices in the terminal
2. **API server** - HTTP JSON API for programmatic access to price information

Both binaries share the same core library code for NS API integration.

## Essential Commands

### Development

**CLI Commands:**
```bash
# Run the trip command to find journeys between two stations
cargo run --bin cli trip "Den Haag C" "Amersfoort C"

# Get price information for a trip (defaults to 2nd class, single trip)
cargo run --bin cli price "Den Haag C" "Amersfoort C"

# Get price for 1st class return trip
cargo run --bin cli price "Den Haag C" "Amersfoort C" --class 1 --return

# Enable price caching to avoid redundant API calls
cargo run --bin cli -- --cache prices.cache price "Den Haag C" "Amersfoort C"

# Show available commands and help
cargo run --bin cli -- --help
```

**API Server:**
```bash
# Run the API server (default port 3000)
cargo run --bin server

# Run the API server with Swagger UI documentation enabled
cargo run --bin server -- --docs

# Run with price caching enabled
cargo run --bin server -- --cache prices.cache

# Run with both documentation and caching
cargo run --bin server -- --docs --cache prices.cache

# Query price via API
curl "http://localhost:3000/price?from=Amsterdam+Centraal&to=Utrecht+Centraal&class=2"
```

**Building:**
```bash
# Build both binaries
cargo build --release

# Build only CLI
cargo build --bin cli --release

# Build only API server
cargo build --bin server --release

# Run tests (if any exist)
cargo test
```

### Environment Setup
Create a `.env` file with your NS API token:
```
NS_API_TOKEN=your_token_here
```

Get a token from the [NS API portal](https://apiportal.ns.nl/signin) by creating an account and generating credentials [here](https://apiportal.ns.nl/api-details#api=reisinformatie-api).

### Docker Deployment

**Quick start with Docker Compose:**
```bash
# 1. Create .env file with your NS_API_TOKEN
cp .env.example .env
# Edit .env and add your token

# 2. Start the server (with caching enabled by default)
docker compose up -d

# 3. Check logs
docker compose logs -f

# 4. Stop the server
docker compose down
```

**Cache Configuration:**
By default, Docker Compose enables price caching with a persistent volume at `./cache`. The cache file (`/data/prices.cache` in the container) persists across restarts, reducing API calls and improving response times.

To disable caching, edit `docker-compose.yml` and remove `--cache /data/prices.cache` from the command line, then restart:
```bash
docker compose up -d
```

**Enable/Disable Swagger UI documentation:**
Edit `docker-compose.yml` and add/remove `--docs` from the command line:
```yaml
command: ["--docs", "--cache", "/data/prices.cache"]  # Both enabled
command: ["--cache", "/data/prices.cache"]            # Cache only
command: ["--docs"]                                    # Docs only
command: []                                            # Neither
```

**Manual Docker build:**
```bash
# Build the image
docker build -t nstimes-api .

# Run the container
docker run -d \
  --name nstimes-api \
  -p 3000:3000 \
  -e NS_API_TOKEN=your_token_here \
  nstimes-api

# Run with documentation enabled
docker run -d \
  --name nstimes-api \
  -p 3000:3000 \
  -e NS_API_TOKEN=your_token_here \
  nstimes-api --docs

# Run with caching enabled (with persistent volume)
docker run -d \
  --name nstimes-api \
  -p 3000:3000 \
  -v $(pwd)/cache:/data \
  -e NS_API_TOKEN=your_token_here \
  nstimes-api --cache /data/prices.cache

# Run with both documentation and caching
docker run -d \
  --name nstimes-api \
  -p 3000:3000 \
  -v $(pwd)/cache:/data \
  -e NS_API_TOKEN=your_token_here \
  nstimes-api --docs --cache /data/prices.cache
```

## Architecture

### Project Structure

The codebase uses a **library + multiple binaries** architecture:

- **`src/lib.rs`** - Core library exposing shared modules
- **`src/bin/cli.rs`** - CLI binary using `clap` for command-line interface
- **`src/bin/server.rs`** - API server binary using `axum` for HTTP endpoints
- **Shared modules** - `cache/`, `stations/`, `prices/`, `trips/`, `commands/`, `constants.rs` used by both binaries

### Module Structure

The shared library contains six main components:

1. **`lib.rs`** - Exposes all public modules for use by binaries

2. **`cache/`** - Price caching system (JSON file-based storage)
   - `models.rs`: `CacheEntry` struct with price, travel class, and expiration date
   - `service.rs`: `PriceCache` implementation with thread-safe operations
     - Uses interior mutability (Mutex) for safe concurrent access
     - `get(from, to, class)` → returns `Option<u32>` (checks expiration automatically)
     - `set(from, to, class, price)` → stores price with automatic expiration date
     - Station pair normalization: A→B equals B→A (alphabetically sorted keys)
     - Automatic expiration: All prices expire on January 1st each year
     - Human-readable JSON format for easy debugging and manual editing

3. **`commands/`** - Command implementations (one file per command)
   - `trip.rs`: Implements the `trip` command which queries journeys between two stations. Orchestrates station lookup and trip fetching.
   - `price.rs`: Implements the `price` command which queries ticket prices. Supports optional flags for travel class (1st/2nd) and trip type (single/return).

4. **`stations/`** - Station lookup and resolution
   - `models.rs`: Serde models for NS stations API responses (`Station`, `StationId`, `StationNames`)
   - `service.rs`: Station lookup logic with two modes:
     - `pick_station_local()`: Fast local lookup using the hardcoded `STATIONS` constant (preferred, used by default)
     - `pick_station()`: Live API call to NS stations endpoint (unused but available)
   - Ambiguous queries (multiple matches) are caught and displayed to the user for refinement

5. **`trips/`** - Journey/trip fetching and display
   - `models.rs`: Serde models for NS trips API responses (`TripsResponse`, `TripRaw`, `LegRaw`, `StopRaw`, `ProductRaw`)
   - `service.rs`:
     - `trips()` function queries the NS Reisinformatie API for journeys between two stations
     - `Trip` struct: Processed trip data with both planned and actual times
     - Custom `Display` implementation formats trips with colored delays and strikethrough for cancelled trips
     - Only displays the first leg of each journey (direct trains)

6. **`prices/`** - Price information fetching and display
   - `models.rs`: Serde models for NS prices API responses (`PriceApiResponse`, `PricesResponse`, `Price`)
   - `service.rs`: `get_prices()` function queries the NS Price API with optional travel class and trip type parameters
     - Integrated caching: Checks cache first, falls back to API on miss
     - Automatic cache updates: Stores new prices after successful API fetch
     - Cache scope: Only single trips are cached (return trips always fetch fresh)

7. **`constants.rs`** - Contains `STATIONS` array with ~630 European station names mapped to UIC codes. This enables offline station lookup without API calls.

### Key Design Decisions

- **Local-first station resolution**: The app uses a hardcoded station list to avoid unnecessary API calls and provide instant autocomplete-like behavior
- **Price caching system**: Optional JSON file-based caching reduces API calls and improves response times
  - Bidirectional keys: A→B and B→A use the same cache entry (prices are the same in both directions)
  - Annual expiration: Prices automatically expire on January 1st each year
  - Thread-safe: Uses Mutex for safe concurrent access in the API server
  - Human-readable: JSON format allows easy debugging and manual cache inspection/editing
  - Configurable: Disabled by default, enabled via `--cache <path>` flag
- **Error handling**: Uses `Result<(), Box<dyn std::error::Error>>` throughout with user-friendly error messages (e.g., "❌ No stations found")
- **Date/time handling**: Uses `chrono` with `FixedOffset` to properly handle timezone-aware datetime strings from the NS API
- **Display formatting**: Uses `colored` crate for terminal output with red delays and strikethrough for cancelled trains

### API Server Endpoints

The HTTP API server exposes the following endpoints:

**GET /price**
- Query parameters:
  - `from` (required): Station name (e.g., "Amsterdam Centraal")
  - `to` (required): Station name (e.g., "Utrecht Centraal")
  - `class` (optional): Travel class, 1 or 2 (default: 2)
- Success response: `{"from": "Amsterdam Centraal", "to": "Utrecht Centraal", "price_cents": 940, "travel_class": "2nd class"}`
- Error response (ambiguous station): Returns error with list of matching stations for user to refine query
  - Example: `{"error": "Multiple stations matched for 'from' query: Amsterdam. Please refine your query.", "matches": [{"name": "Amsterdam Centraal", "uic_code": 8400058}, ...]}`

**GET /health**
- Returns: Simple health check response

**Documentation (when --docs flag is enabled):**
- **GET /docs**: Interactive Swagger UI documentation interface (similar to FastAPI's `/docs`)
- **GET /docs/openapi.json**: OpenAPI 3.0 specification in JSON format

To enable documentation, start the server with the `--docs` flag:
```bash
cargo run --bin server -- --docs
```

### NS API Integration

The app integrates with three NS API endpoints:
1. **Stations API** (v3): `https://gateway.apiportal.ns.nl/nsapp-stations/v3` - queries stations (currently unused in favor of local lookup)
2. **Trips API** (v3): `https://gateway.apiportal.ns.nl/reisinformatie-api/api/v3/trips` - fetches journey options between stations
3. **Price API** (v3): `https://gateway.apiportal.ns.nl/reisinformatie-api/api/v3/price` - fetches ticket price information with options for travel class (1st/2nd), trip type (single/return), and passenger counts

All require the `Ocp-Apim-Subscription-Key` header with the NS API token.

### Release Configuration

The `Cargo.toml` profile is optimized for minimal binary size:
- `opt-level = "z"`: Optimize for size
- `lto = true`: Link-time optimization
- `codegen-units = 1`: Better optimization (slower compile, smaller binary)
- `strip = "symbols"`: Remove debug symbols

GitHub Actions workflow (`release.yml`) builds cross-platform binaries (Windows, Linux musl, macOS) on release creation.
- update the claude.md file