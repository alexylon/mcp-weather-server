use anyhow::Result;
use reqwest::Client;
use rmcp::{
    handler::server::{wrapper::Parameters, ServerHandler, tool::ToolRouter},
    model::{CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServiceExt,
    ErrorData as McpError,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const USER_AGENT: &str = "mcp-rust-weather-server/0.1.0";
const NWS_API_BASE: &str = "https://api.weather.gov";
const OPEN_METEO_API_BASE: &str = "https://api.open-meteo.com/v1";

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    latitude: f64,
    longitude: f64,
    timezone: String,
    daily: DailyData,
    daily_units: DailyUnits,
}

#[derive(Debug, Deserialize)]
struct DailyData {
    time: Vec<String>,
    #[serde(rename = "temperature_2m_max")]
    temperature_max: Vec<f64>,
    #[serde(rename = "temperature_2m_min")]
    temperature_min: Vec<f64>,
    #[serde(rename = "weather_code")]
    weather_code: Vec<i32>,
    #[serde(rename = "wind_speed_10m_max")]
    wind_speed_max: Vec<f64>,
    #[serde(rename = "precipitation_sum")]
    precipitation_sum: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct DailyUnits {
    #[serde(rename = "temperature_2m_max")]
    temperature_max: String,
    #[serde(rename = "wind_speed_10m_max")]
    wind_speed_max: String,
    #[serde(rename = "precipitation_sum")]
    precipitation_sum: String,
}

#[derive(Debug, Deserialize)]
struct AlertResponse {
    features: Vec<AlertFeature>,
}

#[derive(Debug, Deserialize)]
struct AlertFeature {
    properties: AlertProperties,
}

#[derive(Debug, Deserialize)]
struct AlertProperties {
    event: String,
    headline: Option<String>,
    description: Option<String>,
    severity: String,
    #[serde(rename = "areaDesc")]
    area_desc: String,
}

#[derive(Debug, Deserialize)]
struct PointsResponse {
    properties: PointsProperties,
}

#[derive(Debug, Deserialize)]
struct PointsProperties {
    #[serde(rename = "gridId")]
    grid_id: String,
    #[serde(rename = "gridX")]
    grid_x: i32,
    #[serde(rename = "gridY")]
    grid_y: i32,
}

#[derive(Debug, Deserialize)]
struct ForecastResponse {
    properties: ForecastProperties,
}

#[derive(Debug, Deserialize)]
struct ForecastProperties {
    periods: Vec<ForecastPeriod>,
}

#[derive(Debug, Deserialize)]
struct ForecastPeriod {
    name: String,
    temperature: i32,
    #[serde(rename = "temperatureUnit")]
    temperature_unit: String,
    #[serde(rename = "windSpeed")]
    wind_speed: String,
    #[serde(rename = "windDirection")]
    wind_direction: String,
    #[serde(rename = "shortForecast")]
    short_forecast: String,
    #[serde(rename = "detailedForecast")]
    detailed_forecast: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct GetAlertsRequest {
    state: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct GetForecastRequest {
    latitude: f64,
    longitude: f64,
}

#[derive(Clone)]
struct Weather {
    client: Arc<Client>,
    tool_router: ToolRouter<Self>,
}

impl Weather {
    fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()?;

        Ok(Self {
            client: Arc::new(client),
            tool_router: Self::tool_router(),
        })
    }

    async fn make_request<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        let response = self.client
            .get(url)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Request failed with status: {}", response.status());
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }

    fn format_alerts(&self, alerts: AlertResponse) -> String {
        if alerts.features.is_empty() {
            return "No active weather alerts.".to_string();
        }

        let mut output = String::from("Active Weather Alerts:\n\n");
        for (i, feature) in alerts.features.iter().enumerate() {
            let props = &feature.properties;
            output.push_str(&format!(
                "Alert {}:\n  Event: {}\n  Severity: {}\n  Area: {}\n",
                i + 1, props.event, props.severity, props.area_desc
            ));
            if let Some(headline) = &props.headline {
                output.push_str(&format!("  Headline: {}\n", headline));
            }
            if let Some(description) = &props.description {
                output.push_str(&format!("  Description: {}\n", description));
            }
            output.push('\n');
        }
        output
    }

    fn format_forecast(&self, forecast: ForecastResponse) -> String {
        let mut output = String::from("Weather Forecast:\n\n");
        for period in forecast.properties.periods {
            output.push_str(&format!(
                "{}:\n  Temperature: {}\u{00b0}{}\n  Wind: {} {}\n  Conditions: {}\n  Details: {}\n\n",
                period.name,
                period.temperature,
                period.temperature_unit,
                period.wind_speed,
                period.wind_direction,
                period.short_forecast,
                period.detailed_forecast
            ));
        }
        output
    }

    fn format_open_meteo_forecast(&self, forecast: OpenMeteoResponse) -> String {
        let mut output = format!(
            "Weather Forecast (Open-Meteo)\nLocation: {:.4}, {:.4}\nTimezone: {}\n\n",
            forecast.latitude, forecast.longitude, forecast.timezone
        );

        for i in 0..forecast.daily.time.len().min(7) {
            let weather_desc = Self::weather_code_to_description(forecast.daily.weather_code[i]);
            output.push_str(&format!(
                "{}:\n  Temperature: {:.1}\u{00b0}{} - {:.1}\u{00b0}{}\n  Conditions: {}\n  Wind Speed: {:.1} {}\n  Precipitation: {:.1} {}\n\n",
                forecast.daily.time[i],
                forecast.daily.temperature_min[i],
                forecast.daily_units.temperature_max,
                forecast.daily.temperature_max[i],
                forecast.daily_units.temperature_max,
                weather_desc,
                forecast.daily.wind_speed_max[i],
                forecast.daily_units.wind_speed_max,
                forecast.daily.precipitation_sum[i],
                forecast.daily_units.precipitation_sum
            ));
        }
        output
    }

    fn weather_code_to_description(code: i32) -> &'static str {
        match code {
            0 => "Clear sky",
            1 => "Mainly clear",
            2 => "Partly cloudy",
            3 => "Overcast",
            45 | 48 => "Foggy",
            51 | 53 | 55 => "Drizzle",
            61 | 63 | 65 => "Rain",
            71 | 73 | 75 => "Snow",
            77 => "Snow grains",
            80 | 81 | 82 => "Rain showers",
            85 | 86 => "Snow showers",
            95 => "Thunderstorm",
            96 | 99 => "Thunderstorm with hail",
            _ => "Unknown",
        }
    }

    fn is_us_location(latitude: f64, longitude: f64) -> bool {
        // Continental US, Alaska, Hawaii, and territories
        latitude >= 24.0 && latitude <= 72.0 && longitude >= -180.0 && longitude <= -60.0
    }
}

#[tool_handler]
impl ServerHandler for Weather {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "mcp-rust-weather".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: None,
                website_url: None,
            },
            instructions: Some(
                "A weather information service powered by the National Weather Service API. \
                Provides weather alerts and forecasts for US locations."
                    .to_string(),
            ),
        }
    }
}

