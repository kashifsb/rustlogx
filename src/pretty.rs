//! Pretty (human-readable, colored) log formatting.

use crate::config::Level;
use crate::redact::{is_sensitive_key, redact_value};

// ANSI color codes
const RESET: &str = "\x1b[0m";
const GRAY: &str = "\x1b[90m";
const PURPLE: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const BOLD: &str = "\x1b[1m";

/// Formats a log level as a colored 3-character badge.
pub fn format_level(level: Level, no_color: bool) -> String {
    let badge = level.badge();
    let color = match level {
        Level::Trace => GRAY,
        Level::Debug => PURPLE,
        Level::Info => CYAN,
        Level::Warn => YELLOW,
        Level::Error => RED,
        Level::Fatal => &format!("{RED}{BOLD}"),
    };
    colorize(color, badge, no_color)
}

/// Formats a timestamp string with gray color.
pub fn format_timestamp(ts: &str, no_color: bool) -> String {
    colorize(GRAY, ts, no_color)
}

/// Formats a caller string in gray parentheses.
pub fn format_caller(caller: &str, no_color: bool) -> String {
    colorize(GRAY, &format!("({caller})"), no_color)
}

/// Formats a field key=value pair with appropriate colors.
pub fn format_field(key: &str, value: &str, no_color: bool) -> String {
    let val = if is_sensitive_key(key) {
        redact_value(value)
    } else {
        value.to_string()
    };
    format!(
        "{}{}",
        colorize(GRAY, &format!("{key}="), no_color),
        colorize(GREEN, &val, no_color),
    )
}

/// Formats an error field with red value.
pub fn format_error_field(value: &str, no_color: bool) -> String {
    format!(
        "{}{}",
        colorize(GRAY, "error=", no_color),
        colorize(RED, value, no_color),
    )
}

/// Wraps text with ANSI color codes, respecting no_color flag.
pub fn colorize(code: &str, text: &str, no_color: bool) -> String {
    if no_color {
        text.to_string()
    } else {
        format!("{code}{text}{RESET}")
    }
}

/// Trims a caller path to the last 2 segments.
pub fn short_caller(file: &str, line: u32) -> String {
    let parts: Vec<&str> = file.rsplit('/').take(2).collect();
    let short = if parts.len() >= 2 {
        format!("{}/{}", parts[1], parts[0])
    } else {
        file.to_string()
    };
    format!("{short}:{line}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_level_no_color() {
        assert_eq!(format_level(Level::Info, true), "INF");
        assert_eq!(format_level(Level::Warn, true), "WAR");
        assert_eq!(format_level(Level::Error, true), "ERR");
        assert_eq!(format_level(Level::Fatal, true), "FTL");
    }

    #[test]
    fn test_format_level_with_color() {
        let result = format_level(Level::Info, false);
        assert!(result.contains("\x1b["));
        assert!(result.contains("INF"));
    }

    #[test]
    fn test_short_caller() {
        assert_eq!(short_caller("server/main.rs", 42), "server/main.rs:42");
        assert_eq!(
            short_caller("/home/user/project/server/main.rs", 42),
            "server/main.rs:42"
        );
    }

    #[test]
    fn test_format_field_redaction() {
        let result = format_field("api_key", "supersecret", true);
        assert!(result.contains("api_key="));
        assert!(!result.contains("supersecret"));
    }
}
