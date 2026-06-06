//! UTC timestamp formatting without chrono.
//! Uses SystemTime + epoch-to-calendar arithmetic (Hinnant algorithm).

use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current UTC time formatted for pretty mode: "YYYY/MM/DD HH:MM:SS UTC"
pub fn utc_now_pretty() -> String {
    let (y, m, d, h, min, s) = utc_parts();
    format!("{y:04}/{m:02}/{d:02} {h:02}:{min:02}:{s:02} UTC")
}

/// Returns the current UTC time formatted for JSON mode: "YYYY-MM-DDTHH:MM:SSZ"
pub fn utc_now_iso() -> String {
    let (y, m, d, h, min, s) = utc_parts();
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{min:02}:{s:02}Z")
}

fn utc_parts() -> (i64, u32, u32, u32, u32, u32) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = (secs / 86400) as i64;
    let day_secs = (secs % 86400) as u32;
    let (y, m, d) = civil_from_days(days);
    let h = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let s = day_secs % 60;
    (y, m, d, h, min, s)
}

/// Converts days since Unix epoch (1970-01-01) to (year, month, day).
/// Uses Howard Hinnant's algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // year of era [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_civil_from_days_epoch() {
        // 1970-01-01
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn test_civil_from_days_known_dates() {
        // 2024-02-29 (leap year)
        assert_eq!(civil_from_days(19782), (2024, 2, 29));
        // 2000-01-01
        assert_eq!(civil_from_days(10957), (2000, 1, 1));
    }

    #[test]
    fn test_utc_now_pretty_format() {
        let ts = utc_now_pretty();
        assert!(ts.ends_with(" UTC"), "expected UTC suffix: {ts}");
        assert_eq!(ts.len(), 23, "expected length 23: {ts}");
    }

    #[test]
    fn test_utc_now_iso_format() {
        let ts = utc_now_iso();
        assert!(ts.ends_with('Z'), "expected Z suffix: {ts}");
        assert!(ts.contains('T'), "expected T separator: {ts}");
        assert_eq!(ts.len(), 20, "expected length 20: {ts}");
    }
}
