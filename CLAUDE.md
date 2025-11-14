# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NSTimes is a command-line tool for querying Dutch railway (NS) travel information directly in the terminal. It fetches real-time train schedules between stations, displaying departure/arrival times, delays, track numbers, and cancellations with colored formatting.

## Essential Commands

### Development
```bash
# Run the trip command to find journeys between two stations
cargo run trip "Den Haag C" "Amersfoort C"

# Build optimized release binary
cargo build --release

# Run tests (if any exist)
cargo test

# Show available commands and help
cargo run -- --help
cargo run trip --help
```

### Environment Setup
Create a `.env` file with your NS API token:
```
NS_API_TOKEN=your_token_here
```

Get a token from the [NS API portal](https://apiportal.ns.nl/signin) by creating an account and generating credentials [here](https://apiportal.ns.nl/api-details#api=reisinformatie-api).

## Architecture

### Module Structure

The codebase follows a modular architecture with three main components:

1. **`main.rs`** - Entry point using `clap` with subcommand support. Loads environment variables and routes to the appropriate command handler. Currently supports the `trip` subcommand for querying journeys between stations.

2. **`stations/`** - Station lookup and resolution
   - `models.rs`: Serde models for NS stations API responses (`Station`, `StationId`, `StationNames`)
   - `service.rs`: Station lookup logic with two modes:
     - `pick_station_local()`: Fast local lookup using the hardcoded `STATIONS` constant (preferred, used by default)
     - `pick_station()`: Live API call to NS stations endpoint (unused but available)
   - Ambiguous queries (multiple matches) are caught and displayed to the user for refinement

3. **`trips/`** - Journey/trip fetching and display
   - `models.rs`: Serde models for NS trips API responses (`TripsResponse`, `TripRaw`, `LegRaw`, `StopRaw`, `ProductRaw`)
   - `service.rs`:
     - `trips()` function queries the NS Reisinformatie API for journeys between two stations
     - `Trip` struct: Processed trip data with both planned and actual times
     - Custom `Display` implementation formats trips with colored delays and strikethrough for cancelled trips
     - Only displays the first leg of each journey (direct trains)

4. **`constants.rs`** - Contains `STATIONS` array with ~630 European station names mapped to UIC codes. This enables offline station lookup without API calls.

### Key Design Decisions

- **Local-first station resolution**: The app uses a hardcoded station list to avoid unnecessary API calls and provide instant autocomplete-like behavior
- **Error handling**: Uses `Result<(), Box<dyn std::error::Error>>` throughout with user-friendly error messages (e.g., "❌ No stations found")
- **Date/time handling**: Uses `chrono` with `FixedOffset` to properly handle timezone-aware datetime strings from the NS API
- **Display formatting**: Uses `colored` crate for terminal output with red delays and strikethrough for cancelled trains

### API Integration

The app integrates with two NS API endpoints:
1. **Stations API** (v3): `https://gateway.apiportal.ns.nl/nsapp-stations/v3` - queries stations (currently unused in favor of local lookup)
2. **Trips API** (v3): `https://gateway.apiportal.ns.nl/reisinformatie-api/api/v3/trips` - fetches journey options between stations

Both require the `Ocp-Apim-Subscription-Key` header with the NS API token.

### Release Configuration

The `Cargo.toml` profile is optimized for minimal binary size:
- `opt-level = "z"`: Optimize for size
- `lto = true`: Link-time optimization
- `codegen-units = 1`: Better optimization (slower compile, smaller binary)
- `strip = "symbols"`: Remove debug symbols

GitHub Actions workflow (`release.yml`) builds cross-platform binaries (Windows, Linux musl, macOS) on release creation.
