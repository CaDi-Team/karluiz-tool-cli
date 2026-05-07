//! CLI argument and subcommand types for the `ktool kenv` command tree.

use clap::{Args, Subcommand};

// ---------------------------------------------------------------------------
// Top-level kenv args
// ---------------------------------------------------------------------------

/// Arguments for `ktool kenv`.
#[derive(Args, Debug)]
pub struct KenvArgs {
    /// Set the project slug in the active context (or global defaults).
    ///
    /// The project slug is the `{projectSlug}` path segment used in all API URLs.
    /// Example: `ktool kenv --set-project=orbital`
    #[arg(long, value_name = "PROJECT")]
    pub set_project: Option<String>,

    /// Set the default application name in the active context (or global defaults).
    ///
    /// Example: `ktool kenv --set-app=backend`
    #[arg(long, value_name = "APP")]
    pub set_app: Option<String>,

    /// Set the default environment in the active context (or global defaults).
    ///
    /// Example: `ktool kenv --set-env=production`
    #[arg(long, value_name = "ENV")]
    pub set_env: Option<String>,

    #[command(subcommand)]
    pub command: Option<KenvCommands>,
}

// ---------------------------------------------------------------------------
// Subcommands
// ---------------------------------------------------------------------------

/// Subcommands available under `ktool kenv`.
#[derive(Subcommand, Debug)]
pub enum KenvCommands {
    /// Fetch and display environment variables for the configured app/env.
    ///
    /// Values are obfuscated by default (first 2 + last 2 characters visible).
    /// Use --json to get the raw JSON response, or --detailed to see
    /// inheritance information (which variables come from _shared).
    List(ListArgs),

    /// Manage applications inside the project.
    Apps {
        #[command(subcommand)]
        cmd: AppsCommands,
    },

    /// Manage environments inside an app.
    Envs {
        #[command(subcommand)]
        cmd: EnvsCommands,
    },

    /// Manage variables inside an environment.
    Vars {
        #[command(subcommand)]
        cmd: VarsCommands,
    },

    /// Manage named contexts (kubectl-style).
    ///
    /// Named contexts let you store multiple project/app/env combinations
    /// under a short name and switch between them instantly.
    Context {
        #[command(subcommand)]
        cmd: ContextCommands,
    },

    /// Create a local `ktool.toml` in the current directory.
    ///
    /// The local file takes precedence over the global context and is
    /// discovered automatically by walking up the directory tree (like asdf's
    /// `.tool-versions`). Useful for pinning a specific project/app/env
    /// per repository.
    Init(InitArgs),
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

/// Arguments for `ktool kenv list`.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Output the raw JSON response instead of obfuscated KEY=VALUE pairs.
    #[arg(long)]
    pub json: bool,

    /// Use the `detailed` API format: returns [{key, value, shared}] where
    /// `shared` indicates whether the variable is inherited from `_shared`.
    #[arg(long)]
    pub detailed: bool,
}

// ---------------------------------------------------------------------------
// apps
// ---------------------------------------------------------------------------

/// Subcommands for `ktool kenv apps`.
#[derive(Subcommand, Debug)]
pub enum AppsCommands {
    /// List all apps in the project.
    List,

    /// Create a new app in the project.
    Create {
        /// Name of the new application.
        name: String,
    },
}

// ---------------------------------------------------------------------------
// envs
// ---------------------------------------------------------------------------

/// Subcommands for `ktool kenv envs`.
#[derive(Subcommand, Debug)]
pub enum EnvsCommands {
    /// List all environments for an app.
    List(EnvsListArgs),

    /// Create a new environment inside an app.
    Create(EnvsCreateArgs),

    /// Delete an environment and all its variables.
    ///
    /// The `_shared` environment cannot be deleted via API.
    Delete(EnvsDeleteArgs),
}

/// Arguments for `ktool kenv envs list`.
#[derive(Args, Debug)]
pub struct EnvsListArgs {
    /// Override the project slug for this command (ignores context).
    #[arg(long, value_name = "PROJECT")]
    pub project: Option<String>,

