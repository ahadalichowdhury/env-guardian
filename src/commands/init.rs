use std::fs;
use std::path::Path;

use owo_colors::OwoColorize;

use crate::config::{CONFIG_FILE, EnvGuardianConfig};
use crate::error::{AppError, Result};

const EXAMPLE_TEMPLATE: &str = "# EnvGuardian .env.example template\n\
# Copy to matching .env and fill in values\n\
DATABASE_URL=\n\
API_KEY=\n\
PORT=3000\n";

pub fn run(
    root: &Path,
    name: Option<String>,
    with_example: bool,
    with_profiles: bool,
    force: bool,
) -> Result<()> {
    let config_path = root.join(CONFIG_FILE);

    if config_path.exists() && !force {
        return Err(AppError::Other(format!(
            "{} already exists — use --force to overwrite",
            CONFIG_FILE
        )));
    }

    let mut config = EnvGuardianConfig::default();
    if let Some(n) = name {
        config.project.name = n;
    } else if let Ok(cwd_name) = root.canonicalize() {
        if let Some(dir_name) = cwd_name.file_name() {
            config.project.name = dir_name.to_string_lossy().to_string();
        }
    }

    config.save(root)?;
    println!(
        "{} Created {}",
        "✓".green(),
        CONFIG_FILE.cyan()
    );

    if with_example {
        write_example_if_missing(root, &config.files.env_example)?;
    }

    if with_profiles {
        for (name, profile) in &config.profiles {
            write_example_if_missing(root, &profile.env_example)?;
            println!(
                "{} Profile {} → {} / {}",
                "✓".green(),
                name.cyan(),
                profile.env,
                profile.env_example
            );
        }
    }

    suggest_gitignore(root)?;

    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. env-guardian check              — validate env keys");
    println!("  2. env-guardian check -p development — check a profile");
    println!("  3. env-guardian hook install       — block .env git commits");
    println!("  4. env-guardian tui                — interactive editor");
    println!("  5. env-guardian encrypt            — lock .env as .env.enc");

    Ok(())
}

fn write_example_if_missing(root: &Path, rel_path: &str) -> Result<()> {
    let path = root.join(rel_path);
    if path.exists() {
        println!(
            "{} {} already exists — skipped",
            "·".yellow(),
            rel_path
        );
        return Ok(());
    }
    fs::write(&path, EXAMPLE_TEMPLATE)?;
    println!("{} Created {}", "✓".green(), rel_path.cyan());
    Ok(())
}

fn suggest_gitignore(root: &Path) -> Result<()> {
    let gitignore = root.join(".gitignore");
    if !gitignore.exists() {
        return Ok(());
    }

    let contents = fs::read_to_string(&gitignore)?;
    let needs_env = !contents.lines().any(|l| l.trim() == ".env");

    if needs_env {
        let additions = "\n.env\n.env.*\n!.env.example\n!.env.*.enc\n";
        fs::write(&gitignore, format!("{}{}", contents.trim_end(), additions))?;
        println!("{} Updated .gitignore for .env patterns", "✓".green());
    }

    Ok(())
}
