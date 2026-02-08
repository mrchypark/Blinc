use std::collections::HashMap;

use thiserror::Error;

use crate::label::ArgValue;
use crate::label::Message;

fn looks_like_yaml_mapping(src: &str) -> bool {
    for raw in src.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        // YAML mapping lines typically contain `:` as a key/value separator.
        // If `=` appears before `:`, it's more likely the legacy `key = value` format.
        if let Some(colon) = line.find(':') {
            match line.find('=') {
                Some(eq) if eq < colon => {}
                _ => return true,
            }
        }
        // If the first meaningful line looks like legacy, bail out early.
        if line.contains('=') {
            return false;
        }
    }
    false
}

fn try_parse_yaml_map(src: &str) -> Result<Option<HashMap<String, String>>, SimpleParseError> {
    match serde_yaml::from_str::<serde_yaml::Value>(src) {
        Ok(serde_yaml::Value::Mapping(raw)) => {
            let mut out = HashMap::with_capacity(raw.len());
            for (k, v) in raw {
                let Some(key) = k.as_str() else {
                    return Err(SimpleParseError::Yaml(
                        "yaml keys must be strings".to_string(),
                    ));
                };
                let Some(val) = v.as_str() else {
                    return Err(SimpleParseError::Yaml(format!(
                        "yaml value for key `{key}` must be a string"
                    )));
                };
                out.insert(key.to_string(), val.to_string());
            }
            Ok(Some(out))
        }
        Ok(_) => Ok(None),
        Err(e) => {
            if looks_like_yaml_mapping(src) {
                return Err(SimpleParseError::Yaml(format!("yaml parse error: {e}")));
            }
            Ok(None)
        }
    }
}

/// A minimal Blinc catalog format:
/// - One entry per line: `key = value`
/// - Comments: `# ...` or `// ...`
/// - Optional quoting: `"..."` or `'...'` (supports a few escapes)
/// - Placeholders: `{name}` replaced with args by name (stringified)
#[derive(Clone, Debug, Default)]
pub struct SimpleCatalog {
    entries: HashMap<String, String>,
}

impl SimpleCatalog {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|s| s.as_str())
    }

    /// Parse a YAML mapping (preferred) or fall back to the legacy key=value format.
    pub fn parse(src: &str) -> Result<Self, SimpleParseError> {
        if let Some(map) = try_parse_yaml_map(src)? {
            let mut cat = Self::new();
            for (k, v) in map {
                cat.insert(k, v);
            }
            return Ok(cat);
        }

        let mut cat = Self::new();
        for (idx, raw_line) in src.lines().enumerate() {
            let line_no = idx + 1;
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('#') || line.starts_with("//") {
                continue;
            }

            let Some(eq) = line.find('=') else {
                return Err(SimpleParseError::Syntax {
                    line: line_no,
                    msg: "expected `key = value`".to_string(),
                });
            };

            let key = line[..eq].trim();
            let mut value = line[eq + 1..].trim().to_string();
            if key.is_empty() {
                return Err(SimpleParseError::Syntax {
                    line: line_no,
                    msg: "empty key".to_string(),
                });
            }

            // Strip inline comments (only if preceded by whitespace).
            if let Some(pos) = value.find(" #") {
                value.truncate(pos);
                value = value.trim().to_string();
            }
            if let Some(pos) = value.find(" //") {
                value.truncate(pos);
                value = value.trim().to_string();
            }

            let value = unquote_and_unescape(&value).map_err(|e| SimpleParseError::Syntax {
                line: line_no,
                msg: e,
            })?;

            cat.insert(key, value);
        }
        Ok(cat)
    }

    pub fn format_message(&self, msg: &Message) -> Option<String> {
        let tmpl = self.get(msg.id.as_ref())?;
        Some(apply_placeholders(
            tmpl,
            &msg.args
                .iter()
                .map(|(k, v)| (k.as_ref(), v))
                .collect::<Vec<_>>(),
        ))
    }
}

#[derive(Debug, Error)]
pub enum SimpleParseError {
    #[error("yaml catalog error: {0}")]
    Yaml(String),

    #[error("simple catalog syntax error at line {line}: {msg}")]
    Syntax { line: usize, msg: String },
}

fn unquote_and_unescape(s: &str) -> Result<String, String> {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        return unescape(&s[1..s.len() - 1]);
    }
    if s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2 {
        return unescape(&s[1..s.len() - 1]);
    }
    Ok(s.to_string())
}

fn unescape(s: &str) -> Result<String, String> {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let Some(n) = it.next() else {
            return Err("dangling escape".to_string());
        };
        match n {
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            '\'' => out.push('\''),
            _ => {
                // Keep unknown escapes as-is.
                out.push(n);
            }
        }
    }
    Ok(out)
}

