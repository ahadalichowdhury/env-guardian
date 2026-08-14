use std::fs;
use std::path::Path;

use owo_colors::OwoColorize;
use rpassword::read_password;

use crate::config::EnvGuardianConfig;
use crate::error::{AppError, Result};
use crate::vault::crypto::encrypt_bytes;

pub fn run(
    root: &Path,
    profile: Option<&str>,
    file: Option<&Path>,
    output: Option<&Path>,
) -> Result<()> {
    let config = EnvGuardianConfig::load(root)?;
    let profile_name = EnvGuardianConfig::resolve_profile(profile, &config);

    let input_path = file
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| config.env_path_for(root, &profile_name));
    let output_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| config.encrypted_path_for(root, &profile_name));

    if !input_path.exists() {
        return Err(AppError::FileNotFound(input_path));
    }

    println!("Profile: {}", profile_name.cyan());

    let plaintext = fs::read(&input_path)?;

    print!("Enter master password: ");
    let password = read_password()
        .map_err(|e| AppError::Other(format!("failed to read password: {}", e)))?;
    println!();

    if password.is_empty() {
        return Err(AppError::Other("password cannot be empty".to_string()));
    }

    print!("Confirm master password: ");
    let confirm = read_password()
        .map_err(|e| AppError::Other(format!("failed to read password: {}", e)))?;
    println!();

    if password != confirm {
        return Err(AppError::Other("passwords do not match".to_string()));
    }

    let encoded = encrypt_bytes(&plaintext, &password)?;
    fs::write(&output_path, encoded)?;

    println!(
        "{} Encrypted {} → {}",
        "✓".green().bold(),
        input_path.display().to_string().cyan(),
        output_path.display().to_string().cyan()
    );
    println!(
        "{} Keep your master password safe — it cannot be recovered",
        "·".yellow()
    );

    Ok(())
}
