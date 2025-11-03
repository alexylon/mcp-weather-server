# MCP Weather Server

Global weather MCP server with automatic API selection - uses NWS for US locations and Open-Meteo for worldwide coverage.

## Features

- **get_alerts**: Weather alerts for US states
- **get_forecast**: Global weather forecasts (any coordinates worldwide)
- No API keys required
- Automatic API selection based on location

## Quick Start

```bash
cargo build --release
npx @modelcontextprotocol/inspector ./target/release/mcp-weather-server
```

Open `http://127.0.0.1:6274` and test with coordinates like Berlin (52.52, 13.41) or NYC (40.7128, -74.0060).

## Usage

### Testing with MCP Inspector

```bash
npx @modelcontextprotocol/inspector cargo run
# or
npx @modelcontextprotocol/inspector ./target/release/mcp-weather-server
```

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "weather": {
      "command": "/path/to/mcp-weather-server/target/release/mcp-weather-server"
    }
  }
}
```

## Tools

### get_alerts
- **Input**: `state` (two-letter US code, e.g., "CA")
- **Output**: Active weather alerts with severity and descriptions

### get_forecast
- **Input**: `latitude`, `longitude` (any coordinates worldwide)
- **Output**: Weather forecast (NWS for US, Open-Meteo for rest of world)

**Example coordinates:**
- New York: 40.7128, -74.0060
- Berlin: 52.52, 13.41
- Tokyo: 35.6762, 139.6503

## Development

```bash
# Run with logging
RUST_LOG=debug cargo run

# Build release
cargo build --release
```

## Resources

- [MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [Open-Meteo API](https://open-meteo.com/)
- [National Weather Service API](https://www.weather.gov/documentation/services-web-api)
