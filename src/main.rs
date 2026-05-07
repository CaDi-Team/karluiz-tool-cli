mod api;
mod commands;
mod config;
mod localrc;
mod migrate;

use clap::{Parser, Subcommand};
use commands::auth::{AuthCommand, KenvAuthCommand};
use commands::kenv::{
    AppsCommands, ContextCommands, EnvsCommands, KenvArgs, KenvCommands, VarsCommands,
};
use config::{ContextSource, ResolvedContext};

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
    /// Use --set-project / --set-app / --set-env to persist default context values,
    /// or drop a `ktool.toml` in your project directory for per-repo configuration.
    /// Then run `ktool kenv list` to fetch secrets.
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
// kenv — top-level dispatcher
// ---------------------------------------------------------------------------

fn run_kenv(args: KenvArgs) -> Result<(), String> {
    let mut cfg = config::load()?;
    let mut updated = false;

    // --set-project / --set-app / --set-env: update the active context or flat defaults.
    if let Some(project) = args.set_project {
        set_context_field(&mut cfg, |ctx| ctx.project = Some(project));
        updated = true;
    }
    if let Some(app) = args.set_app {
        set_context_field(&mut cfg, |ctx| ctx.app = Some(app));
        updated = true;
    }
    if let Some(env) = args.set_env {
        set_context_field(&mut cfg, |ctx| ctx.env = Some(env));
        updated = true;
    }
    if updated {
        config::save(&cfg)?;
        let resolved = config::resolve_context(&cfg);
        println!(
            "Config updated -- project: {}, app: {}, env: {}.",
            resolved.project.as_deref().unwrap_or("(not set)"),
            resolved.app.as_deref().unwrap_or("(not set)"),
            resolved.env.as_deref().unwrap_or("(not set)"),
        );
    }

    match args.command {
        Some(KenvCommands::List(list_args)) => {
            run_kenv_list(&cfg, list_args.json, list_args.detailed)
        }
        Some(KenvCommands::Apps { cmd }) => {
            let token = commands::auth::load_kenv_token()?;
            let resolved = config::resolve_context(&cfg);
            run_kenv_apps(&resolved, cmd, &token)
        }
        Some(KenvCommands::Envs { cmd }) => {
            let token = commands::auth::load_kenv_token()?;
            let resolved = config::resolve_context(&cfg);
            run_kenv_envs(&resolved, cmd, &token)
        }
        Some(KenvCommands::Vars { cmd }) => {
            let token = commands::auth::load_kenv_token()?;
            let resolved = config::resolve_context(&cfg);
            run_kenv_vars(&resolved, cmd, &token)
        }
        Some(KenvCommands::Context { cmd }) => run_kenv_context(&mut cfg, cmd),
        Some(KenvCommands::Init(init_args)) => {
            let resolved = config::resolve_context(&cfg);
            run_kenv_init(&resolved, init_args)
        }
        None => {
            if !updated {
                // No subcommand and no flags: show current context.
                let resolved = config::resolve_context(&cfg);
                print_resolved_context(&resolved);
            }
            Ok(())
        }
    }
}

/// Apply a mutation to the active named context, or to the flat defaults if no
/// context is active. This is how --set-project/app/env work.
fn set_context_field(cfg: &mut config::Config, f: impl FnOnce(&mut config::Context)) {
    if let Some(name) = cfg.current_context.clone() {
        let ctx = cfg.contexts.entry(name).or_default();
        f(ctx);
    } else {
        // No named context: update the flat legacy fields directly.
        let mut dummy = config::Context {
            project: cfg.project.clone(),
            app: cfg.app.clone(),
            env: cfg.env.clone(),
        };
        f(&mut dummy);
        cfg.project = dummy.project;
        cfg.app = dummy.app;
        cfg.env = dummy.env;
    }
}

/// Display a resolved context with its source information.
fn print_resolved_context(resolved: &ResolvedContext) {
    let source_label = match &resolved.source {
        ContextSource::LocalFile(path) => format!("local file ({})", path.display()),
        ContextSource::NamedContext(name) => format!("context '{name}'"),
        ContextSource::GlobalDefaults => "global defaults".to_string(),
    };
    println!("Active context source: {source_label}");
    println!(
        "  project : {}",
        resolved.project.as_deref().unwrap_or("(not set)")
    );
    println!(
        "  app     : {}",
        resolved.app.as_deref().unwrap_or("(not set)")
    );
    println!(
        "  env     : {}",
        resolved.env.as_deref().unwrap_or("(not set)")
    );
    println!();
    println!("Run `ktool kenv list` to fetch secrets.");
    println!("Run `ktool kenv context --help` to manage named contexts.");
    println!("Run `ktool kenv init` to create a local ktool.toml.");
}

