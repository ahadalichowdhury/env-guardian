use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::error::{AppError, Result};

/// Parsed .env file: key -> value (empty string if unset).
#[derive(Debug, Clone, Default)]
pub struct EnvFile {
    pub vars: BTreeMap<String, String>,
}

impl EnvFile {
    pub fn keys(&self) -> Vec<String> {
        self.vars.keys().cloned().collect()
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.vars.contains_key(key)
    }

    pub fn is_empty_value(&self, key: &str) -> bool {
        self.vars.get(key).map(|v| v.is_empty()).unwrap_or(false)
    }
}

/// Parse a .env file from disk.
pub fn parse_env_file(path: &Path) -> Result<EnvFile> {
    if !path.exists() {
        return Err(AppError::FileNotFound(path.to_path_buf()));
    }
    let contents = fs::read_to_string(path)?;
    Ok(parse_env_contents(&contents))
}

/// Parse .env contents (supports comments, quotes, export prefix).
pub fn parse_env_contents(contents: &str) -> EnvFile {
    let mut vars = BTreeMap::new();

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let key = key.trim();
        if key.is_empty() {
            continue;
        }

        let value = parse_value(value.trim());
        vars.insert(key.to_string(), value);
    }

    EnvFile { vars }
}

fn parse_value(raw: &str) -> String {
    if raw.is_empty() {
        return String::new();
    }

    let quote = raw.chars().next();
    if quote == Some('"') || quote == Some('\'') {
        let q = quote.unwrap();
        let inner = raw[1..].rsplit_once(q).map(|(a, _)| a).unwrap_or(&raw[1..]);
        return unescape(inner, q);
    }

    // Strip inline comment (unquoted values only)
    let end = raw.find('#').unwrap_or(raw.len());
    raw[..end].trim().to_string()
}

fn unescape(s: &str, quote: char) -> String {
    if quote != '"' {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                chars.next();
                match next {
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    '\\' => out.push('\\'),
                    '"' => out.push('"'),
                    other => {
                        out.push('\\');
                        out.push(other);
                    }
                }
            } else {
                out.push('\\');
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_keys() {
        let env = parse_env_contents("KEY=value\nOTHER=123");
        assert_eq!(env.vars.get("KEY"), Some(&"value".to_string()));
        assert_eq!(env.vars.get("OTHER"), Some(&"123".to_string()));
    }

    #[test]
    fn skips_comments_and_export() {
        let env = parse_env_contents("# comment\nexport FOO=bar\nEMPTY=");
        assert_eq!(env.vars.get("FOO"), Some(&"bar".to_string()));
        assert_eq!(env.vars.get("EMPTY"), Some(&String::new()));
        assert!(!env.vars.contains_key("comment"));
    }

    #[test]
    fn parses_quoted_values() {
        let env = parse_env_contents("A=\"hello world\"\nB='single'");
        assert_eq!(env.vars.get("A"), Some(&"hello world".to_string()));
        assert_eq!(env.vars.get("B"), Some(&"single".to_string()));
    }
}
