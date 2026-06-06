//! rustlog — Universal structured logging library for Rust.
//!
//! Provides leveled, structured logging with pretty (colored) and JSON output modes,
//! field redaction, sub-loggers, and convenience helpers.

pub mod config;
pub mod helpers;
pub mod logger;
pub mod macros;
pub mod pretty;
pub mod redact;

// Re-exports for convenience
pub use config::{Config, Format, Level};
pub use helpers::{log_db_query, log_request, log_response, log_service_debug, log_service_error, log_success};
pub use logger::Logger;
pub use redact::{is_sensitive_key, redact_map, redact_value};
