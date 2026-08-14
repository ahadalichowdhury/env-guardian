use std::fs;
use std::path::Path;

use owo_colors::OwoColorize;

use crate::error::{AppError, Result};

const WORKFLOW_PATH: &str = ".github/workflows/env-guardian.yml";

const WORKFLOW_TEMPLATE: &str = r#"# EnvGuardian — environment consistency check (auto-generated)
name: EnvGuardian

on:
  push:
    branches: [main, master]
  pull_request:

jobs:
  env-consistency:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install env-guardian
        run: |
          if [ -f Cargo.toml ] && grep -q 'name = "env-guardian"' Cargo.toml 2>/dev/null; then
            cargo build --release --locked
            echo "$(pwd)/target/release" >> "$GITHUB_PATH"
          else
            cargo install env-guardian --path . --locked 2>/dev/null || \
            cargo install env-guardian --git https://github.com/env-guardian/env-guardian --locked
          fi

      - name: EnvGuardian check (strict)
        run: env-guardian check --strict --no-scan

      - name: EnvGuardian drift (snapshot)
        if: hashFiles('.envguardian.snapshot') != ''
        run: env-guardian drift check --snapshot .envguardian.snapshot
"#;

pub fn install_github_workflow(root: &Path, force: bool) -> Result<()> {
    let workflow_path = root.join(WORKFLOW_PATH);

    if workflow_path.exists() && !force {
        return Err(AppError::Other(format!(
            "{} already exists — use --force to overwrite",
            WORKFLOW_PATH
        )));
    }

    fs::create_dir_all(workflow_path.parent().unwrap())?;
    fs::write(&workflow_path, WORKFLOW_TEMPLATE)?;

    println!(
        "{} Created GitHub Actions workflow: {}",
        "✓".green().bold(),
        WORKFLOW_PATH.cyan()
    );
    println!(
        "{} Runs env-guardian check --strict on push/PR",
        "·".yellow()
    );
    println!(
        "{} Optional: add .envguardian.snapshot for drift check in CI",
        "·".yellow()
    );

    Ok(())
}

pub fn print_workflow() {
    println!("{}", WORKFLOW_TEMPLATE);
}
