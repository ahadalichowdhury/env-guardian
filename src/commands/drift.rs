use std::path::{Path, PathBuf};

use owo_colors::OwoColorize;

use crate::config::EnvGuardianConfig;
use crate::drift::compare::{compare_env_maps, load_env_map_from_path, DriftKind};
use crate::drift::sources::{fetch_aws_ssm_env, fetch_vercel_env, load_snapshot};
use crate::error::{AppError, Result};

pub struct DriftOptions {
    pub profile: Option<String>,
    pub snapshot: Option<PathBuf>,
    pub remote_env: Option<PathBuf>,
    pub vercel_project: Option<String>,
    pub vercel_team: Option<String>,
    pub aws_ssm_path: Option<String>,
    pub aws_region: Option<String>,
}

pub fn run(root: &Path, opts: DriftOptions) -> Result<bool> {
    let config = EnvGuardianConfig::load(root)?;
    let profile_name = EnvGuardianConfig::resolve_profile(opts.profile.as_deref(), &config);
    let local_path = config.env_path_for(root, &profile_name);

    if !local_path.exists() {
        return Err(AppError::FileNotFound(local_path));
    }

    let local = load_env_map_from_path(&local_path)?;

    let (remote_label, remote) = fetch_remote_map(root, &opts)?;

    println!(
        "Drift check — profile: {} | local: {} | remote: {}",
        profile_name.cyan(),
        local_path.display().to_string().yellow(),
        remote_label.cyan()
    );

    let report = compare_env_maps(&local, &remote);

    if report.items.is_empty() {
        println!(
            "{} No drift detected",
            "✓".green().bold()
        );
        return Ok(true);
    }

    print_drift_report(&report);

    println!();
    println!(
        "{} {} drift item(s) detected",
        "✗".red().bold(),
        report.error_count().to_string().red()
    );

    Ok(false)
}

pub fn snapshot(root: &Path, profile: Option<&str>, output: &Path) -> Result<()> {
    let config = EnvGuardianConfig::load(root)?;
    let profile_name = EnvGuardianConfig::resolve_profile(profile, &config);
    let local_path = config.env_path_for(root, &profile_name);

    if !local_path.exists() {
        return Err(AppError::FileNotFound(local_path));
    }

    std::fs::copy(&local_path, output)?;
    println!(
        "{} Snapshot saved: {} (profile: {})",
        "✓".green().bold(),
        output.display().to_string().cyan(),
        profile_name
    );
    println!(
        "{} Commit this file for CI drift checks (values are visible — use encrypted share for secrets)",
        "·".yellow()
    );
    Ok(())
}

fn fetch_remote_map(
    root: &Path,
    opts: &DriftOptions,
) -> Result<(String, std::collections::BTreeMap<String, String>)> {
    if let Some(path) = &opts.snapshot {
        let map = load_snapshot(path)?;
        return Ok((path.display().to_string(), map));
    }

    if let Some(path) = &opts.remote_env {
        let map = load_env_map_from_path(path)?;
        return Ok((path.display().to_string(), map));
    }

    if let Some(project) = &opts.vercel_project {
        let token = std::env::var("VERCEL_TOKEN")
            .map_err(|_| AppError::Other("VERCEL_TOKEN env var required for Vercel drift".to_string()))?;
        let map = fetch_vercel_env(project, opts.vercel_team.as_deref(), &token)?;
        return Ok((format!("vercel:{}", project), map));
    }

    if let Some(path_prefix) = &opts.aws_ssm_path {
        let map = fetch_aws_ssm_env(path_prefix, opts.aws_region.as_deref())?;
        return Ok((format!("aws-ssm:{}", path_prefix), map));
    }

    // Default: look for .envguardian.snapshot in project root
    let default_snapshot = root.join(".envguardian.snapshot");
    if default_snapshot.exists() {
        let map = load_snapshot(&default_snapshot)?;
        return Ok((default_snapshot.display().to_string(), map));
    }

    Err(AppError::Other(
        "no remote source — use --snapshot, --remote-env, --vercel-project, or --aws-ssm-path"
            .to_string(),
    ))
}

fn print_drift_report(report: &crate::drift::compare::DriftReport) {
    let missing_local: Vec<_> = report
        .items
        .iter()
        .filter(|i| i.kind == DriftKind::MissingLocal)
        .collect();
    let missing_remote: Vec<_> = report
        .items
        .iter()
        .filter(|i| i.kind == DriftKind::MissingRemote)
        .collect();
    let mismatches: Vec<_> = report
        .items
        .iter()
        .filter(|i| i.kind == DriftKind::ValueMismatch)
        .collect();

    if !missing_local.is_empty() {
        println!();
        println!(
            "  {} ({}):",
            "MISSING_LOCAL".red().bold(),
            missing_local.len()
        );
        for item in missing_local {
            println!(
                "    • {} — in remote, not in local .env",
                item.key.cyan()
            );
        }
    }

    if !missing_remote.is_empty() {
        println!();
        println!(
            "  {} ({}):",
            "MISSING_REMOTE".yellow().bold(),
            missing_remote.len()
        );
        for item in missing_remote {
            println!(
                "    • {} — in local .env, not in remote",
                item.key.cyan()
            );
        }
    }

    if !mismatches.is_empty() {
        println!();
        println!(
            "  {} ({}):",
            "VALUE_MISMATCH".red().bold(),
            mismatches.len()
        );
        for item in mismatches {
            println!(
                "    • {} — value differs (values hidden)",
                item.key.cyan()
            );
        }
    }
}
