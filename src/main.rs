use clap::Parser;
use env_guardian::cli::{CiCommands, Cli, Commands, DriftCommands, HookCommands, ShareCommands};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let root = cli.root.canonicalize().unwrap_or(cli.root);

    let result = match cli.command {
        Commands::Init {
            name,
            with_example,
            with_profiles,
            force,
        } => env_guardian::commands::init::run(
            &root,
            name,
            with_example,
            with_profiles,
            force,
        ),
        Commands::Check {
            profile,
            strict,
            no_scan,
        } => match env_guardian::commands::check::run(&root, profile.as_deref(), strict, no_scan)
        {
            Ok(passed) if passed => Ok(()),
            Ok(_) => std::process::exit(1),
            Err(e) => Err(e),
        },
        Commands::Encrypt {
            profile,
            file,
            output,
        } => env_guardian::commands::encrypt::run(
            &root,
            profile.as_deref(),
            file.as_deref(),
            output.as_deref(),
        ),
        Commands::Decrypt {
            profile,
            file,
            output,
        } => env_guardian::commands::decrypt::run(
            &root,
            profile.as_deref(),
            file.as_deref(),
            output.as_deref(),
        ),
        Commands::Hook { action } => match action {
            HookCommands::Install => env_guardian::commands::hook::install(&root),
            HookCommands::Run => env_guardian::commands::hook::run(&root),
            HookCommands::Uninstall => env_guardian::commands::hook::uninstall(&root),
        },
        Commands::Tui { profile } => env_guardian::commands::tui::run(&root, profile.as_deref()),
        Commands::Ci { action } => match action {
            CiCommands::Install { force } => {
                env_guardian::commands::ci::install_github_workflow(&root, force)
            }
            CiCommands::Print => {
                env_guardian::commands::ci::print_workflow();
                Ok(())
            }
        },
        Commands::Drift { action } => match action {
            DriftCommands::Check {
                profile,
                snapshot,
                remote_env,
                vercel_project,
                vercel_team,
                aws_ssm_path,
                aws_region,
            } => {
                let opts = env_guardian::commands::drift::DriftOptions {
                    profile,
                    snapshot,
                    remote_env,
                    vercel_project,
                    vercel_team,
                    aws_ssm_path,
                    aws_region,
                };
                match env_guardian::commands::drift::run(&root, opts) {
                    Ok(passed) if passed => Ok(()),
                    Ok(_) => std::process::exit(1),
                    Err(e) => Err(e),
                }
            }
            DriftCommands::Snapshot {
                profile,
                output,
            } => env_guardian::commands::drift::snapshot(&root, profile.as_deref(), &output),
        },
        Commands::Share { action } => match action {
            ShareCommands::Keygen { output_dir } => {
                env_guardian::commands::share::keygen(&output_dir)
            }
            ShareCommands::Create {
                profile,
                recipient,
                input,
                output,
            } => env_guardian::commands::share::create(
                &root,
                profile.as_deref(),
                &recipient,
                input.as_deref(),
                output.as_deref(),
            ),
            ShareCommands::Open {
                share,
                key,
                output,
            } => env_guardian::commands::share::open(&share, &key, output.as_deref()),
        },
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}
