use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use owo_colors::OwoColorize;
use rpassword::read_password;

use crate::config::EnvGuardianConfig;
use crate::error::{AppError, Result};
use crate::vault::crypto::decrypt_bytes;

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
        .unwrap_or_else(|| config.encrypted_path_for(root, &profile_name));
    let output_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| config.env_path_for(root, &profile_name));

    if !input_path.exists() {
        return Err(AppError::FileNotFound(input_path));
    }

    println!("Profile: {}", profile_name.cyan());

    let encoded = fs::read_to_string(&input_path)?;

    print!("Enter master password: ");
    let password = read_password()
        .map_err(|e| AppError::Other(format!("failed to read password: {}", e)))?;
    println!();

    let plaintext = decrypt_bytes(&encoded, &password)?;
    fs::write(&output_path, plaintext)?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&output_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&output_path, perms)?;
    }

    println!(
        "{} Decrypted {} → {}",
        "✓".green().bold(),
        input_path.display().to_string().cyan(),
        output_path.display().to_string().cyan()
    );
    println!(
        "{} Do not commit {} to git",
        "⚠".yellow().bold(),
        output_path.display().to_string().yellow()
    );

    Ok(())
}
