pub fn bytes_to_human_format(bytes: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let (value, unit) = if bytes < (KB - 25f64) as i64 {
        (bytes as f64, "B")
    } else if bytes < (MB - 25f64) as i64 {
        (bytes as f64 / KB, "KB")
    } else if bytes < (GB - 25f64) as i64 {
        (bytes as f64 / MB, "MB")
    } else if bytes < (TB - 25f64) as i64 {
        (bytes as f64 / GB, "GB")
    } else {
        (bytes as f64 / TB, "TB")
    };

    format!("{value:.1} {unit}")
}

pub fn seconds_to_human_format(seconds: i64) -> String {
    const MINUTE: i64 = 60;
    const HOUR: i64 = MINUTE * 60;
    const DAY: i64 = HOUR * 24;

    if seconds == 0 {
        return "0s".to_string();
    }

    let mut curr_string = String::new();

    let mut rest = seconds;
    if seconds > DAY {
        let days = rest / DAY;
        rest %= DAY;

        curr_string = format!("{curr_string}{days}d");
    }

    if seconds > HOUR {
        let hours = rest / HOUR;
        rest %= HOUR;
        curr_string = format!("{curr_string}{hours}h");
        // skip minutes & seconds for multi-day durations
        if seconds > DAY {
            return curr_string;
        }
    }

    if seconds > MINUTE {
        let minutes = rest / MINUTE;
        rest %= MINUTE;
        curr_string = format!("{curr_string}{minutes}m");
    }

    curr_string = format!("{curr_string}{rest}s");
    curr_string
}