    /// Override the app name for this command (ignores context).
    #[arg(long, value_name = "APP")]
    pub app: Option<String>,
}

/// Arguments for `ktool kenv envs create`.
#[derive(Args, Debug)]
pub struct EnvsCreateArgs {
    /// Name of the new environment (e.g. `staging`). The name `_shared` is reserved.
    pub name: String,

    /// Override the project slug for this command (ignores context).
    #[arg(long, value_name = "PROJECT")]
    pub project: Option<String>,

    /// Override the app name for this command (ignores context).
    #[arg(long, value_name = "APP")]
    pub app: Option<String>,
}

/// Arguments for `ktool kenv envs delete`.
#[derive(Args, Debug)]
pub struct EnvsDeleteArgs {
    /// Name of the environment to delete.
    pub name: String,

    /// Override the project slug for this command (ignores context).
    #[arg(long, value_name = "PROJECT")]
    pub project: Option<String>,

    /// Override the app name for this command (ignores context).
    #[arg(long, value_name = "APP")]
    pub app: Option<String>,
}

// ---------------------------------------------------------------------------
// vars
// ---------------------------------------------------------------------------

/// Subcommands for `ktool kenv vars`.
#[derive(Subcommand, Debug)]
pub enum VarsCommands {
    /// Create or update one or more variables (KEY=VALUE pairs).
    ///
    /// Keys are automatically uppercased. Existing keys are updated; new keys
    /// are created. A confirmation prompt is shown unless --yes is passed.
    ///
    /// Example:
    ///   ktool kenv vars set DATABASE_URL=postgres://... API_KEY=sk-abc
    Set(VarsSetArgs),

    /// Delete a single variable by key (case-insensitive).
    Delete(VarsDeleteArgs),
}

/// Arguments for `ktool kenv vars set`.
#[derive(Args, Debug)]
pub struct VarsSetArgs {
    /// One or more KEY=VALUE pairs.
    ///
    /// Keys are uppercased automatically. Values may contain `=` characters
    /// (only the first `=` is used as the delimiter).
    #[arg(required = true, num_args = 1..)]
    pub vars: Vec<String>,

    /// Override the project slug for this command (ignores context).
    #[arg(long, value_name = "PROJECT")]
    pub project: Option<String>,

    /// Override the app name for this command (ignores context).
    #[arg(long, value_name = "APP")]
    pub app: Option<String>,

    /// Override the environment name for this command (ignores context).
    #[arg(long, value_name = "ENV")]
    pub env: Option<String>,

    /// Skip the confirmation prompt and upsert immediately.
    #[arg(long, short = 'y')]
    pub yes: bool,
}

/// Arguments for `ktool kenv vars delete`.
#[derive(Args, Debug)]
pub struct VarsDeleteArgs {
    /// The variable key to delete (case-insensitive; matched as uppercase internally).
    pub key: String,

    /// Override the project slug for this command (ignores context).
    #[arg(long, value_name = "PROJECT")]
    pub project: Option<String>,

    /// Override the app name for this command (ignores context).
    #[arg(long, value_name = "APP")]
    pub app: Option<String>,

    /// Override the environment name for this command (ignores context).
    #[arg(long, value_name = "ENV")]
    pub env: Option<String>,
}

// ---------------------------------------------------------------------------
// context
// ---------------------------------------------------------------------------

/// Subcommands for `ktool kenv context`.
#[derive(Subcommand, Debug)]
pub enum ContextCommands {
    /// List all named contexts. The active context is marked with `*`.
    List,

    /// Show the active context, its values, and where they come from.
    Current,

    /// Switch to a named context.
    Use {
        /// Name of the context to activate.
        name: String,
    },

    /// Create or update a named context.
    ///
    /// Example:
    ///   ktool kenv context set work --project=orbital --app=backend --env=production
    Set(ContextSetArgs),

    /// Delete a named context.
    Delete {
        /// Name of the context to delete.
        name: String,
    },
}

/// Arguments for `ktool kenv context set`.
#[derive(Args, Debug)]
pub struct ContextSetArgs {
    /// Name for the context (e.g. `work-backend`, `personal`).
    pub name: String,

