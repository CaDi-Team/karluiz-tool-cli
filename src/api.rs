//! HTTP client layer for the kENV public API.
//!
//! All requests are authenticated with a Bearer token (`kenv_<hex>`).
//! The base host is [`API_HOST`]; project-scoped URLs are built with the
//! project slug supplied by the caller (resolved from the active context).
//!
//! # URL structure
//! ```text
//! GET  /api/env/{project}                                 — fetch vars
//! GET  /api/env/{project}/apps                            — list apps
//! POST /api/env/{project}/apps                            — create app
//! GET  /api/env/{project}/apps/{app}/envs                 — list environments
//! POST /api/env/{project}/apps/{app}/envs                 — create environment
//! DEL  /api/env/{project}/apps/{app}/envs/{env}           — delete environment
//! PUT  /api/env/{project}/apps/{app}/envs/{env}/vars      — bulk upsert variables
//! DEL  /api/env/{project}/apps/{app}/envs/{env}/vars/{k}  — delete variable
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Base host for all kENV API requests.
pub const API_HOST: &str = "https://karluiz.com";

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

/// An application inside a project, as returned by `listApps`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct App {
    pub name: String,
    #[serde(default)]
    pub environments: Vec<AppEnvRef>,
}

/// Minimal environment reference embedded inside an [`App`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppEnvRef {
    pub name: String,
}

/// An environment inside an app, as returned by `listEnvironments`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Environment {
    pub name: String,
    #[serde(rename = "variableCount", default)]
    pub variable_count: u32,
}

/// A variable entry in the `detailed` response format.
#[allow(dead_code)] // constructed by serde_json during deserialization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VariableDetailed {
    pub key: String,
    pub value: String,
    /// `true` if this variable is inherited from the `_shared` environment.
    #[serde(default)]
    pub shared: bool,
}

/// Summary returned by the bulk-upsert (`PUT .../vars`) endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BulkUpsertResult {
    pub created: u32,
    pub updated: u32,
    /// Sorted list of all variable keys in the environment after the operation.
    pub variables: Vec<String>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Build the base URL for all variable/env/app operations in a project.
fn project_url(project: &str) -> String {
    format!("{API_HOST}/api/env/{project}")
}

/// Attach a Bearer token and an Accept header, then call the given request.
macro_rules! bearer {
    ($req:expr, $token:expr) => {
        $req.set("Authorization", &format!("Bearer {}", $token))
            .set("Accept", "application/json")
    };
}

/// Parse a successful JSON body.
fn parse_json<T: serde::de::DeserializeOwned>(resp: ureq::Response) -> Result<T, String> {
    resp.into_json::<T>()
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Convert a ureq error into a human-readable string.
fn handle_error(e: ureq::Error) -> String {
    match e {
        ureq::Error::Status(code, resp) => {
            let body = resp.into_string().unwrap_or_default();
            // Try to extract the `error` field from the JSON error body.
            if let Ok(v) = serde_json::from_str::<Value>(&body)
                && let Some(msg) = v.get("error").and_then(|m| m.as_str())
            {
                return format!("API error {code}: {msg}");
            }
            format!("API error {code}")
        }
        ureq::Error::Transport(e) => format!("Request failed: {e}"),
    }
}

// ---------------------------------------------------------------------------
// Variables
// ---------------------------------------------------------------------------

/// Fetch environment variables for the given app and environment.
///
/// `detailed = true` → returns the array form `[{key, value, shared}]`.
/// `detailed = false` → returns the flat object form `{KEY: "value"}`.
pub fn fetch_secrets(
    project: &str,
    app: &str,
    env: &str,
    detailed: bool,
    api_key: &str,
) -> Result<Value, String> {
    let format = if detailed { "detailed" } else { "flat" };
    let url = format!(
        "{base}?app={app}&env={env}&format={format}",
        base = project_url(project)
    );

    let resp = bearer!(ureq::get(&url), api_key)
        .call()
        .map_err(handle_error)?;

    parse_json(resp)
}

/// Bulk-create or update variables in an environment.
///
/// `vars` is a slice of `(KEY, value)` pairs; keys will be uppercased by the
/// server automatically.
///
/// Returns a [`BulkUpsertResult`] with counts of created/updated items and the
/// full sorted list of keys in the environment after the operation.
pub fn upsert_variables(
    project: &str,
    app: &str,
    env: &str,
    vars: &[(String, String)],
    token: &str,
) -> Result<BulkUpsertResult, String> {
    let url = format!("{}/apps/{app}/envs/{env}/vars", project_url(project));

    // Build the request body: [{key, value}, ...].
    let body: Vec<Value> = vars
        .iter()
        .map(|(k, v)| serde_json::json!({ "key": k, "value": v }))
        .collect();

    let resp = bearer!(ureq::put(&url), token)
        .set("Content-Type", "application/json")
        .send_json(Value::Array(body))
        .map_err(handle_error)?;

    parse_json(resp)
}

/// Delete a single variable by key.
pub fn delete_variable(
    project: &str,
    app: &str,
    env: &str,
    key: &str,
    token: &str,
) -> Result<(), String> {
    let url = format!("{}/apps/{app}/envs/{env}/vars/{key}", project_url(project));

    bearer!(ureq::delete(&url), token)
        .call()
        .map_err(handle_error)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Apps
// ---------------------------------------------------------------------------

/// List all apps in the project.
pub fn list_apps(project: &str, token: &str) -> Result<Vec<App>, String> {
    let url = format!("{}/apps", project_url(project));

    let resp = bearer!(ureq::get(&url), token)
        .call()
        .map_err(handle_error)?;

    parse_json(resp)
}

/// Create a new app in the project.
pub fn create_app(project: &str, name: &str, token: &str) -> Result<App, String> {
    let url = format!("{}/apps", project_url(project));

    let resp = bearer!(ureq::post(&url), token)
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({ "name": name }))
        .map_err(handle_error)?;

    parse_json(resp)
}

// ---------------------------------------------------------------------------
// Environments
// ---------------------------------------------------------------------------

/// List all environments for an app (excludes `_shared`).
pub fn list_environments(
    project: &str,
    app: &str,
    token: &str,
) -> Result<Vec<Environment>, String> {
    let url = format!("{}/apps/{app}/envs", project_url(project));

    let resp = bearer!(ureq::get(&url), token)
        .call()
        .map_err(handle_error)?;

    parse_json(resp)
}

/// Create a new environment inside an app. The name `_shared` is reserved.
pub fn create_environment(
    project: &str,
    app: &str,
    name: &str,
    token: &str,
) -> Result<Environment, String> {
    let url = format!("{}/apps/{app}/envs", project_url(project));

    let resp = bearer!(ureq::post(&url), token)
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({ "name": name }))
        .map_err(handle_error)?;

    parse_json(resp)
}

