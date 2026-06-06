//! Core logger implementation.

use std::collections::BTreeMap;
use std::io::Write;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use serde_json::{json, Value};

use crate::config::{Config, Format, Level};
use crate::pretty;

/// A thread-safe structured logger.
#[derive(Clone)]
pub struct Logger {
    config: Config,
    fields: BTreeMap<String, Value>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl Logger {
    /// Creates a new Logger with the given config, writing to stderr.
    pub fn new(config: Config) -> Self {
        Logger {
            config,
            fields: BTreeMap::new(),
            writer: Arc::new(Mutex::new(Box::new(std::io::stderr()))),
        }
    }

    /// Creates a new Logger writing to a custom writer (useful for testing).
    pub fn with_writer(config: Config, writer: Box<dyn Write + Send>) -> Self {
        Logger {
            config,
            fields: BTreeMap::new(),
            writer: Arc::new(Mutex::new(writer)),
        }
    }

    /// Logs a message at the given level with optional extra fields.
    pub fn log(&self, level: Level, msg: &str, fields: &[(&str, Value)]) {
        if level < self.config.level {
            return;
        }
        self.emit(level, msg, fields, None);
    }

    /// Logs a message with caller info.
    pub fn log_with_caller(
        &self,
        level: Level,
        msg: &str,
        fields: &[(&str, Value)],
        file: &str,
        line: u32,
    ) {
        if level < self.config.level {
            return;
        }
        self.emit(level, msg, fields, Some((file, line)));
    }

    fn emit(
        &self,
        level: Level,
        msg: &str,
        extra_fields: &[(&str, Value)],
        caller: Option<(&str, u32)>,
    ) {
        let now = Utc::now();
        let resolved = self.config.resolved_format();

        let output = match resolved {
            Format::Pretty => self.format_pretty(level, msg, extra_fields, caller, &now),
            _ => self.format_json(level, msg, extra_fields, caller, &now),
        };

        if let Ok(mut w) = self.writer.lock() {
            let _ = writeln!(w, "{output}");
        }
    }

    fn format_json(
        &self,
        level: Level,
        msg: &str,
        extra_fields: &[(&str, Value)],
        caller: Option<(&str, u32)>,
        now: &chrono::DateTime<Utc>,
    ) -> String {
        let mut obj = serde_json::Map::new();
        obj.insert("level".to_string(), json!(level.as_str()));
        obj.insert(
            "time".to_string(),
            json!(now.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        );
        obj.insert("message".to_string(), json!(msg));

        // Persistent fields
        for (k, v) in &self.fields {
            obj.insert(k.clone(), v.clone());
        }

        // Extra fields
        for (k, v) in extra_fields {
            obj.insert(k.to_string(), v.clone());
        }

        // Caller
        let caller_info = if self.config.caller {
            caller
        } else {
            None
        };
        if let Some((file, line)) = caller_info {
            obj.insert("caller".to_string(), json!(format!("{file}:{line}")));
        }

        serde_json::to_string(&Value::Object(obj)).unwrap_or_default()
    }

    fn format_pretty(
        &self,
        level: Level,
        msg: &str,
        extra_fields: &[(&str, Value)],
        caller: Option<(&str, u32)>,
        now: &chrono::DateTime<Utc>,
    ) -> String {
        let nc = self.config.no_color;
        let mut parts: Vec<String> = Vec::new();

        // Timestamp
        let ts = now.format("%Y/%m/%d %H:%M:%S UTC").to_string();
        parts.push(pretty::format_timestamp(&ts, nc));

        // Level badge
        parts.push(pretty::format_level(level, nc));

        // Message
        parts.push(msg.to_string());

        // Error field first
        let mut all_fields: BTreeMap<String, Value> = self.fields.clone();
        for (k, v) in extra_fields {
            all_fields.insert(k.to_string(), v.clone());
        }

        if let Some(err) = all_fields.remove("error") {
            let err_str = match &err {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            parts.push(pretty::format_error_field(&err_str, nc));
        }

        // Remaining fields sorted
        for (k, v) in &all_fields {
            let val = match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            parts.push(pretty::format_field(k, &val, nc));
        }

        // Caller
        let caller_info = if self.config.caller {
            caller
        } else {
            None
        };
        if let Some((file, line)) = caller_info {
            let short = pretty::short_caller(file, line);
            parts.push(pretty::format_caller(&short, nc));
        }

        parts.join(" ")
    }

    // --- Sub-loggers ---

    /// Returns a sub-logger with a "component" field.
    pub fn with_component(&self, name: &str) -> Logger {
        self.with_field("component", json!(name))
    }

    /// Returns a sub-logger with a "request_id" field.
    pub fn with_request_id(&self, id: &str) -> Logger {
        self.with_field("request_id", json!(id))
    }

    /// Returns a sub-logger with a "trace_id" field.
    pub fn with_trace_id(&self, id: &str) -> Logger {
        self.with_field("trace_id", json!(id))
    }

    /// Returns a sub-logger with an additional field.
    pub fn with_field(&self, key: &str, value: Value) -> Logger {
        let mut new_fields = self.fields.clone();
        new_fields.insert(key.to_string(), value);
        Logger {
            config: self.config.clone(),
            fields: new_fields,
            writer: self.writer.clone(),
        }
    }

    // --- Convenience methods ---

    pub fn trace(&self, msg: &str, fields: &[(&str, Value)]) {
        self.log(Level::Trace, msg, fields);
    }

    pub fn debug(&self, msg: &str, fields: &[(&str, Value)]) {
        self.log(Level::Debug, msg, fields);
    }

    pub fn info(&self, msg: &str, fields: &[(&str, Value)]) {
        self.log(Level::Info, msg, fields);
    }

    pub fn warn(&self, msg: &str, fields: &[(&str, Value)]) {
        self.log(Level::Warn, msg, fields);
    }

    pub fn error(&self, msg: &str, fields: &[(&str, Value)]) {
        self.log(Level::Error, msg, fields);
    }

    pub fn fatal(&self, msg: &str, fields: &[(&str, Value)]) {
        self.log(Level::Fatal, msg, fields);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn test_logger(level: Level, format: Format) -> (Logger, Arc<Mutex<Vec<u8>>>) {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer = SharedBuf(buf.clone());
        let config = Config {
            level,
            format,
            caller: false,
            no_color: true,
        };
        let logger = Logger::with_writer(config, Box::new(writer));
        (logger, buf)
    }

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

    fn get_output(buf: &Arc<Mutex<Vec<u8>>>) -> String {
        String::from_utf8(buf.lock().unwrap().clone()).unwrap()
    }

    fn parse_json(buf: &Arc<Mutex<Vec<u8>>>) -> Value {
        let output = get_output(buf);
        let last_line = output.trim().lines().last().unwrap();
        serde_json::from_str(last_line).unwrap()
    }

    #[test]
    fn test_json_output() {
        let (l, buf) = test_logger(Level::Info, Format::Json);
        l.info("hello", &[("key", json!("value"))]);

        let data = parse_json(&buf);
        assert_eq!(data["message"], "hello");
        assert_eq!(data["key"], "value");
        assert_eq!(data["level"], "info");
    }

    #[test]
    fn test_level_filtering() {
        let (l, buf) = test_logger(Level::Warn, Format::Json);
        l.info("should be filtered", &[]);
        assert!(get_output(&buf).is_empty());

        l.warn("should appear", &[]);
        assert!(!get_output(&buf).is_empty());
    }

    #[test]
    fn test_pretty_output() {
        let (l, buf) = test_logger(Level::Info, Format::Pretty);
        l.info("Server started", &[("port", json!(8080))]);

        let output = get_output(&buf);
        assert!(output.contains("INF"));
        assert!(output.contains("Server started"));
        assert!(output.contains("port=8080"));
    }

    #[test]
    fn test_pretty_no_ansi() {
        let (l, buf) = test_logger(Level::Info, Format::Pretty);
        l.info("test", &[]);
        let output = get_output(&buf);
        assert!(!output.contains("\x1b["));
    }

    #[test]
    fn test_sub_loggers() {
        let (l, buf) = test_logger(Level::Info, Format::Json);
        let sub = l.with_component("auth").with_request_id("req-1");
        sub.info("test", &[]);

        let data = parse_json(&buf);
        assert_eq!(data["component"], "auth");
        assert_eq!(data["request_id"], "req-1");
    }

    #[test]
    fn test_sub_logger_persistence() {
        let (l, buf) = test_logger(Level::Info, Format::Json);
        let sub = l.with_component("db");

        sub.info("first", &[]);
        let d1 = parse_json(&buf);
        assert_eq!(d1["component"], "db");

        sub.warn("second", &[]);
        let output = get_output(&buf);
        let lines: Vec<&str> = output.trim().lines().collect();
        let d2: Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(d2["component"], "db");
    }

    #[test]
    fn test_caller_json() {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let writer = SharedBuf(buf.clone());
        let config = Config {
            level: Level::Info,
            format: Format::Json,
            caller: true,
            no_color: true,
        };
        let l = Logger::with_writer(config, Box::new(writer));
        l.log_with_caller(Level::Info, "test", &[], file!(), line!());

        let data = parse_json(&buf);
        assert!(data["caller"].as_str().unwrap().contains("logger.rs"));
    }
}
