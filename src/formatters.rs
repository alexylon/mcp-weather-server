use crate::models::{AlertResponse, ForecastResponse, OpenMeteoResponse};

/// Formats weather alerts into a human-readable string
pub fn format_alerts(alerts: AlertResponse) -> String {
    if alerts.features.is_empty() {
        return "No active weather alerts.".to_string();
    }

    let mut output = String::from("Active Weather Alerts:\n\n");
    for (i, feature) in alerts.features.iter().enumerate() {
        let props = &feature.properties;
        output.push_str(&format!(
            "Alert {}:\n  Event: {}\n  Severity: {}\n  Area: {}\n",
            i + 1,
            props.event,
            props.severity,
            props.area_desc
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

/// Formats NWS forecast into a human-readable string
pub fn format_forecast(forecast: ForecastResponse) -> String {
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

/// Formats Open-Meteo forecast into a human-readable string
pub fn format_open_meteo_forecast(forecast: OpenMeteoResponse) -> String {
    let mut output = format!(
        "Weather Forecast (Open-Meteo)\nLocation: {:.4}, {:.4}\nTimezone: {}\n\n",
        forecast.latitude, forecast.longitude, forecast.timezone
    );

    for i in 0..forecast.daily.time.len().min(7) {
        let weather_desc = weather_code_to_description(forecast.daily.weather_code[i]);
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

/// Converts WMO weather code to human-readable description
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
