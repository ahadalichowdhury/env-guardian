use std::fs;
use std::path::Path;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use owo_colors::OwoColorize;
use crate::config::is_forbidden_env_commit;
use crate::error::{AppError, Result};

const HOOK_MARKER: &str = "# env-guardian hook";

pub fn install(root: &Path) -> Result<()> {
    let git_dir = root.join(".git");
    if !git_dir.is_dir() {
        return Err(AppError::Other(
            "not a git repository — run from project root".to_string(),
        ));
    }

    let hook_path = git_dir.join("hooks/pre-commit");
    let hook_body = format!(
        r#"#!/bin/sh
{marker}
ROOT="$(git rev-parse --show-toplevel)"
if command -v env-guardian >/dev/null 2>&1; then
  exec env-guardian hook run --root "$ROOT"
elif command -v config-sync >/dev/null 2>&1; then
  exec config-sync hook run --root "$ROOT"
else
  echo "env-guardian: binary not found in PATH — skipping hook"
  exit 0
fi
"#,
        marker = HOOK_MARKER
    );

    fs::write(&hook_path, hook_body)?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    println!(
        "{} Installed pre-commit hook at {}",
        "✓".green().bold(),
        hook_path.display().to_string().cyan()
    );
    println!(
        "{} Blocks committing plaintext .env files (allows .env.example and .env.enc)",
        "·".yellow()
    );

    Ok(())
}

pub fn uninstall(root: &Path) -> Result<()> {
    let hook_path = root.join(".git/hooks/pre-commit");
    if !hook_path.exists() {
        println!("{} No pre-commit hook found", "·".yellow());
        return Ok(());
    }

    let contents = fs::read_to_string(&hook_path)?;
    if !contents.contains(HOOK_MARKER) {
        return Err(AppError::Other(
            "pre-commit hook exists but was not installed by env-guardian — remove manually"
                .to_string(),
        ));
    }

    fs::remove_file(&hook_path)?;
    println!(
        "{} Removed pre-commit hook",
        "✓".green().bold()
    );
    Ok(())
}

pub fn run(root: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only", "--diff-filter=ACM"])
        .current_dir(root)
        .output()
        .map_err(|e| AppError::Other(format!("failed to run git: {}", e)))?;

    if !output.status.success() {
        return Err(AppError::Other("git diff --cached failed".to_string()));
    }

    let staged = String::from_utf8_lossy(&output.stdout);
    let forbidden: Vec<&str> = staged
        .lines()
        .map(str::trim)
        .filter(|p| !p.is_empty() && is_forbidden_env_commit(p))
        .collect();

    if forbidden.is_empty() {
        return Ok(());
    }

    eprintln!();
    eprintln!("{}", "✗ EnvGuardian pre-commit hook blocked commit".red().bold());
    eprintln!("  Plaintext .env files must not be committed:");
    for path in forbidden {
        eprintln!("    • {}", path);
    }
    eprintln!();
    eprintln!("  Commit .env.example or encrypted .env.enc instead.");
    eprintln!("  Encrypt with: env-guardian encrypt");

    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_marker_constant() {
        assert!(HOOK_MARKER.contains("env-guardian"));
    }
}