fn arg_to_string(v: &ArgValue) -> String {
    match v {
        ArgValue::Str(s) => s.clone(),
        ArgValue::Int(i) => i.to_string(),
        ArgValue::Float(f) => {
            // Keep it simple; formatting control is a future concern.
            let mut s = f.to_string();
            if s.contains('.') {
                while s.ends_with('0') {
                    s.pop();
                }
                if s.ends_with('.') {
                    s.pop();
                }
            }
            s
        }
        ArgValue::Bool(b) => b.to_string(),
    }
}

fn apply_placeholders(tmpl: &str, args: &[(&str, &ArgValue)]) -> String {
    if !tmpl.contains('{') && !tmpl.contains('}') {
        return tmpl.to_string();
    }

    const LINEAR_SEARCH_THRESHOLD: usize = 8;

    // `args` is usually tiny; avoid allocating a HashMap for small argument sets.
    let args_map = if args.len() > LINEAR_SEARCH_THRESHOLD {
        Some(
            args.iter()
                .copied()
                .collect::<std::collections::HashMap<&str, &ArgValue>>(),
        )
    } else {
        None
    };

    // Very small placeholder engine: replaces `{name}` tokens.
    let mut out = String::with_capacity(tmpl.len() + 8);
    let mut chars = tmpl.chars().peekable();

    while let Some(c) = chars.next() {
        // Support escaped braces: `{{` -> `{`, `}}` -> `}`.
        if c == '}' {
            if chars.peek() == Some(&'}') {
                chars.next();
                out.push('}');
                continue;
            }
            out.push('}');
            continue;
        }
        if c != '{' {
            out.push(c);
            continue;
        }

        if chars.peek() == Some(&'{') {
            chars.next();
            out.push('{');
            continue;
        }

        // Read until `}`.
        let mut key = String::new();
        let mut closed = false;
        while let Some(&n) = chars.peek() {
            chars.next();
            if n == '}' {
                closed = true;
                break;
            }
            key.push(n);
        }

        // If there's no closing brace, treat the rest as literal text.
        if !closed {
            out.push('{');
            out.push_str(&key);
            break;
        }

        let key = key.trim();
        if key.is_empty() {
            out.push_str("{}");
            continue;
        }

        let value = if let Some(map) = args_map.as_ref() {
            map.get(key).copied()
        } else {
            args.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
        };

        if let Some(v) = value {
            out.push_str(&arg_to_string(v));
        } else {
            // Keep unknown placeholders visible.
            out.push('{');
            out.push_str(key);
            out.push('}');
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_yaml_and_lookup() {
        let src = r#"
demo-title: "Blinc i18n Demo"
demo-hello: "Hello, {name}!"
"#;

        let cat = SimpleCatalog::parse(src).unwrap();
        assert_eq!(cat.get("demo-title"), Some("Blinc i18n Demo"));

        let s = cat
            .format_message(&Message::new("demo-hello").arg("name", "Chris"))
            .unwrap();
        assert_eq!(s, "Hello, Chris!");
    }

    #[test]
    fn parse_legacy_kv_and_lookup() {
        let src = r#"
        # comment
        demo-title = Blinc i18n Demo
        demo-hello = "Hello, {name}!"
        "#;

        let cat = SimpleCatalog::parse(src).unwrap();
        assert_eq!(cat.get("demo-title"), Some("Blinc i18n Demo"));

        let s = cat
            .format_message(&Message::new("demo-hello").arg("name", "Chris"))
            .unwrap();
        assert_eq!(s, "Hello, Chris!");
    }

    #[test]
    fn escaped_braces() {
        let name = ArgValue::from("Chris");
        let args = &[("name", &name)];
        assert_eq!(
            apply_placeholders("Hello, {{name}}!", args),
            "Hello, {name}!"
        );
        assert_eq!(apply_placeholders("{{{name}}}", args), "{Chris}");
        assert_eq!(apply_placeholders("}}", args), "}");
        assert_eq!(apply_placeholders("{{", args), "{");
    }

    #[test]
    fn missing_closing_brace_is_literal() {
        let name = ArgValue::from("Chris");
        let args = &[("name", &name)];
        assert_eq!(apply_placeholders("Hello, {name", args), "Hello, {name");
        assert_eq!(apply_placeholders("{name", args), "{name");
    }

    #[test]
    fn yaml_requires_string_values() {
        let src = r#"
demo-title: 123
"#;
        let err = SimpleCatalog::parse(src).unwrap_err();
        assert!(matches!(err, SimpleParseError::Yaml(_)));
    }
}
