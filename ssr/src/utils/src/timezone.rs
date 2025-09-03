#[cfg(not(feature = "hydrate"))]
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
                                        web_sys::console::log_1(&JsValue::from_str(&format!(
                                            "Detected timezone: {timezone_str}"
                                        )));
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

/// Get timezone abbreviation (like IST, PST, EST) for a given IANA timezone
/// If no timezone is provided, uses the browser's default timezone
pub fn get_timezone_abbreviation(timezone: Option<&str>) -> String {
    #[cfg(feature = "hydrate")]
    {
        use js_sys::{Date, Object, Reflect};
        use wasm_bindgen::{JsCast, JsValue};

        // Create date object
        let date = Date::new_0();

        // Create options object
        let options = Object::new();

        // Set timeZoneName to 'short' to get abbreviation
        if Reflect::set(
            &options,
            &JsValue::from_str("timeZoneName"),
            &JsValue::from_str("short"),
        )
        .is_err()
        {
            return "UTC".to_string();
        }

        // Set specific timezone if provided
        if let Some(tz) = timezone {
            if Reflect::set(
                &options,
                &JsValue::from_str("timeZone"),
                &JsValue::from_str(tz),
            )
            .is_err()
            {
                return "UTC".to_string();
            }
        }

        // Get Intl.DateTimeFormat
        if let Ok(global) = js_sys::global().dyn_into::<web_sys::Window>() {
            if let Ok(intl_value) = Reflect::get(&global, &JsValue::from_str("Intl")) {
                if let Ok(date_time_format_constructor) =
                    Reflect::get(&intl_value, &JsValue::from_str("DateTimeFormat"))
                {
                    // Create DateTimeFormat instance with options
                    let args = js_sys::Array::new();
                    args.push(&JsValue::from_str("en-US"));
                    args.push(&options);

                    if let Ok(formatter) =
                        Reflect::construct(&date_time_format_constructor.into(), &args)
                    {
                        // Call formatToParts method
                        if let Ok(format_to_parts) =
                            Reflect::get(&formatter, &JsValue::from_str("formatToParts"))
                        {
                            let format_args = js_sys::Array::new();
                            format_args.push(&date);

                            if let Ok(parts) =
                                Reflect::apply(&format_to_parts.into(), &formatter, &format_args)
                            {
                                // Convert to Array and find timeZoneName part
                                if let Ok(parts_array) = parts.dyn_into::<js_sys::Array>() {
                                    for i in 0..parts_array.length() {
                                        let part = parts_array.get(i);
                                        if !part.is_undefined() && !part.is_null() {
                                            if let Ok(part_type) =
                                                Reflect::get(&part, &JsValue::from_str("type"))
                                            {
                                                if let Some(type_str) = part_type.as_string() {
                                                    if type_str == "timeZoneName" {
                                                        if let Ok(value) = Reflect::get(
                                                            &part,
                                                            &JsValue::from_str("value"),
                                                        ) {
                                                            if let Some(tz_abbr) = value.as_string()
                                                            {
                                                                let log_msg = if let Some(tz) =
                                                                    timezone
                                                                {
                                                                    format!("Detected timezone abbreviation for {tz}: {tz_abbr}")
                                                                } else {
                                                                    format!("Detected timezone abbreviation (browser default): {tz_abbr}")
                                                                };
                                                                web_sys::console::log_1(
                                                                    &JsValue::from_str(&log_msg),
                                                                );
                                                                return tz_abbr;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback for server-side or if JavaScript fails
    #[cfg(not(feature = "hydrate"))]
    {
        _ = timezone;
    }

    // Default fallback
    "UTC".to_string()
}

/// Format a tournament date with proper timezone handling
///
/// # Arguments
/// * `timestamp` - Unix timestamp in seconds (UTC)
/// * `client_time` - Optional ISO 8601 formatted time string from server
/// * `client_timezone` - Optional timezone string from server
///
/// # Returns
/// Formatted date string like "21st January, 09:30 PM" in local browser time
pub fn format_tournament_date(
    timestamp: i64,
    _client_time: Option<&String>,
    _client_timezone: Option<&String>,
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

    // Convert UTC timestamp to local time in browser
    #[cfg(feature = "hydrate")]
    {
        use js_sys::Date;

        // Create a Date object from the timestamp (multiply by 1000 for milliseconds)
        let date = Date::new(&wasm_bindgen::JsValue::from_f64((timestamp * 1000) as f64));

        // Get local date components
        let day = date.get_date();
        let month = date.get_month(); // 0-indexed
        let hours = date.get_hours();
        let minutes = date.get_minutes();

        // Format month name
        let month_names = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];
        let month_name = month_names.get(month as usize).unwrap_or(&"Unknown");

        // Format time in 12-hour format
        let (hour_12, period) = if hours == 0 {
            (12, "AM")
        } else if hours < 12 {
            (hours, "AM")
        } else if hours == 12 {
            (12, "PM")
        } else {
            (hours - 12, "PM")
        };

        let suffix = get_day_suffix(day);

        format!("{day}{suffix} {month_name}, {hour_12:02}:{minutes:02} {period}")
    }

    // Server-side fallback: just format the UTC time
    #[cfg(not(feature = "hydrate"))]
    {
        if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
            let day = dt.day();
            let suffix = get_day_suffix(day);

            format!(
                "{}{} {}, {}",
                day,
                suffix,
                dt.format("%B"),
                dt.format("%I:%M %p")
            )
        } else {
            "Unknown".to_string()
        }
    }
}

/// Format a tournament date for display in components
/// This is a convenience wrapper for consistent date formatting
pub fn format_tournament_date_with_fallback(
    timestamp: i64,
    client_time: Option<&String>,
    client_timezone: Option<&String>,
) -> String {
    format_tournament_date(timestamp, client_time, client_timezone)
}
