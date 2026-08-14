use std::collections::BTreeMap;
use std::path::Path;

use crate::env::parser::{parse_env_contents, parse_env_file, EnvFile};
use crate::error::{AppError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriftKind {
    MissingLocal,
    MissingRemote,
    ValueMismatch,
}

#[derive(Debug, Clone)]
pub struct DriftItem {
    pub key: String,
    pub kind: DriftKind,
}

#[derive(Debug, Default)]
pub struct DriftReport {
    pub items: Vec<DriftItem>,
}

impl DriftReport {
    pub fn has_drift(&self) -> bool {
        !self.items.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.items.len()
    }
}

/// Compare two env maps (keys + values).
pub fn compare_env_maps(local: &BTreeMap<String, String>, remote: &BTreeMap<String, String>) -> DriftReport {
    let mut report = DriftReport::default();

    let local_keys: BTreeMap<_, _> = local.iter().collect();
    let remote_keys: BTreeMap<_, _> = remote.iter().collect();

    for (key, remote_val) in remote {
        match local.get(key) {
            None => report.items.push(DriftItem {
                key: key.clone(),
                kind: DriftKind::MissingLocal,
            }),
            Some(local_val) if local_val != remote_val => report.items.push(DriftItem {
                key: key.clone(),
                kind: DriftKind::ValueMismatch,
            }),
            _ => {}
        }
    }

    for key in local_keys.keys() {
        if !remote_keys.contains_key(key) {
            report.items.push(DriftItem {
                key: (*key).clone(),
                kind: DriftKind::MissingRemote,
            });
        }
    }

    report.items.sort_by(|a, b| a.key.cmp(&b.key));
    report
}

pub fn env_file_to_map(env: &EnvFile) -> BTreeMap<String, String> {
    env.vars.clone()
}

pub fn load_env_map_from_path(path: &Path) -> Result<BTreeMap<String, String>> {
    if !path.exists() {
        return Err(AppError::FileNotFound(path.to_path_buf()));
    }
    let env = parse_env_file(path)?;
    Ok(env_file_to_map(&env))
}

pub fn load_env_map_from_contents(contents: &str) -> BTreeMap<String, String> {
    env_file_to_map(&parse_env_contents(contents))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_value_mismatch() {
        let local = BTreeMap::from([
            ("API_KEY".to_string(), "local".to_string()),
            ("PORT".to_string(), "3000".to_string()),
        ]);
        let remote = BTreeMap::from([
            ("API_KEY".to_string(), "remote".to_string()),
            ("PORT".to_string(), "3000".to_string()),
        ]);
        let report = compare_env_maps(&local, &remote);
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].kind, DriftKind::ValueMismatch);
    }

    #[test]
    fn detects_missing_keys() {
        let local = BTreeMap::from([("A".to_string(), "1".to_string())]);
        let remote = BTreeMap::from([
            ("A".to_string(), "1".to_string()),
            ("B".to_string(), "2".to_string()),
        ]);
        let report = compare_env_maps(&local, &remote);
        assert!(report
            .items
            .iter()
            .any(|i| i.key == "B" && i.kind == DriftKind::MissingLocal));
    }
}
