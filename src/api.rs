use serde_json::Value;

pub const BASE_URL: &str = "https://karluiz.com/api/env/orbital";

/// Fetch secrets from the karluiz kenv API.
///
/// Returns the parsed JSON response body.
pub fn fetch_secrets(app: &str, env: &str, api_key: &str) -> Result<Value, String> {
    let url = format!("{BASE_URL}?app={app}&env={env}");

    let resp = ureq::get(&url)
        .set("Authorization", &format!("Bearer {api_key}"))
        .set("Accept", "application/json")
        .call()
        .map_err(|e| format!("Request failed: {e}"))?;

    resp.into_json::<Value>()
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
    format!(
        "{}***{}",
        &chars[..2].iter().collect::<String>(),
        &chars[chars.len() - 2..].iter().collect::<String>()
    )
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
        assert_eq!(obfuscate("mysecrettoken"), "my***en");
    }

    #[test]
    fn obfuscate_exactly_five_chars() {
        assert_eq!(obfuscate("abcde"), "ab***de");
    }
}
