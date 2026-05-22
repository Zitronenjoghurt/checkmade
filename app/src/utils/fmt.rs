pub fn fmt_duration(duration: web_time::Duration) -> String {
    let total_secs = duration.as_secs();
    let millis = duration.subsec_millis();

    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    match (days, hours, minutes, seconds, millis) {
        (0, 0, 0, 0, ms) => format!("{ms}ms"),
        (0, 0, 0, s, 0) => format!("{s}s"),
        (0, 0, 0, s, ms) => format!("{s}s {ms}ms"),
        (0, 0, m, s, _) => format!("{m}m {s}s"),
        (0, h, m, s, _) => format!("{h}h {m}m {s}s"),
        (d, h, m, _, _) => format!("{d}d {h}h {m}m"),
    }
}
