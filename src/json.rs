//! Custom JSON value type, serializer, and parser.
//! Replaces serde_json to eliminate external dependencies.

use std::collections::HashMap;
use std::fmt;
use std::ops::Index;

/// A JSON number, either integer or floating-point.
#[derive(Debug, Clone)]
pub enum JsonNumber {
    Int(i64),
    Float(f64),
}

impl PartialEq for JsonNumber {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsonNumber::Int(a), JsonNumber::Int(b)) => a == b,
            (JsonNumber::Float(a), JsonNumber::Float(b)) => a == b,
            (JsonNumber::Int(a), JsonNumber::Float(b)) => (*a as f64) == *b,
            (JsonNumber::Float(a), JsonNumber::Int(b)) => *a == (*b as f64),
        }
    }
}

/// A JSON value.
#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(JsonNumber),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl PartialEq for JsonValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (JsonValue::Null, JsonValue::Null) => true,
            (JsonValue::Bool(a), JsonValue::Bool(b)) => a == b,
            (JsonValue::Number(a), JsonValue::Number(b)) => a == b,
            (JsonValue::String(a), JsonValue::String(b)) => a == b,
            (JsonValue::Array(a), JsonValue::Array(b)) => a == b,
            (JsonValue::Object(a), JsonValue::Object(b)) => a == b,
            _ => false,
        }
    }
}

// --- Comparison impls for ergonomic test assertions ---

impl PartialEq<&str> for JsonValue {
    fn eq(&self, other: &&str) -> bool {
        matches!(self, JsonValue::String(s) if s.as_str() == *other)
    }
}

impl<'a> PartialEq<&str> for &'a JsonValue {
    fn eq(&self, other: &&str) -> bool {
        **self == *other
    }
}

impl PartialEq<i32> for JsonValue {
    fn eq(&self, other: &i32) -> bool {
        matches!(self, JsonValue::Number(JsonNumber::Int(n)) if *n == *other as i64)
    }
}

impl<'a> PartialEq<i32> for &'a JsonValue {
    fn eq(&self, other: &i32) -> bool {
        **self == *other
    }
}

impl PartialEq<i64> for JsonValue {
    fn eq(&self, other: &i64) -> bool {
        matches!(self, JsonValue::Number(JsonNumber::Int(n)) if *n == *other)
    }
}

impl<'a> PartialEq<i64> for &'a JsonValue {
    fn eq(&self, other: &i64) -> bool {
        **self == *other
    }
}

impl<'a> PartialEq<JsonValue> for &'a JsonValue {
    fn eq(&self, other: &JsonValue) -> bool {
        **self == *other
    }
}

// --- Helper methods ---

impl JsonValue {
    /// Returns the contained string value, if any.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

// --- Index by &str for object field access ---

impl Index<&str> for JsonValue {
    type Output = JsonValue;
    fn index(&self, key: &str) -> &JsonValue {
        match self {
            JsonValue::Object(pairs) => {
                for (k, v) in pairs {
                    if k == key {
                        return v;
                    }
                }
                panic!("key not found: {key}");
            }
            _ => panic!("JsonValue::index called on non-object"),
        }
    }
}

// --- From impls for constructing JsonValue ---

impl From<&str> for JsonValue {
    fn from(s: &str) -> Self {
        JsonValue::String(s.to_string())
    }
}

impl From<String> for JsonValue {
    fn from(s: String) -> Self {
        JsonValue::String(s)
    }
}

impl From<bool> for JsonValue {
    fn from(b: bool) -> Self {
        JsonValue::Bool(b)
    }
}

impl From<i32> for JsonValue {
    fn from(n: i32) -> Self {
        JsonValue::Number(JsonNumber::Int(n as i64))
    }
}

impl From<i64> for JsonValue {
    fn from(n: i64) -> Self {
        JsonValue::Number(JsonNumber::Int(n))
    }
}

impl From<u16> for JsonValue {
    fn from(n: u16) -> Self {
        JsonValue::Number(JsonNumber::Int(n as i64))
    }
}

impl From<u32> for JsonValue {
    fn from(n: u32) -> Self {
        JsonValue::Number(JsonNumber::Int(n as i64))
    }
}

impl From<f64> for JsonValue {
    fn from(n: f64) -> Self {
        JsonValue::Number(JsonNumber::Float(n))
    }
}

impl From<HashMap<String, String>> for JsonValue {
    fn from(m: HashMap<String, String>) -> Self {
        let pairs: Vec<(String, JsonValue)> = m
            .into_iter()
            .map(|(k, v)| (k, JsonValue::String(v)))
            .collect();
        JsonValue::Object(pairs)
    }
}

// --- Display: JSON serialization ---

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => f.write_str("null"),
            JsonValue::Bool(b) => f.write_str(if *b { "true" } else { "false" }),
            JsonValue::Number(n) => fmt::Display::fmt(n, f),
            JsonValue::String(s) => write_json_string(f, s),
            JsonValue::Array(arr) => {
                f.write_str("[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    fmt::Display::fmt(v, f)?;
                }
                f.write_str("]")
            }
            JsonValue::Object(pairs) => {
                f.write_str("{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write_json_string(f, k)?;
                    f.write_str(":")?;
                    fmt::Display::fmt(v, f)?;
                }
                f.write_str("}")
            }
        }
    }
}

