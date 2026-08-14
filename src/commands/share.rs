use std::fs;
use std::path::{Path, PathBuf};

use owo_colors::OwoColorize;

use crate::config::EnvGuardianConfig;
use crate::error::{AppError, Result};
use crate::share::crypto::{
    create_share, generate_keypair, load_private_key, load_public_key, open_share,
};

pub fn keygen(output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    let pair = generate_keypair()?;

    let public_path = output_dir.join("env-guardian.pub");
    let private_path = output_dir.join("env-guardian.key");

    fs::write(&public_path, &pair.public)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::write(&private_path, &pair.private)?;
        let mut perms = fs::metadata(&private_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&private_path, perms)?;
    }
    #[cfg(not(unix))]
    fs::write(&private_path, &pair.private)?;

    println!(
        "{} Generated keypair in {}",
        "✓".green().bold(),
        output_dir.display().to_string().cyan()
    );
    println!("  Public:  {} (share with teammates)", public_path.display());
    println!(
        "  Private: {} (keep secret — never commit)",
        private_path.display().to_string().yellow()
    );

    Ok(())
}

pub fn create(
    root: &Path,
    profile: Option<&str>,
    recipient: &Path,
    input: Option<&Path>,
    output: Option<&Path>,
) -> Result<()> {
    let config = EnvGuardianConfig::load(root)?;
    let profile_name = EnvGuardianConfig::resolve_profile(profile, &config);

    let input_path = input
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| config.env_path_for(root, &profile_name));
    let output_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| root.join(format!(".env.{}.share", profile_name)));

    if !input_path.exists() {
        return Err(AppError::FileNotFound(input_path));
    }
    if !recipient.exists() {
        return Err(AppError::FileNotFound(recipient.to_path_buf()));
    }

    let recipient_contents = fs::read_to_string(recipient)?;
    let recipient_public = load_public_key(&recipient_contents)?;
    let plaintext = fs::read(&input_path)?;

    let share = create_share(&plaintext, &recipient_public, &profile_name)?;
    fs::write(&output_path, share)?;

    println!(
        "{} Created zero-knowledge share: {}",
        "✓".green().bold(),
        output_path.display().to_string().cyan()
    );
    println!(
        "{} Encrypted for recipient — only their private key can decrypt",
        "·".yellow()
    );
    println!("  Send {} to teammate (email, Slack, etc.)", output_path.display());

    Ok(())
}

pub fn open(share_path: &Path, private_key_path: &Path, output: Option<&Path>) -> Result<()> {
    if !share_path.exists() {
        return Err(AppError::FileNotFound(share_path.to_path_buf()));
    }
    if !private_key_path.exists() {
        return Err(AppError::FileNotFound(private_key_path.to_path_buf()));
    }

    let share_contents = fs::read_to_string(share_path)?;
    let private_contents = fs::read_to_string(private_key_path)?;
    let private = load_private_key(&private_contents)?;

    let plaintext = open_share(&share_contents, &private)?;

    let out = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        share_path
            .parent()
            .map(|p| p.join("decrypted.env"))
            .unwrap_or_else(|| PathBuf::from("decrypted.env"))
    });

    fs::write(&out, plaintext)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&out)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&out, perms)?;
    }

    println!(
        "{} Decrypted share → {}",
        "✓".green().bold(),
        out.display().to_string().cyan()
    );
    println!(
        "{} Do not commit decrypted secrets to git",
        "⚠".yellow().bold()
    );

    Ok(())
}
