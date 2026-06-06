//! Logger configuration.

use std::env;
use std::io::IsTerminal;

/// Log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl Level {
    /// Returns the 3-character badge for this level.
    pub fn badge(&self) -> &'static str {
        match self {
            Level::Trace => "TRC",
            Level::Debug => "DBG",
            Level::Info => "INF",
            Level::Warn => "WAR",
            Level::Error => "ERR",
            Level::Fatal => "FTL",
        }
    }

    /// Returns the lowercase name for JSON output.
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Trace => "trace",
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Error => "error",
            Level::Fatal => "fatal",
        }
    }

    /// Parses a level string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Level> {
        match s.to_lowercase().as_str() {
            "trace" => Some(Level::Trace),
            "debug" => Some(Level::Debug),
            "info" => Some(Level::Info),
            "warn" | "warning" => Some(Level::Warn),
            "error" => Some(Level::Error),
            "fatal" | "critical" => Some(Level::Fatal),
            _ => None,
        }
    }
}

/// Output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Auto,
    Pretty,
    Json,
}

/// Logger configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub level: Level,
    pub format: Format,
    pub caller: bool,
    pub no_color: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            level: Level::Info,
            format: Format::Auto,
            caller: false,
            no_color: false,
        }
    }
}

impl Config {
    /// Creates a Config from environment variables.
    pub fn from_env() -> Self {
        let mut cfg = Config::default();

        if let Ok(lvl) = env::var("LOG_LEVEL") {
            if let Some(level) = Level::from_str(&lvl) {
                cfg.level = level;
            }
        }

        if let Ok(fmt) = env::var("LOG_FORMAT") {
            cfg.format = match fmt.to_lowercase().as_str() {
                "pretty" => Format::Pretty,
                "json" => Format::Json,
                _ => Format::Auto,
            };
        }

        if let Ok(caller) = env::var("LOG_CALLER") {
            cfg.caller = caller == "true" || caller == "1";
        }

        if env::var("NO_COLOR").is_ok() || env::var("TERM").ok().as_deref() == Some("dumb") {
            cfg.no_color = true;
        }

        cfg
    }

    /// Resolves Auto format to Pretty or Json.
    pub fn resolved_format(&self) -> Format {
        match self.format {
            Format::Auto => {
                if std::io::stderr().is_terminal() {
                    Format::Pretty
                } else {
                    Format::Json
                }
            }
            other => other,
        }
    }
}
