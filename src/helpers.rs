//! Convenience logging helpers.

use serde_json::json;

use crate::config::Level;
use crate::logger::Logger;

/// Logs a success message at Info level.
pub fn log_success(logger: &Logger, msg: &str) {
    logger.info(msg, &[("result", json!("success"))]);
}

/// Logs an incoming HTTP or gRPC request.
pub fn log_request(logger: &Logger, method: &str, path: &str, user: &str) {
    logger.info(
        "Request received",
        &[
            ("method", json!(method)),
            ("path", json!(path)),
            ("user", json!(user)),
        ],
    );
}

/// Logs an outgoing response, choosing the level based on status code.
pub fn log_response(
    logger: &Logger,
    method: &str,
    path: &str,
    status: u16,
    duration_ms: f64,
    error: Option<&str>,
) {
    let mut fields = vec![
        ("method", json!(method)),
        ("path", json!(path)),
        ("status", json!(status)),
        ("duration_ms", json!(duration_ms)),
    ];

    if let Some(err) = error {
        fields.push(("error", json!(err)));
    }

    let level = match status {
        500..=599 => Level::Error,
        400..=499 => Level::Warn,
        _ => Level::Info,
    };

    logger.log(level, "Response sent", &fields);
}

/// Logs a database query at Debug level.
pub fn log_db_query(
    logger: &Logger,
    operation: &str,
    table: &str,
    duration_ms: f64,
    rows_affected: i64,
) {
    logger.log(
        Level::Debug,
        "Database query executed",
        &[
            ("operation", json!(operation)),
            ("table", json!(table)),
            ("duration_ms", json!(duration_ms)),
            ("rows_affected", json!(rows_affected)),
        ],
    );
}

/// Logs a service-level error with structured context fields.
pub fn log_service_error(
    logger: &Logger,
    service: &str,
    operation: &str,
    error: &str,
    fields: &[(&str, serde_json::Value)],
) {
    let mut all_fields = vec![
        ("error", json!(error)),
        ("service", json!(service)),
        ("operation", json!(operation)),
    ];
    all_fields.extend_from_slice(fields);
    logger.error("Service operation failed", &all_fields);
}

/// Logs service-level debug information with structured context fields.
pub fn log_service_debug(
    logger: &Logger,
    service: &str,
    operation: &str,
    message: &str,
    fields: &[(&str, serde_json::Value)],
) {
    let mut all_fields = vec![
        ("service", json!(service)),
        ("operation", json!(operation)),
    ];
    all_fields.extend_from_slice(fields);
    logger.log(Level::Debug, message, &all_fields);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Format};
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct SharedBuf(Arc<Mutex<Vec<u8>>>);

    impl Write for SharedBuf {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn test_logger() -> (Logger, Arc<Mutex<Vec<u8>>>) {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer = SharedBuf(buf.clone());
        let config = Config {
            level: Level::Trace,
            format: Format::Json,
            caller: false,
            no_color: true,
        };
        (Logger::with_writer(config, Box::new(writer)), buf)
    }

    fn last_json(buf: &Arc<Mutex<Vec<u8>>>) -> serde_json::Value {
        let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
        let last = output.trim().lines().last().unwrap();
        serde_json::from_str(last).unwrap()
    }

    #[test]
    fn test_log_success() {
        let (l, buf) = test_logger();
        log_success(&l, "done");
        let data = last_json(&buf);
        assert_eq!(data["result"], "success");
    }

    #[test]
    fn test_log_request() {
        let (l, buf) = test_logger();
        log_request(&l, "GET", "/api/users", "baleeghu");
        let data = last_json(&buf);
        assert_eq!(data["method"], "GET");
        assert_eq!(data["path"], "/api/users");
        assert_eq!(data["user"], "baleeghu");
        assert_eq!(data["message"], "Request received");
    }

    #[test]
    fn test_log_response_levels() {
        for (status, expected) in [(200u16, "info"), (404, "warn"), (500, "error")] {
            let (l, buf) = test_logger();
            log_response(&l, "GET", "/", status, 10.0, None);
            let data = last_json(&buf);
            assert_eq!(data["level"], expected, "status {status}");
            assert_eq!(data["message"], "Response sent");
        }
    }

    #[test]
    fn test_log_response_with_error() {
        let (l, buf) = test_logger();
        log_response(&l, "GET", "/api", 500, 150.0, Some("db timeout"));
        let data = last_json(&buf);
        assert_eq!(data["level"], "error");
        assert_eq!(data["error"], "db timeout");
    }

    #[test]
    fn test_log_db_query() {
        let (l, buf) = test_logger();
        log_db_query(&l, "SELECT", "users", 5.0, 42);
        let data = last_json(&buf);
        assert_eq!(data["operation"], "SELECT");
        assert_eq!(data["table"], "users");
        assert_eq!(data["rows_affected"], 42);
        assert_eq!(data["level"], "debug");
        assert_eq!(data["message"], "Database query executed");
    }

    #[test]
    fn test_log_service_error() {
        let (l, buf) = test_logger();
        log_service_error(&l, "UserService", "CreateUser", "duplicate email", &[
            ("email", json!("user@example.com")),
        ]);
        let data = last_json(&buf);
        assert_eq!(data["service"], "UserService");
        assert_eq!(data["operation"], "CreateUser");
        assert_eq!(data["level"], "error");
        assert_eq!(data["message"], "Service operation failed");
    }

    #[test]
    fn test_log_service_debug() {
        let (l, buf) = test_logger();
        log_service_debug(&l, "CacheService", "Get", "Cache lookup", &[
            ("hit", json!(true)),
        ]);
        let data = last_json(&buf);
        assert_eq!(data["service"], "CacheService");
        assert_eq!(data["operation"], "Get");
        assert_eq!(data["level"], "debug");
    }
}
