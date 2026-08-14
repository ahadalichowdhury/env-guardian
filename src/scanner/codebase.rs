use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use regex::Regex;

use crate::config::EnvGuardianConfig;
use crate::error::Result;

/// A variable reference found in source code.
#[derive(Debug, Clone)]
pub struct EnvUsage {
    pub key: String,
    pub file: PathBuf,
    pub line: usize,
}

/// Built-in regex patterns for common env access styles.
fn build_patterns() -> Vec<Regex> {
    let raw_patterns = [
        // JS/TS: process.env.VAR_NAME
        r"process\.env\.([A-Z][A-Z0-9_]*)",
        // JS/TS: process.env['VAR'] or process.env["VAR"]
        r#"process\.env\[['"]([A-Z][A-Z0-9_]*)['"]\]"#,
        // JS/TS: import.meta.env.VAR
        r"import\.meta\.env\.([A-Z][A-Z0-9_]*)",
        // Python: os.getenv('VAR')
        r#"os\.getenv\(\s*['"]([A-Z][A-Z0-9_]*)['"]"#,
        // Python: os.environ['VAR']
        r#"os\.environ\[['"]([A-Z][A-Z0-9_]*)['"]\]"#,
        // Rust: std::env::var("VAR")
        r#"std::env::var\(\s*["']([A-Z][A-Z0-9_]*)["']"#,
        // Rust: env!("VAR")
        r#"env!\(\s*["']([A-Z][A-Z0-9_]*)["']"#,
        // Go: os.Getenv("VAR")
        r#"os\.Getenv\(\s*["']([A-Z][A-Z0-9_]*)["']"#,
        // Ruby: ENV['VAR'] or ENV.fetch('VAR')
        r#"ENV\[['"]([A-Z][A-Z0-9_]*)['"]\]"#,
        r#"ENV\.fetch\(\s*['"]([A-Z][A-Z0-9_]*)['"]"#,
    ];

    raw_patterns
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
}

/// Scan codebase for environment variable usage.
pub fn scan_codebase(root: &Path, config: &EnvGuardianConfig) -> Result<Vec<EnvUsage>> {
    if !config.scan.enabled {
        return Ok(Vec::new());
    }

    let patterns = build_patterns();
    let mut usages: BTreeMap<String, EnvUsage> = BTreeMap::new();

    let mut scan_roots: Vec<PathBuf> = if config.scan.include.is_empty() {
        vec![root.to_path_buf()]
    } else {
        config
            .scan
            .include
            .iter()
            .map(|p| root.join(p))
            .filter(|p| p.exists())
            .collect()
    };

    if scan_roots.is_empty() {
        scan_roots.push(root.to_path_buf());
    }

    for scan_root in scan_roots {
        let walker = WalkBuilder::new(&scan_root)
            .hidden(true)
            .git_ignore(true)
            .git_global(false)
            .git_exclude(true)
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if should_skip(path, root, &config.scan.exclude) {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                scan_file_content(&content, path, &patterns, &mut usages);
            }
        }
    }

    Ok(usages.values().cloned().collect())
}

fn should_skip(path: &Path, root: &Path, exclude: &[String]) -> bool {
    let rel = path
        .strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    for part in &rel {
        if exclude.iter().any(|e| e == part) {
            return true;
        }
    }

    // Skip common binary / lock files by extension
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    matches!(
        ext,
        "png" | "jpg" | "jpeg" | "gif" | "ico" | "svg" | "woff" | "woff2" | "ttf" | "eot"
            | "pdf" | "zip" | "gz" | "tar" | "bin" | "exe" | "dll" | "so" | "dylib" | "lock"
    )
}

fn scan_file_content(
    content: &str,
    path: &Path,
    patterns: &[Regex],
    usages: &mut BTreeMap<String, EnvUsage>,
) {
    for (line_idx, line) in content.lines().enumerate() {
        for pattern in patterns {
            for cap in pattern.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    let key = m.as_str().to_string();
                    if !usages.contains_key(&key) {
                        usages.insert(
                            key.clone(),
                            EnvUsage {
                                key,
                                file: path.to_path_buf(),
                                line: line_idx + 1,
                            },
                        );
                    }
                }
            }
        }
    }
}

/// Extract unique keys from scan results.
pub fn unique_keys(usages: &[EnvUsage]) -> BTreeSet<String> {
    usages.iter().map(|u| u.key.clone()).collect()
}

/// Scan a single file's content (for tests).
pub fn scan_content(content: &str, path: &Path) -> Result<Vec<EnvUsage>> {
    let patterns = build_patterns();
    let mut usages: BTreeMap<String, EnvUsage> = BTreeMap::new();
    scan_file_content(content, path, &patterns, &mut usages);
    Ok(usages.values().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_js_process_env() {
        let src = "const url = process.env.DATABASE_URL;\nconst x = process.env['API_KEY'];";
        let usages = scan_content(src, Path::new("app.ts")).unwrap();
        let keys: Vec<_> = usages.iter().map(|u| u.key.as_str()).collect();
        assert!(keys.contains(&"DATABASE_URL"));
        assert!(keys.contains(&"API_KEY"));
    }

    #[test]
    fn detects_python_and_rust() {
        let src = "os.getenv('SECRET_KEY')\nstd::env::var(\"PORT\")";
        let usages = scan_content(src, Path::new("mixed.txt")).unwrap();
        let keys: Vec<_> = usages.iter().map(|u| u.key.as_str()).collect();
        assert!(keys.contains(&"SECRET_KEY"));
        assert!(keys.contains(&"PORT"));
    }
}
