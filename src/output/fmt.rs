/// Format an ISO 8601 timestamp as a relative time string like "2m ago", "3h ago", "5d ago".
/// Falls back to the raw timestamp if parsing fails.
pub fn relative_time(iso: &str) -> String {
    let Ok(then) = chrono::DateTime::parse_from_rfc3339(iso) else {
        return iso.to_string();
    };
    let now = chrono::Utc::now();
    let delta = now.signed_duration_since(then);

    if delta.num_seconds() < 0 {
        // Future time
        let abs = -delta.num_seconds();
        return format_duration_ago(abs, true);
    }

    format_duration_ago(delta.num_seconds(), false)
}

fn format_duration_ago(secs: i64, future: bool) -> String {
    let suffix = if future { "from now" } else { "ago" };
    if secs < 60 {
        format!("{secs}s {suffix}")
    } else if secs < 3600 {
        format!("{}m {suffix}", secs / 60)
    } else if secs < 86400 {
        format!("{}h {suffix}", secs / 3600)
    } else {
        format!("{}d {suffix}", secs / 86400)
    }
}

/// Format seconds into a human-friendly duration: "30s", "5m", "2h", "1d 12h".
pub fn duration(seconds: u64) -> String {
    if seconds == 0 {
        return "0s".to_string();
    }
    let d = seconds / 86400;
    let h = (seconds % 86400) / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;

    let mut parts = Vec::new();
    if d > 0 {
        parts.push(format!("{d}d"));
    }
    if h > 0 {
        parts.push(format!("{h}h"));
    }
    if m > 0 {
        parts.push(format!("{m}m"));
    }
    if s > 0 && d == 0 {
        parts.push(format!("{s}s"));
    }
    parts.join(" ")
}
