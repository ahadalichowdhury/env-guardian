use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

pub const CONFIG_FILE: &str = ".envguardian.toml";
pub const DEFAULT_PROFILE: &str = "default";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_profile_name")]
    pub default_profile: String,
}

fn default_profile_name() -> String {
    DEFAULT_PROFILE.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesConfig {
    pub env: String,
    pub env_example: String,
    pub encrypted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub enabled: bool,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: String,
    pub kdf: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileFilesConfig {
    pub env: String,
    pub env_example: String,
    pub encrypted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvGuardianConfig {
    pub project: ProjectConfig,
    pub files: FilesConfig,
    pub scan: ScanConfig,
    pub encryption: EncryptionConfig,
    #[serde(default)]
    pub profiles: BTreeMap<String, ProfileFilesConfig>,
}

impl Default for EnvGuardianConfig {
    fn default() -> Self {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "development".to_string(),
            ProfileFilesConfig {
                env: ".env.development".to_string(),
                env_example: ".env.example.development".to_string(),
                encrypted: ".env.development.enc".to_string(),
            },
        );
        profiles.insert(
            "staging".to_string(),
            ProfileFilesConfig {
                env: ".env.staging".to_string(),
                env_example: ".env.example.staging".to_string(),
                encrypted: ".env.staging.enc".to_string(),
            },
        );
        profiles.insert(
            "production".to_string(),
            ProfileFilesConfig {
                env: ".env.production".to_string(),
                env_example: ".env.example.production".to_string(),
                encrypted: ".env.production.enc".to_string(),
            },
        );

        Self {
            project: ProjectConfig {
                name: "my-app".to_string(),
                default_profile: DEFAULT_PROFILE.to_string(),
            },
            files: FilesConfig {
                env: ".env".to_string(),
                env_example: ".env.example".to_string(),
                encrypted: ".env.enc".to_string(),
            },
            scan: ScanConfig {
                enabled: true,
                include: vec![
                    "src".to_string(),
                    "app".to_string(),
                    "lib".to_string(),
                ],
                exclude: vec![
                    "node_modules".to_string(),
                    "target".to_string(),
                    "dist".to_string(),
                    ".git".to_string(),
                    "vendor".to_string(),
                ],
                patterns: vec![
                    "process.env".to_string(),
                    "os.getenv".to_string(),
                    "std::env::var".to_string(),
                ],
            },
            encryption: EncryptionConfig {
                algorithm: "AES-256-GCM".to_string(),
                kdf: "argon2id".to_string(),
            },
            profiles,
        }
    }
}

impl EnvGuardianConfig {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(CONFIG_FILE);
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path)?;
        let config: EnvGuardianConfig = toml::from_str(&contents)
            .map_err(|e| AppError::Config(format!("failed to parse {}: {}", CONFIG_FILE, e)))?;
        Ok(config)
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let path = root.join(CONFIG_FILE);
        let contents = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("failed to serialize config: {}", e)))?;
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn resolve_profile(profile: Option<&str>, config: &Self) -> String {
        profile
            .map(|s| s.to_string())
            .unwrap_or_else(|| config.project.default_profile.clone())
    }

    pub fn files_for_profile(&self, profile: &str) -> FilesConfig {
        if profile == DEFAULT_PROFILE {
            return self.files.clone();
        }

        if let Some(p) = self.profiles.get(profile) {
            return FilesConfig {
                env: p.env.clone(),
                env_example: p.env_example.clone(),
                encrypted: p.encrypted.clone(),
            };
        }

        FilesConfig {
            env: format!(".env.{}", profile),
            env_example: format!(".env.example.{}", profile),
            encrypted: format!(".env.{}.enc", profile),
        }
    }

    pub fn profile_names(&self) -> Vec<String> {
        let mut names = vec![DEFAULT_PROFILE.to_string()];
        for key in self.profiles.keys() {
            names.push(key.clone());
        }
        names.sort();
        names.dedup();
        names
    }

    pub fn env_path_for(&self, root: &Path, profile: &str) -> PathBuf {
        root.join(&self.files_for_profile(profile).env)
    }

    pub fn env_example_path_for(&self, root: &Path, profile: &str) -> PathBuf {
        root.join(&self.files_for_profile(profile).env_example)
    }

    pub fn encrypted_path_for(&self, root: &Path, profile: &str) -> PathBuf {
        root.join(&self.files_for_profile(profile).encrypted)
    }

    pub fn env_path(&self, root: &Path) -> PathBuf {
        self.env_path_for(root, DEFAULT_PROFILE)
    }

    pub fn env_example_path(&self, root: &Path) -> PathBuf {
        self.env_example_path_for(root, DEFAULT_PROFILE)
    }

    pub fn encrypted_path(&self, root: &Path) -> PathBuf {
        self.encrypted_path_for(root, DEFAULT_PROFILE)
    }
}

/// Returns true if a staged file path should be blocked from git commit.
pub fn is_forbidden_env_commit(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    if name == ".env" {
        return true;
    }
    if !name.starts_with(".env") {
        return false;
    }
    if name.ends_with(".enc") {
        return false;
    }
    if name == ".env.example" || name.starts_with(".env.example.") {
        return false;
    }
    if name.starts_with(".env.") {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forbidden_env_detection() {
        assert!(is_forbidden_env_commit(".env"));
        assert!(is_forbidden_env_commit("config/.env"));
        assert!(is_forbidden_env_commit(".env.development"));
        assert!(!is_forbidden_env_commit(".env.example"));
        assert!(!is_forbidden_env_commit(".env.example.production"));
        assert!(!is_forbidden_env_commit(".env.enc"));
        assert!(!is_forbidden_env_commit(".env.production.enc"));
    }

    #[test]
    fn profile_file_resolution() {
        let config = EnvGuardianConfig::default();
        let dev = config.files_for_profile("development");
        assert_eq!(dev.env, ".env.development");
        assert_eq!(dev.encrypted, ".env.development.enc");

        let custom = config.files_for_profile("custom");
        assert_eq!(custom.env, ".env.custom");
    }
}
