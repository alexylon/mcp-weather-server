use anyhow::Result;
use reqwest::Client;
use rmcp::{
    handler::server::{wrapper::Parameters, ServerHandler, tool::ToolRouter},
    model::{CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    ErrorData as McpError,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::constants::{NWS_API_BASE, OPEN_METEO_API_BASE, USER_AGENT};
use crate::formatters::{format_alerts, format_forecast, format_open_meteo_forecast};
use crate::models::{
    AlertResponse, ForecastResponse, GetAlertsRequest, GetForecastRequest,
    OpenMeteoResponse, PointsResponse,
};

/// Main weather service that handles MCP requests
#[derive(Clone)]
pub struct Weather {
    client: Arc<Client>,
    tool_router: ToolRouter<Self>,
}

impl Weather {
    /// Creates a new Weather service instance
    pub fn new() -> Result<Self> {
        let client = Client::builder().user_agent(USER_AGENT).build()?;

        Ok(Self {
            client: Arc::new(client),
            tool_router: Self::tool_router(),
        })
    }

    /// Makes an HTTP GET request and deserializes the JSON response
    async fn make_request<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Request failed with status: {}", response.status());
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }

    /// Determines if coordinates are within US coverage area
    fn is_us_location(latitude: f64, longitude: f64) -> bool {
        // Continental US, Alaska, Hawaii, and territories
        latitude >= 24.0 && latitude <= 72.0 && longitude >= -180.0 && longitude <= -60.0
    }

    /// Gets forecast using NWS API for US locations
    async fn get_forecast_nws(
        &self,
        request: GetForecastRequest,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Using NWS API for US location");

        let points_url = format!(
            "{}/points/{},{}",
            NWS_API_BASE, request.latitude, request.longitude
        );

        let points = self
            .make_request::<PointsResponse>(&points_url)
            .await
            .map_err(|e| {
                if e.to_string().contains("404") {
                    McpError::invalid_params(
                        "Location not found in NWS coverage area. This location may be in US waters not covered by the grid system.",
                        None,
                    )
                } else {
                    McpError::internal_error(
                        format!("Failed to fetch grid points: {}", e),
                        None,
                    )
                }
            })?;

        let forecast_url = format!(
            "{}/gridpoints/{}/{},{}/forecast",
            NWS_API_BASE,
            points.properties.grid_id,
            points.properties.grid_x,
            points.properties.grid_y
        );

        let forecast = self
            .make_request::<ForecastResponse>(&forecast_url)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to fetch forecast: {}", e), None)
            })?;

        let formatted = format_forecast(forecast);

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    /// Gets forecast using Open-Meteo API for non-US locations
    async fn get_forecast_open_meteo(
        &self,
        request: GetForecastRequest,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Using Open-Meteo API for non-US location");

        let url = format!(
            "{}/forecast?latitude={}&longitude={}&daily=temperature_2m_max,temperature_2m_min,weather_code,wind_speed_10m_max,precipitation_sum&timezone=auto",
            OPEN_METEO_API_BASE, request.latitude, request.longitude
        );

        let forecast = self
            .make_request::<OpenMeteoResponse>(&url)
            .await
            .map_err(|e| {
                McpError::internal_error(
                    format!("Failed to fetch Open-Meteo forecast: {}", e),
                    None,
                )
            })?;

        let formatted = format_open_meteo_forecast(forecast);

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
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
    /// Gets active weather alerts for a US state
    #[tool(description = "Get active weather alerts for a US state. Provide a two-letter state code (e.g., 'CA' for California, 'NY' for New York).")]
    async fn get_alerts(
        &self,
        Parameters(request): Parameters<GetAlertsRequest>,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("Getting alerts for state: {}", request.state);

        let url = format!("{}/alerts/active?area={}", NWS_API_BASE, request.state);

        let alerts = self
            .make_request::<AlertResponse>(&url)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to fetch alerts: {}", e), None)
            })?;

        let formatted = format_alerts(alerts);

        Ok(CallToolResult::success(vec![Content::text(formatted)]))
    }

    /// Gets weather forecast for any location worldwide
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
}
