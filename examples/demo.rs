//! rustlogx — Universal Structured Logger Demo
//!
//! Run with: `cargo run --example demo`

use serde_json::json;

use rustlogx::{
    log_db_query, log_request, log_response, log_service_debug, log_service_error, log_success,
    Config, Format, Level, Logger,
};

fn main() {
    // ──────────────────────────────────────────
    // 1. Initialize the logger
    // ──────────────────────────────────────────
    let pretty = Logger::with_writer(
        Config {
            level: Level::Trace,
            format: Format::Pretty,
            caller: true,
            no_color: false,
        },
        Box::new(std::io::stdout()),
    );

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║             rustlogx Logger — Format Showcase                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // ──────────────────────────────────────────
    // 2. Basic log levels
    // ──────────────────────────────────────────
    print_section("Basic Log Levels");

    rustlogx::log_trace!(pretty, "This is a TRACE message — finest granularity");
    rustlogx::log_debug!(pretty, "This is a DEBUG message — diagnostic detail");
    rustlogx::log_info!(pretty, "This is an INFO message — normal operation");
    rustlogx::log_warn!(pretty, "This is a WARN message — something looks off");
    rustlogx::log_error!(pretty, "This is an ERROR message — something failed");

    // ──────────────────────────────────────────
    // 3. Structured fields
    // ──────────────────────────────────────────
    print_section("Structured Fields");

    rustlogx::log_info!(pretty, "User authentication successful",
        "user" => "baleeghu",
        "action" => "login",
        "ip" => "192.168.1.42",
        "attempt" => 1,
        "mfa" => true,
    );

    rustlogx::log_debug!(pretty, "Query plan analyzed",
        "query" => "SELECT * FROM users WHERE active = true",
        "params" => 0,
        "cost_estimate" => 0.0034,
    );

    // ──────────────────────────────────────────
    // 4. Error logging with context
    // ──────────────────────────────────────────
    print_section("Error Logging");

    rustlogx::log_error!(pretty, "Failed to connect to database",
        "error" => "connection refused: dial tcp 10.0.0.5:5432",
        "host" => "10.0.0.5",
        "port" => 5432,
        "database" => "myapp_prod",
    );

    rustlogx::log_error!(pretty, "Service operation failed",
        "error" => "user service: repository: connection refused",
        "operation" => "GetUserByID",
        "user_id" => "usr_abc123",
    );

    // ──────────────────────────────────────────
    // 5. Sub-loggers with context
    // ──────────────────────────────────────────
    print_section("Sub-loggers (Component / Request Scoped)");

    let db_logger = pretty.with_component("database");
    db_logger.info("Connection pool initialized", &[("driver", json!("pgx")), ("host", json!("localhost:5432"))]);
    db_logger.debug("Pool stats", &[("pool_size", json!(25)), ("idle", json!(10))]);

    let auth_logger = pretty.with_component("auth");
    auth_logger.info("Auth provider configured", &[("provider", json!("kerberos"))]);

    let req_logger = pretty.with_request_id("req-7f3a-4b2c-9d1e");
    req_logger.info("Processing request", &[("method", json!("POST")), ("path", json!("/api/v1/entries"))]);
    req_logger.debug("Request body parsed", &[("content_type", json!("application/json")), ("body_size", json!(1024))]);

    let trace_logger = pretty.with_trace_id("trace-abc123def456");
    trace_logger.info("Distributed trace started", &[]);

    // ──────────────────────────────────────────
    // 6. Helper functions
    // ──────────────────────────────────────────
    print_section("Helper Functions");

    log_success(&pretty, "Database migration completed");

    log_request(&pretty, "GET", "/api/v1/users/42", "baleeghu");
    log_request(&pretty, "POST", "/api/v1/entries", "servicebot");

    log_response(&pretty, "GET", "/api/v1/users/42", 200, 12.0, None);
    log_response(&pretty, "POST", "/api/v1/entries", 201, 45.0, None);
    log_response(&pretty, "GET", "/api/v1/secrets", 403, 2.0, Some("insufficient permissions"));
    log_response(&pretty, "GET", "/api/v1/missing", 404, 1.0, Some("resource not found"));
    log_response(&pretty, "POST", "/api/v1/entries", 500, 3000.0, Some("deadlock detected"));

    log_db_query(&pretty, "SELECT", "users", 3.0, 42);
    log_db_query(&pretty, "INSERT", "time_entries", 15.0, 1);
    log_db_query(&pretty, "DELETE", "old_sessions", 200.0, 1337);

    log_service_error(&pretty, "UserService", "CreateUser", "duplicate email", &[
        ("email", json!("user@example.com")),
        ("provider", json!("internal")),
    ]);

    log_service_debug(&pretty, "CacheService", "Get", "Cache lookup completed", &[
        ("key", json!("user:42:profile")),
        ("hit", json!(true)),
        ("ttl_ms", json!(45000)),
    ]);

    // ──────────────────────────────────────────
    // 7. Redaction
    // ──────────────────────────────────────────
    print_section("Sensitive Field Redaction");

    pretty.info("Login attempt with redacted credentials", &[
        ("username", json!("baleeghu")),
        ("password", json!("super_secret_pass_123")),
        ("api_key", json!("sk-proj-abc123def456ghi789")),
        ("email", json!("user@deshaw.com")),
    ]);

    let redacted = rustlogx::redact_map(&std::collections::HashMap::from([
        ("username".to_string(), "baleeghu".to_string()),
        ("password".to_string(), "hunter2".to_string()),
        ("access_token".to_string(), "eyJhbGciOiJIUzI1NiJ9".to_string()),
        ("action".to_string(), "login".to_string()),
    ]));
    pretty.info("Form submission (redacted)", &[("form_data", json!(redacted))]);

    // ──────────────────────────────────────────
    // 8. JSON output mode
    // ──────────────────────────────────────────
    print_section("JSON Output Mode (for production / log aggregators)");

    let json_logger = Logger::with_writer(
        Config {
            level: Level::Info,
            format: Format::Json,
            caller: true,
            no_color: false,
        },
        Box::new(std::io::stdout()),
    );

    rustlogx::log_info!(json_logger, "Application started",
        "environment" => "production",
        "version" => "2.4.1",
    );

    rustlogx::log_error!(json_logger, "Cache unavailable",
        "error" => "connection timeout",
        "service" => "redis",
        "host" => "redis.internal:6379",
    );

    // ──────────────────────────────────────────
    // 9. Environment-based initialization
    // ──────────────────────────────────────────
    print_section("Environment-based Init");
    println!("  (Set LOG_LEVEL, LOG_FORMAT, LOG_CALLER, NO_COLOR, TERM to configure)");
    let env_logger = Logger::new(Config::from_env());
    env_logger.info("Logger initialized from environment", &[]);

    // ──────────────────────────────────────────
    // Done!
    // ──────────────────────────────────────────
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    All formats demonstrated!                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

fn print_section(title: &str) {
    let pad = 58usize.saturating_sub(title.len());
    let line: String = "─".repeat(pad);
    println!();
    println!("── {title} {line}");
    println!();
}