impl fmt::Display for JsonNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonNumber::Int(n) => write!(f, "{n}"),
            JsonNumber::Float(n) => write!(f, "{n}"),
        }
    }
}

/// Writes a JSON-escaped string (with surrounding quotes) per RFC 8259.
fn write_json_string(f: &mut fmt::Formatter<'_>, s: &str) -> fmt::Result {
    f.write_str("\"")?;
    for c in s.chars() {
        match c {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\u{0008}' => f.write_str("\\b")?,
            '\u{000C}' => f.write_str("\\f")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
            c => {
                let mut buf = [0u8; 4];
                f.write_str(c.encode_utf8(&mut buf))?;
            }
        }
    }
    f.write_str("\"")
}

// --- JSON parser (test-only) ---

/// Parses a JSON string into a JsonValue.
#[cfg(test)]
pub fn from_json_str(input: &str) -> Option<JsonValue> {
    let bytes = input.trim().as_bytes();
    let (val, _) = parse_value(bytes, 0)?;
    Some(val)
}

#[cfg(test)]
fn skip_ws(input: &[u8], mut pos: usize) -> usize {
    while pos < input.len() && matches!(input[pos], b' ' | b'\t' | b'\n' | b'\r') {
        pos += 1;
    }
    pos
}

#[cfg(test)]
fn parse_value(input: &[u8], pos: usize) -> Option<(JsonValue, usize)> {
    let pos = skip_ws(input, pos);
    if pos >= input.len() {
        return None;
    }
    match input[pos] {
        b'"' => {
            let (s, p) = parse_string(input, pos)?;
            Some((JsonValue::String(s), p))
        }
        b'{' => parse_object(input, pos),
        b'[' => parse_array(input, pos),
        b't' | b'f' => parse_bool(input, pos),
        b'n' => parse_null(input, pos),
        b'-' | b'0'..=b'9' => parse_number(input, pos),
        _ => None,
    }
}

#[cfg(test)]
fn parse_string(input: &[u8], pos: usize) -> Option<(String, usize)> {
    if input[pos] != b'"' {
        return None;
    }
    let mut pos = pos + 1;
    let mut s = String::new();
    while pos < input.len() {
        match input[pos] {
            b'"' => return Some((s, pos + 1)),
            b'\\' => {
                pos += 1;
                if pos >= input.len() {
                    return None;
                }
                match input[pos] {
                    b'"' => s.push('"'),
                    b'\\' => s.push('\\'),
                    b'/' => s.push('/'),
                    b'b' => s.push('\u{0008}'),
                    b'f' => s.push('\u{000C}'),
                    b'n' => s.push('\n'),
                    b'r' => s.push('\r'),
                    b't' => s.push('\t'),
                    b'u' => {
                        if pos + 4 >= input.len() {
                            return None;
                        }
                        let hex = std::str::from_utf8(&input[pos + 1..pos + 5]).ok()?;
                        let code = u32::from_str_radix(hex, 16).ok()?;
                        s.push(char::from_u32(code)?);
                        pos += 4;
                    }
                    _ => return None,
                }
                pos += 1;
            }
            b => {
                s.push(b as char);
                pos += 1;
            }
        }
    }
    None
}

