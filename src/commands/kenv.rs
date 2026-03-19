use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct KenvArgs {
    /// Persist a new default application name in the config file.
    #[arg(long, value_name = "APP")]
    pub set_app: Option<String>,

    /// Persist a new default environment in the config file (e.g. prod, staging, dev).
    #[arg(long, value_name = "ENV")]
    pub set_env: Option<String>,

    #[command(subcommand)]
    pub command: Option<KenvCommands>,
}

#[derive(Subcommand, Debug)]
pub enum KenvCommands {
    /// List secret names and obfuscated values for the configured app/env.
    List(ListArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Output the full JSON response instead of obfuscated KEY=VALUE pairs.
    #[arg(long)]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use crate::Cli;

    #[test]
    fn kenv_set_app_and_set_env_are_optional() {
        // Neither flag → valid (shows help or config)
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
    fn kenv_set_app_parses() {
        let r = Cli::command()
            .try_get_matches_from(["ktool", "kenv", "--set-app=my-app"]);
        assert!(r.is_ok());
    }

    #[test]
    fn kenv_set_env_parses() {
        let r = Cli::command()
            .try_get_matches_from(["ktool", "kenv", "--set-env=prod"]);
        assert!(r.is_ok());
    }
}
