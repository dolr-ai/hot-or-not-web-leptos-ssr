use speedate::{DateTime, ParseError};
use uts2ts::uts2ts;
use web_time::Duration;

/// Get day & month -> DD MMM format
/// where DD -> 2 digits
/// MMM -> String representing the month. i.e AUG for august
pub fn get_day_month(epoch_secs: u64) -> String {
    let ts = uts2ts(epoch_secs as i64);
    let month = match ts.month {
        1 => "JAN",
        2 => "FEB",
        3 => "MARCH",
        4 => "APRIL",
        5 => "MAY",
        6 => "JUNE",
        7 => "JULY",
        8 => "AUG",
        9 => "SEPT",
        10 => "OCT",
        11 => "NOV",
        12 => "DEC",
        _ => unreachable!(),
    };
    format!("{:02} {month}", ts.day)
}

pub fn to_hh_mm_ss(duration: Duration) -> String {
    let secs = duration.as_secs();
    let hh = secs / 3600;
    let mm = (secs - hh * 3600) / 60;
    let ss = (secs - hh * 3600) - mm * 60;

    format!("{hh:02}:{mm:02}:{ss:02}")
}

pub async fn sleep(duration: Duration) {
    #[cfg(feature = "ssr")]
    {
        use tokio::time;
        time::sleep(duration).await;
    }
    #[cfg(feature = "hydrate")]
    {
        use gloo::timers::future::sleep;
        sleep(duration).await;
    }
}

pub fn parse_ns_to_datetime(timestamp: u64) -> Result<String, ParseError> {
    DateTime::from_timestamp(
        (timestamp / 1_000_000_000) as i64,           // seconds
        ((timestamp % 1_000_000_000) / 1_000) as u32, // microseconds
    )
    .map(|dt| {
        format!(
            "{} {}, {} {:02}:{:02} {}",
            match dt.date.month {
                1 => "January",
                2 => "February",
                3 => "March",
                4 => "April",
                5 => "May",
                6 => "June",
                7 => "July",
                8 => "August",
                9 => "September",
                10 => "October",
                11 => "November",
                12 => "December",
                _ => unimplemented!(),
            },
            dt.date.day,
            dt.date.year,
            if dt.time.hour > 12 {
                dt.time.hour - 12
            } else {
                dt.time.hour
            },
            dt.time.minute,
            if dt.time.hour >= 12 { "PM" } else { "AM" },
        )
    })
}