// ---------------------------------------------------------------------------
// kenv list
// ---------------------------------------------------------------------------

fn run_kenv_list(cfg: &config::Config, as_json: bool, detailed: bool) -> Result<(), String> {
    let token = commands::auth::load_kenv_token()?;
    let resolved = config::resolve_context(cfg);

    let project = resolved
        .project
        .as_deref()
        .ok_or("No project set. Run `ktool kenv --set-project=<slug>` or create a ktool.toml.")?;

    let app = resolved
        .app
        .as_deref()
        .ok_or("No app set. Run `ktool kenv --set-app=<app>` first.")?;

    let env = resolved
        .env
        .as_deref()
        .ok_or("No env set. Run `ktool kenv --set-env=<env>` first.")?;

    let value = api::fetch_secrets(project, app, env, detailed, &token)?;

    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_default()
        );
    } else if detailed {
        // Detailed format: array of {key, value, shared}.
        if let Some(arr) = value.as_array() {
            for item in arr {
                let key = item.get("key").and_then(|v| v.as_str()).unwrap_or("?");
                let val = item
                    .get("value")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let shared = item
                    .get("shared")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let tag = if shared { " [shared]" } else { "" };
                println!("{key}={}{tag}", api::obfuscate(val));
            }
        } else {
            println!("{value}");
        }
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

// ---------------------------------------------------------------------------
// kenv apps
// ---------------------------------------------------------------------------

fn run_kenv_apps(resolved: &ResolvedContext, cmd: AppsCommands, token: &str) -> Result<(), String> {
    let project = resolved
        .project
        .as_deref()
        .ok_or("No project set. Run `ktool kenv --set-project=<slug>` or create a ktool.toml.")?;

    match cmd {
        AppsCommands::List => {
            let apps = api::list_apps(project, token)?;
            if apps.is_empty() {
                println!("No apps found in project '{project}'.");
            } else {
                println!("Apps in project '{project}':");
                for app in &apps {
                    let env_names: Vec<&str> =
                        app.environments.iter().map(|e| e.name.as_str()).collect();
                    if env_names.is_empty() {
                        println!("  {}", app.name);
                    } else {
                        println!("  {} (envs: {})", app.name, env_names.join(", "));
                    }
                }
            }
            Ok(())
        }
        AppsCommands::Create { name } => {
            let app = api::create_app(project, &name, token)?;
            println!("App '{}' created in project '{project}'.", app.name);
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// kenv envs
// ---------------------------------------------------------------------------

fn run_kenv_envs(resolved: &ResolvedContext, cmd: EnvsCommands, token: &str) -> Result<(), String> {
    match cmd {
        EnvsCommands::List(args) => {
            let project = args
                .project
                .as_deref()
                .or(resolved.project.as_deref())
                .ok_or(
                    "No project set. Use --project or set one via `ktool kenv --set-project`.",
                )?;
            let app = args
                .app
                .as_deref()
                .or(resolved.app.as_deref())
                .ok_or("No app set. Use --app or set one via `ktool kenv --set-app`.")?;

            let envs = api::list_environments(project, app, token)?;
            if envs.is_empty() {
                println!("No environments found for app '{app}' in project '{project}'.");
            } else {
                println!("Environments for '{project}/{app}':");
                for env in &envs {
                    println!("  {} ({} variable(s))", env.name, env.variable_count);
                }
            }
            Ok(())
        }

        EnvsCommands::Create(args) => {
            let project = args
                .project
                .as_deref()
                .or(resolved.project.as_deref())
                .ok_or(
                    "No project set. Use --project or set one via `ktool kenv --set-project`.",
                )?;
            let app = args
                .app
                .as_deref()
                .or(resolved.app.as_deref())
                .ok_or("No app set. Use --app or set one via `ktool kenv --set-app`.")?;

            let env = api::create_environment(project, app, &args.name, token)?;
            println!("Environment '{}' created in '{project}/{app}'.", env.name);
            Ok(())
        }

        EnvsCommands::Delete(args) => {
            let project = args
                .project
                .as_deref()
                .or(resolved.project.as_deref())
                .ok_or(
                    "No project set. Use --project or set one via `ktool kenv --set-project`.",
                )?;
            let app = args
                .app
                .as_deref()
                .or(resolved.app.as_deref())
                .ok_or("No app set. Use --app or set one via `ktool kenv --set-app`.")?;

            api::delete_environment(project, app, &args.name, token)?;
            println!(
                "Environment '{}' deleted from '{project}/{app}'.",
                args.name
            );
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// kenv vars
// ---------------------------------------------------------------------------

fn run_kenv_vars(resolved: &ResolvedContext, cmd: VarsCommands, token: &str) -> Result<(), String> {
    match cmd {
        VarsCommands::Set(args) => {
            let project = args
                .project
                .as_deref()
                .or(resolved.project.as_deref())
                .ok_or(
                    "No project set. Use --project or set one via `ktool kenv --set-project`.",
                )?;
            let app = args
                .app
                .as_deref()
                .or(resolved.app.as_deref())
                .ok_or("No app set. Use --app or set one via `ktool kenv --set-app`.")?;
            let env = args
                .env
                .as_deref()
                .or(resolved.env.as_deref())
                .ok_or("No env set. Use --env or set one via `ktool kenv --set-env`.")?;

            // Parse all KEY=VALUE pairs, splitting only on the first `=`.
            let pairs: Vec<(String, String)> = args
                .vars
                .iter()
                .map(|s| parse_key_value(s))
                .collect::<Result<_, _>>()?;

            // Confirmation prompt (skipped with -y / --yes).
            if !args.yes {
                println!(
                    "This will set {} variable(s) in {project}/{app}/{env}:",
                    pairs.len()
                );
                for (k, _) in &pairs {
                    println!("  {k}");
                }
                println!();
                println!("Existing keys will be overwritten.");
                if !commands::common::confirm("Continue?") {
                    println!("Aborted.");
                    return Ok(());
                }
            }

            let result = api::upsert_variables(project, app, env, &pairs, token)?;
            println!(
                "Done. {} variable(s) created, {} updated.",
                result.created, result.updated
            );
            println!(
                "All keys in {project}/{app}/{env} ({}): {}",
                result.variables.len(),
                result.variables.join(", ")
            );
            Ok(())
        }

        VarsCommands::Delete(args) => {
            let project = args
                .project
                .as_deref()
                .or(resolved.project.as_deref())
                .ok_or(
                    "No project set. Use --project or set one via `ktool kenv --set-project`.",
                )?;
            let app = args
                .app
                .as_deref()
                .or(resolved.app.as_deref())
                .ok_or("No app set. Use --app or set one via `ktool kenv --set-app`.")?;
            let env = args
                .env
                .as_deref()
                .or(resolved.env.as_deref())
                .ok_or("No env set. Use --env or set one via `ktool kenv --set-env`.")?;

            api::delete_variable(project, app, env, &args.key, token)?;
            println!(
                "Variable '{}' deleted from {project}/{app}/{env}.",
                args.key
            );
            Ok(())
        }
    }
}

/// Parse a `KEY=VALUE` string, splitting on the first `=`.
///
/// The key is uppercased; the value preserves its original casing (including
/// any `=` characters that appear after the first delimiter).
fn parse_key_value(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("Invalid KEY=VALUE pair '{s}': missing '='"))?;
    let key = s[..pos].trim().to_uppercase();
    if key.is_empty() {
        return Err(format!("Invalid KEY=VALUE pair '{s}': key is empty"));
    }
    let value = s[pos + 1..].to_string();
    Ok((key, value))
}

// ---------------------------------------------------------------------------
// kenv context
// ---------------------------------------------------------------------------

fn run_kenv_context(cfg: &mut config::Config, cmd: ContextCommands) -> Result<(), String> {
    match cmd {
        ContextCommands::List => {
            if cfg.contexts.is_empty() {
                println!("No named contexts. Create one with `ktool kenv context set <name>`.");
            } else {
                println!("Named contexts:");
                for (name, ctx) in &cfg.contexts {
                    let active = cfg
                        .current_context
                        .as_deref()
                        .map(|c| c == name)
                        .unwrap_or(false);
                    let marker = if active { "* " } else { "  " };
                    println!(
                        "{marker}{name}  project={} app={} env={}",
                        ctx.project.as_deref().unwrap_or("-"),
                        ctx.app.as_deref().unwrap_or("-"),
                        ctx.env.as_deref().unwrap_or("-"),
                    );
                }
            }
            Ok(())
        }

        ContextCommands::Current => {
            let resolved = config::resolve_context(cfg);
            print_resolved_context(&resolved);
            Ok(())
        }

        ContextCommands::Use { name } => {
            if !cfg.contexts.contains_key(&name) {
                return Err(format!(
                    "Context '{name}' not found. Create it with `ktool kenv context set {name}`."
                ));
            }
            cfg.current_context = Some(name.clone());
            config::save(cfg)?;
            println!("Switched to context '{name}'.");
            Ok(())
        }

        ContextCommands::Set(args) => {
            let existing = cfg.contexts.entry(args.name.clone()).or_default();
            if args.project.is_some() {
                existing.project = args.project;
            }
            if args.app.is_some() {
                existing.app = args.app;
            }
            if args.env.is_some() {
                existing.env = args.env;
            }
            config::save(cfg)?;
            let ctx = cfg.contexts.get(&args.name).unwrap();
            println!("Context '{}' saved:", args.name);
            println!(
                "  project : {}",
                ctx.project.as_deref().unwrap_or("(not set)")
            );
            println!("  app     : {}", ctx.app.as_deref().unwrap_or("(not set)"));
            println!("  env     : {}", ctx.env.as_deref().unwrap_or("(not set)"));
            Ok(())
        }

        ContextCommands::Delete { name } => {
            if cfg.contexts.remove(&name).is_none() {
                return Err(format!("Context '{name}' not found."));
            }
            // If the deleted context was active, clear current_context.
            if cfg.current_context.as_deref() == Some(&name) {
                cfg.current_context = None;
                println!("Context '{name}' deleted (was active; no context is now active).");
            } else {
                println!("Context '{name}' deleted.");
            }
            config::save(cfg)?;
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// kenv init
// ---------------------------------------------------------------------------

fn run_kenv_init(resolved: &ResolvedContext, args: commands::kenv::InitArgs) -> Result<(), String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("Cannot determine current directory: {e}"))?;
    let path = cwd.join("ktool.toml");

    if path.exists() && !args.force {
        return Err(format!(
            "ktool.toml already exists in {}.\nUse --force to overwrite.",
            cwd.display()
        ));
    }

    // Use explicit flags first, then fall back to the resolved context values.
    let project = args.project.or(resolved.project.clone());
    let app = args.app.or(resolved.app.clone());
    let env = args.env.or(resolved.env.clone());

    let ctx = localrc::LocalContext { project, app, env };
    let content =
        toml::to_string_pretty(&ctx).map_err(|e| format!("Failed to serialize ktool.toml: {e}"))?;

    std::fs::write(&path, &content)
        .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;

    println!("Created {}:", path.display());
    print!("{content}");
    println!();
    println!(
        "This file will be picked up automatically by `ktool` in this directory and any subdirectory."
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_key_value_simple() {
        let (k, v) = parse_key_value("KEY=value").unwrap();
        assert_eq!(k, "KEY");
        assert_eq!(v, "value");
    }

    #[test]
    fn parse_key_value_uppercases_key() {
        let (k, _) = parse_key_value("my_key=val").unwrap();
        assert_eq!(k, "MY_KEY");
    }

    #[test]
    fn parse_key_value_preserves_value_with_equals() {
        // Values like base64 or connection strings may contain `=`.
        let (k, v) = parse_key_value("TOKEN=abc=def==").unwrap();
        assert_eq!(k, "TOKEN");
        assert_eq!(v, "abc=def==");
    }

    #[test]
    fn parse_key_value_empty_value_is_ok() {
        let (k, v) = parse_key_value("KEY=").unwrap();
        assert_eq!(k, "KEY");
        assert_eq!(v, "");
    }

    #[test]
    fn parse_key_value_missing_equals_is_error() {
        let err = parse_key_value("NOEQUALS").unwrap_err();
        assert!(err.contains("missing '='"), "unexpected error: {err}");
    }

    #[test]
    fn parse_key_value_empty_key_is_error() {
        let err = parse_key_value("=value").unwrap_err();
        assert!(err.contains("key is empty"), "unexpected error: {err}");
    }
}