    /// Project slug to store in this context.
    #[arg(long, value_name = "PROJECT")]
    pub project: Option<String>,

    /// Application name to store in this context.
    #[arg(long, value_name = "APP")]
    pub app: Option<String>,

    /// Environment name to store in this context.
    #[arg(long, value_name = "ENV")]
    pub env: Option<String>,
}

// ---------------------------------------------------------------------------
// init
// ---------------------------------------------------------------------------

/// Arguments for `ktool kenv init`.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Project slug to write into `ktool.toml`. Defaults to the resolved context value.
    #[arg(long, value_name = "PROJECT")]
    pub project: Option<String>,

    /// App name to write into `ktool.toml`. Defaults to the resolved context value.
    #[arg(long, value_name = "APP")]
    pub app: Option<String>,

    /// Environment name to write into `ktool.toml`. Defaults to the resolved context value.
    #[arg(long, value_name = "ENV")]
    pub env: Option<String>,

    /// Overwrite an existing `ktool.toml` if one is already present in the current directory.
    #[arg(long)]
    pub force: bool,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::Cli;
    use clap::CommandFactory;

    // ---- existing ----

    #[test]
    fn kenv_set_app_and_set_env_are_optional() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_list_subcommand_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "list"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_list_json_flag_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "list", "--json"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_list_detailed_flag_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "list", "--detailed"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_set_app_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "--set-app=my-app"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_set_env_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "--set-env=prod"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_set_project_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "--set-project=orbital"]);
        assert!(r.is_ok());
    }

    // ---- apps ----

    #[test]
    fn kenv_apps_list_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "apps", "list"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_apps_create_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "apps", "create", "my-app"]);
        assert!(r.is_ok());
    }

    // ---- envs ----

    #[test]
    fn kenv_envs_list_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "envs", "list"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_envs_list_with_app_override_parses() {
        let r =
            Cli::command().try_get_matches_from(["ktool", "kenv", "envs", "list", "--app=backend"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_envs_create_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "envs", "create", "staging"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_envs_delete_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "envs", "delete", "staging"]);
        assert!(r.is_ok());
    }

    // ---- vars ----

    #[test]
    fn kenv_vars_set_single_pair_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "vars", "set", "KEY=value"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_vars_set_multiple_pairs_parses() {
        let r = Cli::command().try_get_matches_from([
            "ktool",
            "kenv",
            "vars",
            "set",
            "KEY1=val1",
            "KEY2=val2",
        ]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_vars_set_yes_flag_parses() {
        let r = Cli::command()
            .try_get_matches_from(["ktool", "kenv", "vars", "set", "KEY=val", "--yes"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_vars_set_short_yes_flag_parses() {
        let r =
            Cli::command().try_get_matches_from(["ktool", "kenv", "vars", "set", "KEY=val", "-y"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_vars_set_with_overrides_parses() {
        let r = Cli::command().try_get_matches_from([
            "ktool",
            "kenv",
            "vars",
            "set",
            "KEY=val",
            "--project=orbital",
            "--app=backend",
            "--env=prod",
        ]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_vars_delete_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "vars", "delete", "MY_KEY"]);
        assert!(r.is_ok());
    }

    // ---- context ----

    #[test]
    fn kenv_context_list_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "context", "list"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_context_current_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "context", "current"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_context_use_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "context", "use", "work"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_context_set_parses() {
        let r = Cli::command().try_get_matches_from([
            "ktool",
            "kenv",
            "context",
            "set",
            "work",
            "--project=orbital",
            "--app=backend",
            "--env=prod",
        ]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_context_delete_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "context", "delete", "work"]);
        assert!(r.is_ok());
    }

    // ---- init ----

    #[test]
    fn kenv_init_parses() {
        let r = Cli::command().try_get_matches_from(["ktool", "kenv", "init"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_init_with_all_flags_parses() {
        let r = Cli::command().try_get_matches_from([
            "ktool",
            "kenv",
            "init",
            "--project=orbital",
            "--app=backend",
            "--env=production",
            "--force",
        ]);
        assert!(r.is_ok());
    }
}
