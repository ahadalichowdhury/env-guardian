use std::collections::BTreeSet;
use std::path::Path;

use owo_colors::OwoColorize;

use crate::config::EnvGuardianConfig;
use crate::env::parser::{parse_env_file, EnvFile};
use crate::error::Result;
use crate::scanner::codebase::{scan_codebase, EnvUsage};

#[derive(Debug, Default)]
pub struct CheckReport {
    pub missing_in_env: Vec<String>,
    pub extra_in_env: Vec<String>,
    pub empty_in_env: Vec<String>,
    pub undefined_in_env: Vec<EnvUsage>,
    pub errors: usize,
    pub warnings: usize,
}

pub fn run(
    root: &Path,
    profile: Option<&str>,
    strict: bool,
    no_scan: bool,
) -> Result<bool> {
    run_internal(root, profile, strict, no_scan, false)
}

/// Run check without terminal output (for TUI).
pub fn run_quiet(
    root: &Path,
    profile: Option<&str>,
    strict: bool,
    no_scan: bool,
) -> Result<(bool, String)> {
    let passed = run_internal(root, profile, strict, no_scan, true)?;
    let summary = if passed {
        "Check passed".to_string()
    } else {
        "Check failed".to_string()
    };
    Ok((passed, summary))
}

fn run_internal(
    root: &Path,
    profile: Option<&str>,
    strict: bool,
    no_scan: bool,
    quiet: bool,
) -> Result<bool> {
    let config = EnvGuardianConfig::load(root)?;
    let profile_name = EnvGuardianConfig::resolve_profile(profile, &config);
    let files = config.files_for_profile(&profile_name);

    let env_path = config.env_path_for(root, &profile_name);
    let example_path = config.env_example_path_for(root, &profile_name);

    if !quiet {
        println!(
            "Profile: {} ({} ↔ {})",
            profile_name.cyan(),
            files.env,
            files.env_example
        );
    }

    let env_file = load_env_optional(&env_path)?;
    let example_file = if quiet {
        load_example_optional_quiet(&example_path)?
    } else {
        load_example_optional(&example_path, &files.env_example)?
    };

    let mut report = compare_files(&env_file, &example_file);

    if !no_scan && config.scan.enabled {
        let usages = scan_codebase(root, &config)?;
        let defined_keys: BTreeSet<String> = env_file
            .vars
            .keys()
            .chain(example_file.vars.keys())
            .cloned()
            .collect();

        for usage in usages {
            if !defined_keys.contains(&usage.key) {
                report.undefined_in_env.push(usage);
                report.errors += 1;
            }
        }
    }

    if !quiet {
        print_report(&report, &files.env, &files.env_example);
    }

    let failed = report.errors > 0 || (strict && report.warnings > 0);
    if !quiet {
        if failed {
            println!();
            println!(
                "{} {}",
                "✗".red().bold(),
                "EnvGuardian Check Failed".red().bold()
            );
            println!(
                "  Summary: {} error(s), {} warning(s)",
                report.errors.red(),
                report.warnings.yellow()
            );
        } else {
            println!();
            println!(
                "{} {}",
                "✓".green().bold(),
                "EnvGuardian Check Passed".green().bold()
            );
        }
    }

    Ok(!failed)
}

fn load_env_optional(path: &Path) -> Result<EnvFile> {
    if path.exists() {
        parse_env_file(path)
    } else {
        Ok(EnvFile::default())
    }
}

fn load_example_optional_quiet(path: &Path) -> Result<EnvFile> {
    if path.exists() {
        parse_env_file(path)
    } else {
        Ok(EnvFile::default())
    }
}

fn load_example_optional(path: &Path, label: &str) -> Result<EnvFile> {
    if path.exists() {
        parse_env_file(path)
    } else {
        println!(
            "{} {} not found — skipping example comparison",
            "·".yellow(),
            label
        );
        Ok(EnvFile::default())
    }
}

fn compare_files(env: &EnvFile, example: &EnvFile) -> CheckReport {
    let mut report = CheckReport::default();

    let example_keys: BTreeSet<_> = example.vars.keys().collect();
    let env_keys: BTreeSet<_> = env.vars.keys().collect();

    for key in example_keys.difference(&env_keys) {
        report.missing_in_env.push((*key).clone());
        report.errors += 1;
    }

    for key in env_keys.difference(&example_keys) {
        report.extra_in_env.push((*key).clone());
        report.warnings += 1;
    }

    for key in env_keys.intersection(&example_keys) {
        if env.is_empty_value(key) {
            report.empty_in_env.push((*key).clone());
            report.warnings += 1;
        }
    }

    report
}

fn print_report(report: &CheckReport, env_label: &str, example_label: &str) {
    if !report.missing_in_env.is_empty() {
        println!();
        println!(
            "  {} ({}):",
            "MISSING".red().bold(),
            report.missing_in_env.len()
        );
        for key in &report.missing_in_env {
            println!(
                "    • {} — in {}, not in {}",
                key.cyan(),
                example_label,
                env_label
            );
        }
    }

    if !report.undefined_in_env.is_empty() {
        println!();
        println!(
            "  {} ({}):",
            "UNDEFINED_IN_ENV".red().bold(),
            report.undefined_in_env.len()
        );
        for usage in &report.undefined_in_env {
            println!(
                "    • {} — used in {}:{}, not in {} or {}",
                usage.key.cyan(),
                usage.file.display(),
                usage.line,
                env_label,
                example_label
            );
        }
    }

    if !report.extra_in_env.is_empty() {
        println!();
        println!(
            "  {} ({}):",
            "EXTRA".yellow().bold(),
            report.extra_in_env.len()
        );
        for key in &report.extra_in_env {
            println!(
                "    • {} — in {}, not in {}",
                key.cyan(),
                env_label,
                example_label
            );
        }
    }

    if !report.empty_in_env.is_empty() {
        println!();
        println!(
            "  {} ({}):",
            "EMPTY".yellow().bold(),
            report.empty_in_env.len()
        );
        for key in &report.empty_in_env {
            println!(
                "    • {} — key present but value is empty",
                key.cyan()
            );
        }
    }
}
