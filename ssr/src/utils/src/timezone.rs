use chrono::{DateTime, Datelike};

/// Get the browser's timezone using JavaScript's Intl API
/// Returns IANA timezone identifier like "America/New_York"
pub fn get_browser_timezone() -> String {
    #[cfg(feature = "hydrate")]
    {
        use js_sys::Reflect;
        use wasm_bindgen::{JsCast, JsValue};

        // Try to get timezone from Intl.DateTimeFormat().resolvedOptions().timeZone
        if let Ok(global) = js_sys::global().dyn_into::<web_sys::Window>() {
            // Access Intl object
            if let Ok(intl_value) = Reflect::get(&global, &JsValue::from_str("Intl")) {
                // Create new DateTimeFormat instance
                if let Ok(date_time_format_constructor) =
                    Reflect::get(&intl_value, &JsValue::from_str("DateTimeFormat"))
                {
                    // Call constructor with no arguments (uses default locale and options)
                    if let Ok(date_time_format_instance) = Reflect::construct(
                        &date_time_format_constructor.into(),
                        &js_sys::Array::new(),
                    ) {
                        // Call resolvedOptions() method
                        if let Ok(resolved_options_fn) = Reflect::get(
                            &date_time_format_instance,
                            &JsValue::from_str("resolvedOptions"),
                        ) {
                            // Call the function
                            if let Ok(resolved_options) = Reflect::apply(
                                &resolved_options_fn.into(),
                                &date_time_format_instance,
                                &js_sys::Array::new(),
                            ) {
                                // Get timeZone property
                                if let Ok(timezone_value) =
                                    Reflect::get(&resolved_options, &JsValue::from_str("timeZone"))
                                {
                                    if let Some(timezone_str) = timezone_value.as_string() {
                                        return timezone_str;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Default fallback
    "UTC".to_string()
}

/// Format a tournament date with proper timezone handling
/// 
/// # Arguments
/// * `timestamp` - Unix timestamp in seconds
/// * `client_time` - Optional ISO 8601 formatted time string from server
/// * `client_timezone` - Optional timezone string from server
/// * `use_browser_timezone` - Whether to use browser timezone as fallback
/// 
/// # Returns
/// Formatted date string like "21st January, 09:30 PM EST"
pub fn format_tournament_date(
    timestamp: i64,
    client_time: Option<&String>,
    client_timezone: Option<&String>,
    use_browser_timezone: bool,
) -> String {
    // Helper function for day suffix
    fn get_day_suffix(day: u32) -> &'static str {
        match day {
            1 | 21 | 31 => "st",
            2 | 22 => "nd",
            3 | 23 => "rd",
            _ => "th",
        }
    }

    // First, try to use client_time if provided
    if let Some(client_time_str) = client_time {
        if let Ok(dt) = DateTime::parse_from_rfc3339(client_time_str) {
            // Extract timezone abbreviation if available
            let tz_str = client_timezone
                .and_then(|tz| tz.split('/').last())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    if use_browser_timezone {
                        // Get browser timezone and extract the last part
                        let browser_tz = get_browser_timezone();
                        // Return owned String instead of reference
                        browser_tz.split('/').last().unwrap_or("Local Time").to_string()
                    } else {
                        "Local Time".to_string()
                    }
                });

            let day = dt.day();
            let suffix = get_day_suffix(day);

            return format!(
                "{}{} {}, {} {}",
                day,
                suffix,
                dt.format("%B"),
                dt.format("%I:%M %p"),
                tz_str
            );
        }
    }

    // Fallback to timestamp formatting
    if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
        let day = dt.day();
        let suffix = get_day_suffix(day);

        // Determine timezone label
        let tz_label = if use_browser_timezone {
            #[cfg(feature = "hydrate")]
            {
                let browser_tz = get_browser_timezone();
                if browser_tz != "UTC" {
                    // Return owned String
                    browser_tz.split('/').last().unwrap_or("Local Time").to_string()
                } else {
                    "UTC".to_string()
                }
            }
            #[cfg(not(feature = "hydrate"))]
            {
                "UTC".to_string()
            }
        } else {
            "UTC".to_string()
        };

        format!(
            "{}{} {}, {} {}",
            day,
            suffix,
            dt.format("%B"),
            dt.format("%I:%M %p"),
            tz_label
        )
    } else {
        "Unknown".to_string()
    }
}

/// Format a tournament date for display in components
/// This is a convenience wrapper that automatically uses browser timezone as fallback
pub fn format_tournament_date_with_fallback(
    timestamp: i64,
    client_time: Option<&String>,
    client_timezone: Option<&String>,
) -> String {
    format_tournament_date(timestamp, client_time, client_timezone, true)
}