use std::collections::HashMap;

use crate::Result;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Value {
    Integer(u64),
    Text { content: String, format: String },
    Vector(Vec<u8>),
}

pub(crate) fn integer_fits(value: u64, bits: u64) -> bool {
    if bits == 64 {
        return true;
    }
    let mask = (1u64 << bits) - 1;
    let encoded = value & mask;
    // Accept either an unsigned value or a value whose discarded bits are sign extension.
    value == encoded || (encoded & (1u64 << (bits - 1)) != 0 && value >> bits == u64::MAX >> bits)
}

pub(crate) fn parse_unsigned(input: &str) -> Result<u64> {
    let s = input.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).map_err(|_| format!("invalid unsigned integer: {input}"))
    } else if let Some(hex) = s.strip_suffix('h').or_else(|| s.strip_suffix('H')) {
        u64::from_str_radix(hex, 16).map_err(|_| format!("invalid unsigned integer: {input}"))
    } else {
        s.parse::<u64>()
            .map_err(|_| format!("invalid unsigned integer: {input}"))
    }
}

pub(crate) fn parse_integer(input: &str) -> Result<u64> {
    let s = input.trim();
    let (negative, body) = if let Some(rest) = s.strip_prefix('-') {
        (true, rest)
    } else {
        (false, s.strip_prefix('+').unwrap_or(s))
    };
    let magnitude = parse_unsigned(body)?;
    if negative {
        if magnitude > (1u64 << 63) {
            return Err(format!("signed integer is outside i64 range: {input}"));
        }
        Ok((0u64).wrapping_sub(magnitude))
    } else {
        if magnitude > i64::MAX as u64 {
            return Err(format!("signed integer is outside i64 range: {input}"));
        }
        Ok(magnitude)
    }
}

pub(crate) fn smallest_bits(value: u64, signed_input: bool) -> u64 {
    if signed_input {
        let value = value as i64;
        if (-128..=127).contains(&value) {
            8
        } else if (-32768..=32767).contains(&value) {
            16
        } else if (i32::MIN as i64..=i32::MAX as i64).contains(&value) {
            32
        } else {
            64
        }
    } else if value <= 0xFF {
        8
    } else if value <= 0xFFFF {
        16
    } else if value <= 0xFFFF_FFFF {
        32
    } else {
        64
    }
}

pub(crate) fn expression(input: &str, vars: &HashMap<String, Value>) -> Result<u64> {
    let mut parts = Vec::new();
    let mut start = 0;
    // Split only on interior signs so a leading sign remains part of its numeric term.
    for (i, ch) in input.char_indices() {
        if i > 0 && (ch == '+' || ch == '-') {
            parts.push(&input[start..i]);
            start = i;
        }
    }
    parts.push(&input[start..]);
    let mut total = 0u64;
    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            return Err(format!("invalid integer expression: {input}"));
        }
        let (negative, term) = if let Some(s) = part.strip_prefix('-') {
            (true, s)
        } else {
            (false, part.strip_prefix('+').unwrap_or(part))
        };
        let value = match vars.get(term) {
            Some(Value::Integer(value)) => *value,
            Some(_) => return Err(format!("variable {term} is not an integer")),
            None => parse_unsigned(term)?,
        };
        total = if negative {
            total.wrapping_sub(value)
        } else {
            total.wrapping_add(value)
        };
    }
    Ok(total)
}

pub(crate) fn integer_argument(input: &str, vars: &HashMap<String, Value>) -> Result<u64> {
    let has_operator = input
        .char_indices()
        .any(|(i, c)| i > 0 && (c == '+' || c == '-'));
    if vars.contains_key(input) || has_operator {
        expression(input, vars)
    } else {
        parse_integer(input)
    }
}

pub(crate) fn variable_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 16
        || !name.is_ascii()
        || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(format!("invalid variable name: {name}"));
    }
    Ok(())
}

pub(crate) fn vector(input: &str, vars: &HashMap<String, Value>) -> Result<Vec<u8>> {
    if let Some(value) = vars.get(input) {
        return match value {
            Value::Vector(v) => Ok(v.clone()),
            _ => Err(format!("variable {input} is not a vector")),
        };
    }
    let inner = input
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(input);
    let without_prefixes = inner.replace("0x", "").replace("0X", "");
    let normalized: String = without_prefixes
        .chars()
        .filter(|c| !matches!(c, ',' | ':' | ';' | '-' | ' ' | '\t'))
        .collect();
    let mut bytes = Vec::new();
    if normalized.is_empty() || !normalized.len().is_multiple_of(2) || !normalized.is_ascii() {
        return Err("vector must contain 1-64 bytes written as hexadecimal".into());
    }
    for i in (0..normalized.len()).step_by(2) {
        bytes.push(
            u8::from_str_radix(&normalized[i..i + 2], 16)
                .map_err(|_| "invalid vector byte".to_string())?,
        );
    }
    if bytes.len() > 64 {
        return Err("vector may contain at most 64 bytes".into());
    }
    Ok(bytes)
}

pub(crate) fn text_value(
    input: &str,
    vars: &HashMap<String, Value>,
) -> Result<(String, Option<String>)> {
    match vars.get(input) {
        Some(Value::Text { content, format }) => Ok((content.clone(), Some(format.clone()))),
        Some(_) => Err(format!("variable {input} is not text")),
        None => {
            let text = if input.len() >= 2 && input.starts_with('"') && input.ends_with('"') {
                &input[1..input.len() - 1]
            } else {
                input
            };
            let mut result = String::new();
            let mut chars = text.chars();
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    match chars.next() {
                        Some('"') => result.push('"'),
                        Some('n') => result.push('\n'),
                        Some('r') => result.push('\r'),
                        Some('\\') => result.push('\\'),
                        Some(other) => {
                            result.push('\\');
                            result.push(other);
                        }
                        None => result.push('\\'),
                    }
                } else {
                    result.push(ch);
                }
            }
            Ok((result, None))
        }
    }
}

pub(crate) fn encode_text(
    input: &str,
    format: Option<&str>,
    nullterm: bool,
    vars: &HashMap<String, Value>,
) -> Result<Vec<u8>> {
    let (text, inherited_format) = text_value(input, vars)?;
    let format = format.or(inherited_format.as_deref()).unwrap_or("ascii");
    text_bytes(&text, format, nullterm)
}

pub(crate) fn text_bytes(text: &str, format: &str, nullterm: bool) -> Result<Vec<u8>> {
    let mut bytes = match format.to_ascii_lowercase().as_str() {
        "ascii" if text.is_ascii() => text.as_bytes().to_vec(),
        "ascii" => return Err("ASCII text contains non-ASCII characters".into()),
        "utf8" => text.as_bytes().to_vec(),
        "utf16" => text.encode_utf16().flat_map(u16::to_le_bytes).collect(),
        _ => return Err(format!("unsupported text format: {format}")),
    };
    if nullterm {
        bytes.extend(if format.eq_ignore_ascii_case("utf16") {
            vec![0, 0]
        } else {
            vec![0]
        });
    }
    Ok(bytes)
}
