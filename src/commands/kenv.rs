use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum KenvCommands {
    /// Retrieve secrets for an application and environment
    Get(GetArgs),
}

#[derive(Args)]
pub struct GetArgs {
    /// Application name
    #[arg(short, long)]
    pub app: String,

    /// Environment (e.g. prod, staging, dev)
    #[arg(short, long)]
    pub env: String,

    /// API key for authentication (overrides the KENV_API_KEY environment variable)
    #[arg(short = 'k', long)]
    pub api_key: Option<String>,

    /// Output the full JSON response instead of KEY=VALUE pairs
    #[arg(long)]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use crate::Cli;

    #[test]
    fn kenv_get_requires_app_and_env() {
        // Missing --app should produce an error
        let result = Cli::command().try_get_matches_from(["karluiz-tool", "kenv", "get", "--env", "prod"]);
        assert!(result.is_err());

        // Missing --env should produce an error
        let result = Cli::command().try_get_matches_from(["karluiz-tool", "kenv", "get", "--app", "myapp"]);
        assert!(result.is_err());

        // Both provided → ok
        let result = Cli::command().try_get_matches_from([
            "karluiz-tool", "kenv", "get", "--app", "myapp", "--env", "prod",
        ]);
        assert!(result.is_ok());
    }

    #[test]
    fn kenv_get_json_flag_defaults_to_false() {
        let result = Cli::command()
            .try_get_matches_from(["karluiz-tool", "kenv", "get", "--app", "a", "--env", "b"])
            .unwrap();
        // Walk through the matched subcommands to reach GetArgs
        let kenv = result.subcommand_matches("kenv").unwrap();
        let get = kenv.subcommand_matches("get").unwrap();
        let json_flag: bool = *get.get_one("json").unwrap();
        assert!(!json_flag);
    }
}
