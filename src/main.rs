mod api;
mod commands;
mod config;

use clap::{Parser, Subcommand};
use commands::kenv::{KenvArgs, KenvCommands};

#[derive(Parser)]
#[command(
    name = "ktool",
    about = "CLI for karluiz tools",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authenticate with the karluiz API — saves your token to the local config.
    Login,

    /// Manage the kenv secrets service.
    ///
    /// Use --set-app / --set-env to save default context, then `ktool kenv list` to
    /// fetch secrets.
    Kenv(KenvArgs),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Login => run_login(),
        Commands::Kenv(args) => run_kenv(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// login
// ---------------------------------------------------------------------------

fn run_login() -> Result<(), String> {
    let token = rpassword::prompt_password("Enter your KENV API token: ")
        .map_err(|e| format!("Failed to read token: {e}"))?;

    let token = token.trim().to_string();
    if token.is_empty() {
        return Err("Token cannot be empty.".to_string());
    }

    let mut cfg = config::load()?;
    cfg.token = Some(token);
    config::save(&cfg)?;

    println!("✓ Token saved to {}.", config::config_path()?.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// kenv
// ---------------------------------------------------------------------------

fn run_kenv(args: KenvArgs) -> Result<(), String> {
    let mut cfg = config::load()?;
    let mut updated = false;

    if let Some(app) = args.set_app {
        cfg.app = Some(app);
        updated = true;
    }
    if let Some(env) = args.set_env {
        cfg.env = Some(env);
        updated = true;
    }
    if updated {
        config::save(&cfg)?;
        println!(
            "✓ Config updated — app: {}, env: {}.",
            cfg.app.as_deref().unwrap_or("(not set)"),
            cfg.env.as_deref().unwrap_or("(not set)"),
        );
    }

    match args.command {
        Some(KenvCommands::List(list_args)) => run_kenv_list(&cfg, list_args.json),
        None => {
            if !updated {
                // No subcommand and no flags: print current context.
                println!(
                    "Current context — app: {}, env: {}",
                    cfg.app.as_deref().unwrap_or("(not set)"),
                    cfg.env.as_deref().unwrap_or("(not set)"),
                );
                println!("Run `ktool kenv list` to fetch secrets.");
            }
            Ok(())
        }
    }
}

fn run_kenv_list(cfg: &config::Config, as_json: bool) -> Result<(), String> {
    let token = cfg
        .token
        .as_deref()
        .ok_or("No token found. Run `ktool login` first.")?;

    let app = cfg
        .app
        .as_deref()
        .ok_or("No app set. Run `ktool kenv --set-app=<app>` first.")?;

    let env = cfg
        .env
        .as_deref()
        .ok_or("No env set. Run `ktool kenv --set-env=<env>` first.")?;

    let value = api::fetch_secrets(app, env, token)?;

    if as_json {
        println!("{}", serde_json::to_string_pretty(&value).unwrap_or_default());
    } else if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            let plain = val.as_str().map(|s| s.to_owned()).unwrap_or_else(|| val.to_string());
            println!("{key}={}", api::obfuscate(&plain));
        }
    } else {
        println!("{value}");
    }

    Ok(())
}
