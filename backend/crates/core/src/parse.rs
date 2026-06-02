//! 数值/文本解析的共享小工具（此前在 news/art/exam/shop 各写一份，现统一）。

use serde_json::Value;

/// 模拟 JS `Number.parseInt(x, 10)`：取十进制前缀，无有效数字返回 None。
pub fn parse_int_radix10(s: &str) -> Option<i64> {
    let t = s.trim_start();
    let bytes = t.as_bytes();
    let mut i = 0;
    let mut sign = 1_i64;
    if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
        if bytes[i] == b'-' {
            sign = -1;
        }
        i += 1;
    }
    let start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return None;
    }
    t[start..i].parse::<i64>().ok().map(|n| sign * n)
}

/// `parse_int_radix10` + JS falsy 语义：0 或无效都取 default。
pub fn parse_int_or(s: Option<&str>, default: i64) -> i64 {
    match s.and_then(parse_int_radix10) {
        Some(v) if v != 0 => v,
        _ => default,
    }
}

/// 从 `serde_json::Value` 取 i64（兼容整数/浮点/数字字符串）。
pub fn num_i64(v: &Value) -> Option<i64> {
    if let Some(n) = v.as_i64() {
        Some(n)
    } else if let Some(f) = v.as_f64() {
        Some(f as i64)
    } else {
        v.as_str()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .map(|f| f as i64)
    }
}

/// 解析为浮点、floor 后裁剪到 [min, max]，无效返回 d。
pub fn clamp_int(v: Option<&str>, min: i64, max: i64, d: i64) -> i64 {
    match v.and_then(|s| s.trim().parse::<f64>().ok()) {
        Some(n) if n.is_finite() => (n.floor() as i64).clamp(min, max),
        _ => d,
    }
}

/// trim 后截断到最多 m 个字符（按 char）。
pub fn clamp_len(s: Option<&str>, m: usize) -> String {
    s.unwrap_or("").chars().take(m).collect()
}

/// trim 后的字符串（None → ""）。
pub fn safe_text(v: Option<&str>) -> String {
    v.unwrap_or("").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_int_radix10_js_semantics() {
        assert_eq!(parse_int_radix10("42abc"), Some(42));
        assert_eq!(parse_int_radix10("  -7 "), Some(-7));
        assert_eq!(parse_int_radix10("+5"), Some(5));
        assert_eq!(parse_int_radix10("abc"), None);
        assert_eq!(parse_int_radix10(""), None);
    }

    #[test]
    fn parse_int_or_falsy_default() {
        assert_eq!(parse_int_or(Some("0"), 9), 9); // 0 falsy → default
        assert_eq!(parse_int_or(Some("3"), 9), 3);
        assert_eq!(parse_int_or(None, 9), 9);
        assert_eq!(parse_int_or(Some("x"), 9), 9);
    }

    #[test]
    fn clamp_and_text() {
        assert_eq!(clamp_int(Some("100"), 1, 50, 10), 50);
        assert_eq!(clamp_int(Some("-3"), 0, 50, 10), 0);
        assert_eq!(clamp_int(None, 1, 50, 10), 10);
        assert_eq!(clamp_len(Some("héllo"), 3), "hél");
        assert_eq!(safe_text(Some("  hi ")), "hi");
    }

    #[test]
    fn num_i64_variants() {
        assert_eq!(num_i64(&json!(5)), Some(5));
        assert_eq!(num_i64(&json!(5.9)), Some(5));
        assert_eq!(num_i64(&json!("7")), Some(7));
        assert_eq!(num_i64(&json!(true)), None);
    }
}
