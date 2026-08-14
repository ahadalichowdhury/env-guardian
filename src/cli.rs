use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "env-guardian",
    about = "EnvGuardian — secure .env management CLI (ConfigSync Pro)",
    version
)]
pub struct Cli {
    /// Project root directory (default: current directory)
    #[arg(long, global = true, default_value = ".")]
    pub root: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize EnvGuardian config in the project
    Init {
        /// Project name for config
        #[arg(long)]
        name: Option<String>,

        /// Also create a .env.example template
        #[arg(long)]
        with_example: bool,

        /// Create profile example files (development, staging, production)
        #[arg(long)]
        with_profiles: bool,

        /// Force overwrite existing config
        #[arg(long)]
        force: bool,
    },

    /// Check .env consistency against .env.example and codebase
    Check {
        /// Environment profile (default, development, staging, production, …)
        #[arg(long, short)]
        profile: Option<String>,

        /// Treat warnings as errors (for CI)
        #[arg(long)]
        strict: bool,

        /// Skip codebase scan
        #[arg(long)]
        no_scan: bool,
    },

    /// Encrypt .env file to .env.enc
    Encrypt {
        /// Environment profile
        #[arg(long, short)]
        profile: Option<String>,

        /// Input .env file path
        #[arg(long)]
        file: Option<PathBuf>,

        /// Output encrypted file path
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Decrypt .env.enc file to .env
    Decrypt {
        /// Environment profile
        #[arg(long, short)]
        profile: Option<String>,

        /// Input encrypted file path
        #[arg(long)]
        file: Option<PathBuf>,

        /// Output .env file path
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Git hook management (leakage prevention)
    Hook {
        #[command(subcommand)]
        action: HookCommands,
    },

    /// Interactive terminal UI for viewing and editing env vars
    Tui {
        /// Environment profile
        #[arg(long, short)]
        profile: Option<String>,
    },

    /// CI/CD integration (GitHub Actions)
    Ci {
        #[command(subcommand)]
        action: CiCommands,
    },

    /// Detect drift between local .env and remote/snapshot
    Drift {
        #[command(subcommand)]
        action: DriftCommands,
    },

    /// Zero-knowledge team sharing (E2E encrypted)
    Share {
        #[command(subcommand)]
        action: ShareCommands,
    },
}

#[derive(Subcommand)]
pub enum HookCommands {
    /// Install pre-commit hook in .git/hooks
    Install,

    /// Run hook checks (used by pre-commit hook)
    Run,

    /// Remove EnvGuardian pre-commit hook
    Uninstall,
}

#[derive(Subcommand)]
pub enum CiCommands {
    /// Install GitHub Actions workflow (.github/workflows/env-guardian.yml)
    Install {
        /// Overwrite existing workflow
        #[arg(long)]
        force: bool,
    },

    /// Print workflow YAML to stdout
    Print,
}

#[derive(Subcommand)]
pub enum DriftCommands {
    /// Compare local env against remote source or snapshot
    Check {
        /// Environment profile
        #[arg(long, short)]
        profile: Option<String>,

        /// Compare against local snapshot file
        #[arg(long)]
        snapshot: Option<PathBuf>,

        /// Compare against another local .env file
        #[arg(long)]
        remote_env: Option<PathBuf>,

        /// Vercel project ID or name
        #[arg(long)]
        vercel_project: Option<String>,

        /// Vercel team ID (optional)
        #[arg(long)]
        vercel_team: Option<String>,

        /// AWS SSM parameter path prefix (uses AWS CLI)
        #[arg(long)]
        aws_ssm_path: Option<String>,

        /// AWS region for SSM
        #[arg(long)]
        aws_region: Option<String>,
    },

    /// Save current local .env as drift snapshot for CI
    Snapshot {
        /// Environment profile
        #[arg(long, short)]
        profile: Option<String>,

        /// Output snapshot path
        #[arg(long, short, default_value = ".envguardian.snapshot")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum ShareCommands {
    /// Generate X25519 keypair for E2E sharing
    Keygen {
        /// Output directory for keys
        #[arg(long, short, default_value = ".")]
        output_dir: PathBuf,
    },

    /// Create encrypted share package for a recipient
    Create {
        /// Environment profile
        #[arg(long, short)]
        profile: Option<String>,

        /// Recipient public key file (.pub)
        #[arg(long)]
        recipient: PathBuf,

        /// Input file (default: profile .env)
        #[arg(long)]
        input: Option<PathBuf>,

        /// Output share package path
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Decrypt a share package with your private key
    Open {
        /// Share package file
        #[arg(long)]
        share: PathBuf,

        /// Your private key file
        #[arg(long)]
        key: PathBuf,

        /// Output decrypted file path
        #[arg(long)]
        output: Option<PathBuf>,
    },
}
