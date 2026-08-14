use std::collections::BTreeMap;
use std::process::Command;

use serde::Deserialize;

use crate::drift::compare::load_env_map_from_contents;
use crate::error::{AppError, Result};

#[derive(Debug, Deserialize)]
struct VercelEnvEntry {
    key: String,
    value: Option<String>,
    #[serde(rename = "type")]
    env_type: Option<String>,
}

/// Fetch Vercel project env vars (keys + decrypted values when available).
pub fn fetch_vercel_env(
    project_id: &str,
    team_id: Option<&str>,
    token: &str,
) -> Result<BTreeMap<String, String>> {
    let url = if let Some(team) = team_id {
        format!(
            "https://api.vercel.com/v9/projects/{}/env?teamId={}",
            project_id, team
        )
    } else {
        format!("https://api.vercel.com/v9/projects/{}/env", project_id)
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Network(format!("http client: {}", e)))?;

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .map_err(|e| AppError::Network(format!("vercel request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(AppError::Network(format!(
            "vercel API {}: {}",
            status, body
        )));
    }

    let entries: Vec<VercelEnvEntry> = response
        .json()
        .map_err(|e| AppError::Parse(format!("vercel response parse: {}", e)))?;

    let mut map = BTreeMap::new();
    for entry in entries {
        // Skip system vars without values; compare keys that have values
        if let Some(value) = entry.value {
            map.insert(entry.key, value);
        } else if entry.env_type.as_deref() == Some("plain") {
            map.insert(entry.key, String::new());
        }
    }

    Ok(map)
}

/// Fetch AWS SSM parameters under a path via AWS CLI (requires `aws` installed).
pub fn fetch_aws_ssm_env(path_prefix: &str, region: Option<&str>) -> Result<BTreeMap<String, String>> {
    let mut cmd = Command::new("aws");
    cmd.args([
        "ssm",
        "get-parameters-by-path",
        "--path",
        path_prefix,
        "--recursive",
        "--with-decryption",
        "--output",
        "json",
    ]);
    if let Some(r) = region {
        cmd.args(["--region", r]);
    }

    let output = cmd
        .output()
        .map_err(|e| AppError::Other(format!("failed to run aws CLI: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Other(format!("aws ssm failed: {}", stderr)));
    }

#[derive(Deserialize)]
struct AwsResponse {
    #[serde(rename = "Parameters")]
    parameters: Vec<AwsParam>,
}
#[derive(Deserialize)]
struct AwsParam {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Value")]
    value: String,
}

    let resp: AwsResponse = serde_json::from_slice(&output.stdout)
        .map_err(|e| AppError::Parse(format!("aws output parse: {}", e)))?;

    let mut map = BTreeMap::new();
    for param in resp.parameters {
        let key = param
            .name
            .rsplit('/')
            .next()
            .unwrap_or(&param.name)
            .to_string();
        map.insert(key, param.value);
    }

    Ok(map)
}

/// Parse snapshot file (standard .env format).
pub fn load_snapshot(path: &std::path::Path) -> Result<BTreeMap<String, String>> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| AppError::Io(e))?;
    Ok(load_env_map_from_contents(&contents))
}
