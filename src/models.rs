use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ============================================================================
// Open-Meteo API Models
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct OpenMeteoResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub daily: DailyData,
    pub daily_units: DailyUnits,
}

#[derive(Debug, Deserialize)]
pub struct DailyData {
    pub time: Vec<String>,
    #[serde(rename = "temperature_2m_max")]
    pub temperature_max: Vec<f64>,
    #[serde(rename = "temperature_2m_min")]
    pub temperature_min: Vec<f64>,
    #[serde(rename = "weather_code")]
    pub weather_code: Vec<i32>,
    #[serde(rename = "wind_speed_10m_max")]
    pub wind_speed_max: Vec<f64>,
    #[serde(rename = "precipitation_sum")]
    pub precipitation_sum: Vec<f64>,
}

#[derive(Debug, Deserialize)]
pub struct DailyUnits {
    #[serde(rename = "temperature_2m_max")]
    pub temperature_max: String,
    #[serde(rename = "wind_speed_10m_max")]
    pub wind_speed_max: String,
    #[serde(rename = "precipitation_sum")]
    pub precipitation_sum: String,
}

// ============================================================================
// National Weather Service API Models
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AlertResponse {
    pub features: Vec<AlertFeature>,
}

#[derive(Debug, Deserialize)]
pub struct AlertFeature {
    pub properties: AlertProperties,
}

#[derive(Debug, Deserialize)]
pub struct AlertProperties {
    pub event: String,
    pub headline: Option<String>,
    pub description: Option<String>,
    pub severity: String,
    #[serde(rename = "areaDesc")]
    pub area_desc: String,
}

#[derive(Debug, Deserialize)]
pub struct PointsResponse {
    pub properties: PointsProperties,
}

#[derive(Debug, Deserialize)]
pub struct PointsProperties {
    #[serde(rename = "gridId")]
    pub grid_id: String,
    #[serde(rename = "gridX")]
    pub grid_x: i32,
    #[serde(rename = "gridY")]
    pub grid_y: i32,
}

#[derive(Debug, Deserialize)]
pub struct ForecastResponse {
    pub properties: ForecastProperties,
}

#[derive(Debug, Deserialize)]
pub struct ForecastProperties {
    pub periods: Vec<ForecastPeriod>,
}

#[derive(Debug, Deserialize)]
pub struct ForecastPeriod {
    pub name: String,
    pub temperature: i32,
    #[serde(rename = "temperatureUnit")]
    pub temperature_unit: String,
    #[serde(rename = "windSpeed")]
    pub wind_speed: String,
    #[serde(rename = "windDirection")]
    pub wind_direction: String,
    #[serde(rename = "shortForecast")]
    pub short_forecast: String,
    #[serde(rename = "detailedForecast")]
    pub detailed_forecast: String,
}

// ============================================================================
// MCP Tool Request Models
// ============================================================================

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetAlertsRequest {
    pub state: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetForecastRequest {
    pub latitude: f64,
    pub longitude: f64,
}
