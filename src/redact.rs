//! Field redaction for sensitive values.

use std::collections::HashMap;

/// Sensitive key substrings (case-insensitive match).
const SENSITIVE_SUBSTRINGS: &[&str] = &[
    "password",
    "secret",
    "token",
    "access_token",
    "refresh_token",
    "authorization",
    "api_key",
    "apikey",
    "api-key",
    "credit_card",
    "ssn",
];

/// Returns `true` if `key` contains any sensitive substring (case-insensitive).
pub fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    SENSITIVE_SUBSTRINGS.iter().any(|s| lower.contains(s))
}

/// Masks a sensitive value based on its length.
///
/// - empty → `"****"`
/// - 1-2 chars → all asterisks
/// - 3-6 chars → first char visible, rest asterisks
/// - 7+ chars → first and last char visible, middle asterisks
pub fn redact_value(s: &str) -> String {
    let n = s.len();
    match n {
        0 => "****".to_string(),
        1..=2 => "*".repeat(n),
        3..=6 => {
            let first = &s[..1];
            format!("{}{}", first, "*".repeat(n - 1))
        }
        _ => {
            let first = &s[..1];
            let last = &s[n - 1..];
            format!("{}{}{}", first, "*".repeat(n - 2), last)
        }
    }
}

/// Returns a copy of `map` with sensitive values redacted.
pub fn redact_map(map: &HashMap<String, String>) -> HashMap<String, String> {
    map.iter()
        .map(|(k, v)| {
            let val = if is_sensitive_key(k) {
                redact_value(v)
            } else {
                v.clone()
            };
            (k.clone(), val)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitive_keys() {
        assert!(is_sensitive_key("password"));
        assert!(is_sensitive_key("Password"));
        assert!(is_sensitive_key("PASSWORD"));
        assert!(is_sensitive_key("user_password"));
        assert!(is_sensitive_key("secret"));
        assert!(is_sensitive_key("token"));
        assert!(is_sensitive_key("access_token"));
        assert!(is_sensitive_key("Authorization"));
        assert!(is_sensitive_key("api_key"));
        assert!(is_sensitive_key("apikey"));
        assert!(is_sensitive_key("X-Api-Key"));
        assert!(is_sensitive_key("credit_card"));
        assert!(is_sensitive_key("ssn"));
    }

    #[test]
    fn test_non_sensitive_keys() {
        assert!(!is_sensitive_key("username"));
        assert!(!is_sensitive_key("email"));
        assert!(!is_sensitive_key("host"));
        assert!(!is_sensitive_key("port"));
        assert!(!is_sensitive_key(""));
    }

    #[test]
    fn test_redact_value() {
        assert_eq!(redact_value(""), "****");
        assert_eq!(redact_value("a"), "*");
        assert_eq!(redact_value("ab"), "**");
        assert_eq!(redact_value("abc"), "a**");
        assert_eq!(redact_value("abcd"), "a***");
        assert_eq!(redact_value("abcdef"), "a*****");
        assert_eq!(redact_value("abcdefg"), "a*****g");
        assert_eq!(redact_value("abcdefghij"), "a********j");
    }

    #[test]
    fn test_redact_map() {
        let mut m = HashMap::new();
        m.insert("username".to_string(), "john".to_string());
        m.insert("password".to_string(), "secret123".to_string());
        m.insert("host".to_string(), "localhost".to_string());

        let result = redact_map(&m);
        assert_eq!(result["username"], "john");
        assert_eq!(result["host"], "localhost");
        assert_eq!(result["password"], "s*******3");
    }
}
