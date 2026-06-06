//! Convenience macros for logging with automatic caller info.

/// Logs at trace level with automatic caller info.
#[macro_export]
macro_rules! log_trace {
    ($logger:expr, $msg:expr $(, $key:expr => $val:expr)* $(,)?) => {
        $logger.log_with_caller(
            $crate::config::Level::Trace,
            $msg,
            &[$(($key, serde_json::json!($val)),)*],
            file!(),
            line!(),
        )
    };
}

/// Logs at debug level with automatic caller info.
#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $msg:expr $(, $key:expr => $val:expr)* $(,)?) => {
        $logger.log_with_caller(
            $crate::config::Level::Debug,
            $msg,
            &[$(($key, serde_json::json!($val)),)*],
            file!(),
            line!(),
        )
    };
}

/// Logs at info level with automatic caller info.
#[macro_export]
macro_rules! log_info {
    ($logger:expr, $msg:expr $(, $key:expr => $val:expr)* $(,)?) => {
        $logger.log_with_caller(
            $crate::config::Level::Info,
            $msg,
            &[$(($key, serde_json::json!($val)),)*],
            file!(),
            line!(),
        )
    };
}

/// Logs at warn level with automatic caller info.
#[macro_export]
macro_rules! log_warn {
    ($logger:expr, $msg:expr $(, $key:expr => $val:expr)* $(,)?) => {
        $logger.log_with_caller(
            $crate::config::Level::Warn,
            $msg,
            &[$(($key, serde_json::json!($val)),)*],
            file!(),
            line!(),
        )
    };
}

/// Logs at error level with automatic caller info.
#[macro_export]
macro_rules! log_error {
    ($logger:expr, $msg:expr $(, $key:expr => $val:expr)* $(,)?) => {
        $logger.log_with_caller(
            $crate::config::Level::Error,
            $msg,
            &[$(($key, serde_json::json!($val)),)*],
            file!(),
            line!(),
        )
    };
}

/// Logs at fatal level with automatic caller info.
#[macro_export]
macro_rules! log_fatal {
    ($logger:expr, $msg:expr $(, $key:expr => $val:expr)* $(,)?) => {
        $logger.log_with_caller(
            $crate::config::Level::Fatal,
            $msg,
            &[$(($key, serde_json::json!($val)),)*],
            file!(),
            line!(),
        )
    };
}