#[tool_router]
impl Weather {
    #[tool(description = "Get active weather alerts for a US state. Provide a two-letter state code (e.g., 'CA' for California, 'NY' for New York).")]
    async fn get_alerts(
        &self,
        Parameters(request): Parameters<GetAlertsRequest>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Getting alerts for state: {}", request.state);

        let url = format!("{}/alerts/active?area={}", NWS_API_BASE, request.state);

        let alerts = self.make_request::<AlertResponse>(&url)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to fetch alerts: {}", e), None))?;

        let formatted = self.format_alerts(alerts);

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    #[tool(description = "Get weather forecast for any location worldwide. Provide latitude and longitude (e.g., latitude: 52.52, longitude: 13.41 for Berlin, or latitude: 40.7128, longitude: -74.0060 for New York). Automatically uses the best weather service for the location (NWS for US, Open-Meteo for rest of world).")]
    async fn get_forecast(
        &self,
        Parameters(request): Parameters<GetForecastRequest>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!(
            "Getting forecast for coordinates: {}, {}",
            request.latitude,
            request.longitude
        );

        if Self::is_us_location(request.latitude, request.longitude) {
            self.get_forecast_nws(request).await
        } else {
            self.get_forecast_open_meteo(request).await
        }
    }

    async fn get_forecast_nws(
        &self,
        request: GetForecastRequest,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Using NWS API for US location");

        let points_url = format!(
            "{}/points/{},{}",
            NWS_API_BASE, request.latitude, request.longitude
        );

        let points = self.make_request::<PointsResponse>(&points_url)
            .await
            .map_err(|e| {
                if e.to_string().contains("404") {
                    McpError::invalid_params(
                        "Location not found in NWS coverage area. This location may be in US waters not covered by the grid system.",
                        None
                    )
                } else {
                    McpError::internal_error(format!("Failed to fetch grid points: {}", e), None)
                }
            })?;

        let forecast_url = format!(
            "{}/gridpoints/{}/{},{}/forecast",
            NWS_API_BASE,
            points.properties.grid_id,
            points.properties.grid_x,
            points.properties.grid_y
        );

        let forecast = self.make_request::<ForecastResponse>(&forecast_url)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to fetch forecast: {}", e), None))?;

        let formatted = self.format_forecast(forecast);

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    async fn get_forecast_open_meteo(
        &self,
        request: GetForecastRequest,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Using Open-Meteo API for non-US location");

        let url = format!(
            "{}/forecast?latitude={}&longitude={}&daily=temperature_2m_max,temperature_2m_min,weather_code,wind_speed_10m_max,precipitation_sum&timezone=auto",
            OPEN_METEO_API_BASE, request.latitude, request.longitude
        );

        let forecast = self.make_request::<OpenMeteoResponse>(&url)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to fetch Open-Meteo forecast: {}", e), None))?;

        let formatted = self.format_open_meteo_forecast(forecast);

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcp_rust_weather=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting MCP weather server");

    let weather = Weather::new()?;
    let server = weather.serve(rmcp::transport::stdio()).await?;
    server.waiting().await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}
