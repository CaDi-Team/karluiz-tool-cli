mod api;
mod commands;
mod config;
mod migrate;

use clap::{Parser, Subcommand};
use commands::auth::{AuthCommand, KenvAuthCommand};
use commands::kenv::{KenvArgs, KenvCommands};

#[derive(Parser)]
#[command(name = "ktool", about = "CLI for karluiz tools", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Print the version string.
    Version,

    /// Show the karluiz 8-bit hero screen.
    Magic,

    /// Self-update to the latest release from GitHub.
    Update,

    /// Manage authentication credentials.
    Auth {
        #[command(subcommand)]
        cmd: AuthCommand,
    },

    /// Manage the kenv secrets service.
    ///
    /// Use --set-app / --set-env to save default context, then `ktool kenv list` to
    /// fetch secrets.
    Kenv(KenvArgs),
}

fn main() {
    // Run one-time migration from old config layout.
    migrate::run();

    let cli = Cli::parse();

    let result = match cli.command {
        None => {
            // No subcommand: print help.
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
            Ok(())
        }
        Some(Commands::Version) => {
            println!("ktool {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some(Commands::Magic) => {
            commands::cadi::run();
            Ok(())
        }
        Some(Commands::Update) => commands::update::run(),
        Some(Commands::Auth { cmd }) => run_auth(cmd),
        Some(Commands::Kenv(args)) => run_kenv(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// auth
// ---------------------------------------------------------------------------

fn run_auth(cmd: AuthCommand) -> Result<(), String> {
    match cmd {
        AuthCommand::Kenv { cmd } => match cmd {
            KenvAuthCommand::Login { token } => commands::auth::kenv_login(&token),
            KenvAuthCommand::Logout => commands::auth::kenv_logout(),
            KenvAuthCommand::Whoami => commands::auth::kenv_whoami(),
        },
        AuthCommand::Status => commands::auth::status(),
        AuthCommand::Logout => commands::auth::logout_all(),
    }
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
            "Config updated -- app: {}, env: {}.",
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
                    "Current context -- app: {}, env: {}",
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
    let token = commands::auth::load_kenv_token()?;

    let app = cfg
        .app
        .as_deref()
        .ok_or("No app set. Run `ktool kenv --set-app=<app>` first.")?;

    let env = cfg
        .env
        .as_deref()
        .ok_or("No env set. Run `ktool kenv --set-env=<env>` first.")?;

    let value = api::fetch_secrets(app, env, &token)?;

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_default()
        );
    } else if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            let plain = val
                .as_str()
                .map(|s| s.to_owned())
                .unwrap_or_else(|| val.to_string());
            println!("{key}={}", api::obfuscate(&plain));
        }
    } else {
        println!("{value}");
    }

    Ok(())
}
