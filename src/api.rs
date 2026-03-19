use reqwest::blocking::Client;
use serde_json::Value;

pub const BASE_URL: &str = "https://karluiz.com/api/env/orbital";

/// Fetch secrets from the karluiz kenv API.
///
/// Returns the parsed JSON response body.
pub fn fetch_secrets(app: &str, env: &str, api_key: &str) -> Result<Value, String> {
    let url = format!("{BASE_URL}?app={app}&env={env}");

    let client = Client::new();

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "application/json")
        .send()
        .map_err(|e| format!("Request failed: {e}"))?;

    let status = response.status();

    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        return Err(format!("Request failed with status {status}: {body}"));
    }

    response
        .json::<Value>()
        .map_err(|e| format!("Failed to parse response: {e}"))
}

/// Obfuscate a secret value for display: show the first two and last two characters,
/// replacing everything in between with `***`.
///
/// Values of 4 characters or fewer are fully masked.
pub fn obfuscate(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 4 {
        return "***".to_string();
    }
    format!("{}***{}", &chars[..2].iter().collect::<String>(), &chars[chars.len() - 2..].iter().collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_url_is_correct() {
        assert_eq!(BASE_URL, "https://karluiz.com/api/env/orbital");
    }

    #[test]
    fn obfuscate_short_value_is_fully_masked() {
        assert_eq!(obfuscate("abc"), "***");
        assert_eq!(obfuscate("1234"), "***");
    }

    #[test]
    fn obfuscate_long_value_shows_prefix_and_suffix() {
        // "mysecrettoken" → "my***en"
        assert_eq!(obfuscate("mysecrettoken"), "my***en");
    }

    #[test]
    fn obfuscate_exactly_five_chars() {
        // 5 chars → first 2 + *** + last 2 = "ab***de"
        assert_eq!(obfuscate("abcde"), "ab***de");
    }
}
