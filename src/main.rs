mod api;
mod commands;

use clap::{Parser, Subcommand};
use commands::kenv::{GetArgs, KenvCommands};
use std::env;

#[derive(Parser)]
#[command(
    name = "karluiz-tool",
    about = "CLI for karluiz tools",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interact with the karluiz kenv secrets service
    Kenv {
        #[command(subcommand)]
        command: KenvCommands,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Kenv {
            command: KenvCommands::Get(args),
        } => run_kenv_get(args),
    }
}

fn run_kenv_get(args: GetArgs) {
    let api_key = args
        .api_key
        .or_else(|| env::var("KENV_API_KEY").ok())
        .unwrap_or_else(|| {
            eprintln!(
                "Error: API key is required. \
                 Pass --api-key or set the KENV_API_KEY environment variable."
            );
            std::process::exit(1);
        });

    match api::fetch_secrets(&args.app, &args.env, &api_key) {
        Ok(value) => {
            if args.json {
                println!("{}", serde_json::to_string_pretty(&value).unwrap_or_default());
            } else if let Some(obj) = value.as_object() {
                for (key, val) in obj {
                    let display = val.as_str().map(|s| s.to_owned()).unwrap_or_else(|| val.to_string());
                    println!("{key}={display}");
                }
            } else {
                println!("{value}");
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