#[cfg(test)]
fn parse_number(input: &[u8], pos: usize) -> Option<(JsonValue, usize)> {
    let start = pos;
    let mut pos = pos;
    let mut is_float = false;

    if pos < input.len() && input[pos] == b'-' {
        pos += 1;
    }
    while pos < input.len() && input[pos].is_ascii_digit() {
        pos += 1;
    }
    if pos < input.len() && input[pos] == b'.' {
        is_float = true;
        pos += 1;
        while pos < input.len() && input[pos].is_ascii_digit() {
            pos += 1;
        }
    }
    if pos < input.len() && (input[pos] == b'e' || input[pos] == b'E') {
        is_float = true;
        pos += 1;
        if pos < input.len() && (input[pos] == b'+' || input[pos] == b'-') {
            pos += 1;
        }
        while pos < input.len() && input[pos].is_ascii_digit() {
            pos += 1;
        }
    }

    let s = std::str::from_utf8(&input[start..pos]).ok()?;
    if is_float {
        let f: f64 = s.parse().ok()?;
        Some((JsonValue::Number(JsonNumber::Float(f)), pos))
    } else {
        let n: i64 = s.parse().ok()?;
        Some((JsonValue::Number(JsonNumber::Int(n)), pos))
    }
}

#[cfg(test)]
fn parse_object(input: &[u8], pos: usize) -> Option<(JsonValue, usize)> {
    if input[pos] != b'{' {
        return None;
    }
    let mut pos = skip_ws(input, pos + 1);
    let mut pairs = Vec::new();

    if pos < input.len() && input[pos] == b'}' {
        return Some((JsonValue::Object(pairs), pos + 1));
    }

    loop {
        pos = skip_ws(input, pos);
        let (key, p) = parse_string(input, pos)?;
        pos = skip_ws(input, p);
        if pos >= input.len() || input[pos] != b':' {
            return None;
        }
        pos = skip_ws(input, pos + 1);
        let (val, p) = parse_value(input, pos)?;
        pairs.push((key, val));
        pos = skip_ws(input, p);
        if pos >= input.len() {
            return None;
        }
        if input[pos] == b'}' {
            return Some((JsonValue::Object(pairs), pos + 1));
        }
        if input[pos] == b',' {
            pos += 1;
            continue;
        }
        return None;
    }
}

#[cfg(test)]
fn parse_array(input: &[u8], pos: usize) -> Option<(JsonValue, usize)> {
    if input[pos] != b'[' {
        return None;
    }
    let mut pos = skip_ws(input, pos + 1);
    let mut arr = Vec::new();

    if pos < input.len() && input[pos] == b']' {
        return Some((JsonValue::Array(arr), pos + 1));
    }

    loop {
        let (val, p) = parse_value(input, pos)?;
        arr.push(val);
        pos = skip_ws(input, p);
        if pos >= input.len() {
            return None;
        }
        if input[pos] == b']' {
            return Some((JsonValue::Array(arr), pos + 1));
        }
        if input[pos] == b',' {
            pos = skip_ws(input, pos + 1);
            continue;
        }
        return None;
    }
}

#[cfg(test)]
fn parse_bool(input: &[u8], pos: usize) -> Option<(JsonValue, usize)> {
    if input[pos..].starts_with(b"true") {
        Some((JsonValue::Bool(true), pos + 4))
    } else if input[pos..].starts_with(b"false") {
        Some((JsonValue::Bool(false), pos + 5))
    } else {
        None
    }
}

#[cfg(test)]
fn parse_null(input: &[u8], pos: usize) -> Option<(JsonValue, usize)> {
    if input[pos..].starts_with(b"null") {
        Some((JsonValue::Null, pos + 4))
    } else {
        None
    }
}

/// Custom json! macro replacing serde_json::json!().
#[macro_export]
macro_rules! json {
    ($e:expr) => {
        $crate::JsonValue::from($e)
    };
}