/// Delete an environment and all its variables.
///
/// The `_shared` environment cannot be deleted via API.
pub fn delete_environment(project: &str, app: &str, env: &str, token: &str) -> Result<(), String> {
    let url = format!("{}/apps/{app}/envs/{env}", project_url(project));

    bearer!(ureq::delete(&url), token)
        .call()
        .map_err(handle_error)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

/// Obfuscate a secret value for display: show the first two and last two characters,
/// replacing everything in between with `***`.
///
/// Values of 4 characters or fewer are fully masked.
pub fn obfuscate(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 4 {
        return "***".to_string();
    }
    format!(
        "{}***{}",
        &chars[..2].iter().collect::<String>(),
        &chars[chars.len() - 2..].iter().collect::<String>()
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_host_is_correct() {
        assert_eq!(API_HOST, "https://karluiz.com");
    }

    #[test]
    fn project_url_is_correct() {
        assert_eq!(
            project_url("orbital"),
            "https://karluiz.com/api/env/orbital"
        );
    }

    #[test]
    fn obfuscate_short_value_is_fully_masked() {
        assert_eq!(obfuscate("abc"), "***");
        assert_eq!(obfuscate("1234"), "***");
    }

    #[test]
    fn obfuscate_long_value_shows_prefix_and_suffix() {
        assert_eq!(obfuscate("mysecrettoken"), "my***en");
    }

    #[test]
    fn obfuscate_exactly_five_chars() {
        assert_eq!(obfuscate("abcde"), "ab***de");
    }

    #[test]
    fn bulk_upsert_result_deserializes() {
        let json = r#"{"created":2,"updated":1,"variables":["API_KEY","DB_URL","SECRET"]}"#;
        let result: BulkUpsertResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.created, 2);
        assert_eq!(result.updated, 1);
        assert_eq!(result.variables, vec!["API_KEY", "DB_URL", "SECRET"]);
    }

    #[test]
    fn variable_detailed_deserializes() {
        let json = r#"{"key":"DB_URL","value":"postgres://...","shared":true}"#;
        let v: VariableDetailed = serde_json::from_str(json).unwrap();
        assert_eq!(v.key, "DB_URL");
        assert!(v.shared);
    }

    #[test]
    fn app_env_ref_deserializes() {
        let json = r#"{"name":"my-app","environments":[{"name":"production"}]}"#;
        let app: App = serde_json::from_str(json).unwrap();
        assert_eq!(app.name, "my-app");
        assert_eq!(app.environments[0].name, "production");
    }

    #[test]
    fn environment_deserializes() {
        let json = r#"{"name":"production","variableCount":12}"#;
        let env: Environment = serde_json::from_str(json).unwrap();
        assert_eq!(env.name, "production");
        assert_eq!(env.variable_count, 12);
    }
}
