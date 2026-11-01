use crate::credentials::{resolve_config_secret, SystemCredentialVault};
use crate::SessionTokenStore;
use crate::{api_base_url, with_desktop_client_version};

#[tauri::command]
pub fn get_llm_model_capability(
    provider: String,
    base_url: String,
    model: String,
) -> crate::llm::model_capabilities::ModelCapability {
    crate::llm::model_capabilities::model_capability(
        &provider,
        &base_url,
        &model,
        crate::llm::prompt::CONTEXT_PROMPT_VERSION,
    )
}

fn synthetic_operation_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        (now >> 96) as u32,
        (now >> 80) as u16,
        (now >> 64) as u16,
        (now >> 48) as u16,
        now & 0x0000_ffff_ffff_ffff_ffffu128
    )
}

fn has_managed_cloud_access(body: &serde_json::Value) -> bool {
    if matches!(
        body["licenseStatus"].as_str(),
        Some("refunded") | Some("deactivated")
    ) {
        return false;
    }

    let source = body["source"].as_str().unwrap_or_default();
    let plan = body["plan"].as_str().unwrap_or_default();
    let cloud_words_limit = body["cloudWordsLimit"].as_i64().unwrap_or_default();
    let display_words_limit = body["displayWordsLimit"].as_i64().unwrap_or_default();
    if source == "appsumo" {
        return cloud_words_limit > 0 && body["licenseStatus"].as_str() == Some("active");
    }
    if source == "lifetime" {
        return cloud_words_limit > 0 || display_words_limit > 0 || plan == "lifetime_starter";
    }
    if source == "creem" && (cloud_words_limit > 0 || display_words_limit > 0) {
        return true;
    }

    matches!(plan, "pro" | "lifetime_starter")
}

#[tauri::command]
pub async fn test_llm_connection(
    api_key: String,
    provider: String,
    base_url: String,
    model: String,
    token_store: tauri::State<'_, SessionTokenStore>,
    client: tauri::State<'_, reqwest::Client>,
) -> Result<bool, String> {
    if provider.is_empty() {
        return Ok(false);
    }

    // Cloud provider: verify session token + managed cloud entitlement via API.
    if provider == "cloud" {
        let token = token_store
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if token.is_empty() {
            return Ok(false);
        }
        let api_base = api_base_url();
        let resp = with_desktop_client_version(
            client.get(format!("{}/api/subscription/status", api_base)),
        )
        .header("Authorization", format!("Bearer {}", token))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Ok(false);
        }
        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        return Ok(has_managed_cloud_access(&body));
    }

    let api_key = resolve_config_secret(&api_key, "llm", &provider, &SystemCredentialVault)
        .map_err(|e| e.to_string())?;

    if base_url.is_empty() || !crate::llm::has_usable_provider_credentials(&provider, &api_key) {
        return Ok(false);
    }

    // Validate base_url is a proper HTTP(S) URL
    let parsed = url::Url::parse(&base_url).map_err(|e| format!("Invalid base URL: {e}"))?;
    if parsed.scheme() != "https" && parsed.scheme() != "http" {
        return Err("Base URL must use http or https scheme".to_string());
    }

    let url = crate::llm::protocol::chat_endpoint(&provider, &base_url)?;
    let body = crate::llm::protocol::build_chat_body(
        &provider,
        &base_url,
        &model,
        vec![serde_json::json!({"role": "user", "content": "hi"})],
        1,
        0.3,
        false,
    );

    let request = client.post(&url).header("Content-Type", "application/json");
    let resp = crate::llm::protocol::apply_auth_headers(request, &provider, &base_url, &api_key)
        .json(&body)
        .timeout(crate::llm::protocol::request_timeout(
            &provider, &base_url, &model,
        ))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(resp.status().is_success())
}

fn build_fetch_models_request(
    client: &reqwest::Client,
    provider: &str,
    base_url: &str,
    api_key: &str,
    url: &str,
) -> reqwest::RequestBuilder {
    crate::llm::protocol::apply_auth_headers(client.get(url), provider, base_url, api_key)
}

#[tauri::command]
pub async fn fetch_llm_models(
    api_key: String,
    provider: String,
    base_url: String,
    client: tauri::State<'_, reqwest::Client>,
) -> Result<Vec<String>, String> {
    if base_url.is_empty() {
        return Ok(vec![]);
    }
    if !crate::llm::has_usable_provider_credentials(&provider, &api_key) {
        return Ok(vec![]);
    }

    // Validate base_url is a proper HTTP(S) URL
    let parsed = url::Url::parse(&base_url).map_err(|e| format!("Invalid base URL: {e}"))?;
    if parsed.scheme() != "https" && parsed.scheme() != "http" {
        return Err("Base URL must use http or https scheme".to_string());
    }

    let url = crate::llm::protocol::models_endpoint(&provider, &base_url)?;

    let resp = build_fetch_models_request(&client, &provider, &base_url, &api_key, &url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Ok(vec![]);
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    // OpenAI-compatible: { data: [{ id: "model-name" }] }
    // Ollama-compatible: { models: [{ name: "model-name" }] }
    let mut models: Vec<String> = Vec::new();

    if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
        for item in data {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                models.push(id.to_string());
            }
        }
    } else if let Some(data) = body.get("models").and_then(|d| d.as_array()) {
        for item in data {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                models.push(name.to_string());
            }
        }
    }

    models.sort();
    if crate::llm::protocol::detect_api_kind(&provider, &base_url)
        == crate::llm::protocol::LlmApiKind::AnthropicMessages
    {
        models = latest_claude_models(models);
    }
    Ok(models)
}

/// Claude families offered in the model picker, fastest first.
const CLAUDE_PICKER_FAMILIES: [&str; 3] = ["haiku", "sonnet", "opus"];

/// Parses `claude-<family>-<major>[-<minor>][-<date>]` into the family and
/// version. Claude 3 names (`claude-3-5-sonnet-…`) and other families
/// return `None`.
fn claude_family_version(model: &str) -> Option<(&'static str, (u32, u32))> {
    let mut parts = model.strip_prefix("claude-")?.split('-');
    let family_name = parts.next()?;
    let family = CLAUDE_PICKER_FAMILIES
        .into_iter()
        .find(|family| *family == family_name)?;
    let major = parts.next()?.parse::<u32>().ok()?;
    // A date suffix such as claude-sonnet-4-20250514 is not a minor version.
    let minor = parts
        .next()
        .and_then(|part| part.parse::<u32>().ok())
        .filter(|minor| *minor < 100)
        .unwrap_or(0);
    Some((family, (major, minor)))
}

/// Keeps only the newest Haiku, Sonnet and Opus from Anthropic's model list,
/// so older generations and other families (such as Fable) are not offered.
/// Returns the list unchanged when none of its names can be parsed.
fn latest_claude_models(models: Vec<String>) -> Vec<String> {
    let mut latest: Vec<String> = Vec::new();
    for family in CLAUDE_PICKER_FAMILIES {
        let newest = models
            .iter()
            .filter_map(|model| {
                claude_family_version(model)
                    .filter(|(found, _)| *found == family)
                    .map(|(_, version)| (version, model))
            })
            // Same version: prefer the alias without a date suffix.
            .max_by(|(a, a_model), (b, b_model)| {
                a.cmp(b).then_with(|| b_model.len().cmp(&a_model.len()))
            });
        if let Some((_, model)) = newest {
            latest.push(model.clone());
        }
    }
    if latest.is_empty() {
        models
    } else {
        latest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_picker_keeps_only_the_newest_haiku_sonnet_and_opus() {
        // The list api.anthropic.com returned on 2026-11-01.
        let models = [
            "claude-haiku-5-5",
            "claude-sonnet-5-5",
            "claude-opus-5-5",
            "claude-fable-5-1",
            "claude-opus-5",
            "claude-sonnet-5",
            "claude-fable-5",
            "claude-opus-4-8",
            "claude-opus-4-7",
            "claude-sonnet-4-6",
            "claude-opus-4-6",
            "claude-opus-4-5-20251101",
            "claude-haiku-4-5-20251001",
            "claude-sonnet-4-5-20250929",
        ]
        .map(String::from)
        .to_vec();

        assert_eq!(
            latest_claude_models(models),
            ["claude-haiku-5-5", "claude-sonnet-5-5", "claude-opus-5-5"]
        );
    }

    #[test]
    fn claude_picker_prefers_aliases_and_keeps_unknown_lists() {
        let models = ["claude-haiku-4-5-20251001", "claude-haiku-4-5"]
            .map(String::from)
            .to_vec();
        assert_eq!(latest_claude_models(models), ["claude-haiku-4-5"]);

        let proxy = ["glm-4.6", "kimi-k2"].map(String::from).to_vec();
        assert_eq!(latest_claude_models(proxy.clone()), proxy);
    }

    #[test]
    fn managed_cloud_access_requires_active_appsumo_license() {
        let active = serde_json::json!({
            "plan": "appsumo_tier1",
            "source": "appsumo",
            "cloudWordsLimit": 200000,
            "licenseStatus": "active"
        });
        let pending = serde_json::json!({
            "plan": "appsumo_tier1",
            "source": "appsumo",
            "cloudWordsLimit": 200000,
            "licenseStatus": "pending"
        });
        let missing = serde_json::json!({
            "plan": "appsumo_tier1",
            "source": "appsumo",
            "cloudWordsLimit": 200000
        });

        assert!(has_managed_cloud_access(&active));
        assert!(!has_managed_cloud_access(&pending));
        assert!(!has_managed_cloud_access(&missing));
    }

    #[test]
    fn managed_cloud_access_allows_direct_lifetime_license() {
        let lifetime_legacy_quota = serde_json::json!({
            "plan": "lifetime_starter",
            "source": "lifetime",
            "cloudWordsLimit": 0,
            "licenseStatus": "active"
        });
        let lifetime_cloud_words = serde_json::json!({
            "plan": "lifetime_starter",
            "source": "lifetime",
            "cloudWordsLimit": 100000
        });

        assert!(has_managed_cloud_access(&lifetime_legacy_quota));
        assert!(has_managed_cloud_access(&lifetime_cloud_words));
    }

    #[test]
    fn model_request_omits_authorization_for_keyless_ollama() {
        let request = build_fetch_models_request(
            &reqwest::Client::new(),
            "ollama",
            "http://localhost:11434/v1",
            "",
            "http://localhost:11434/v1/models",
        )
        .build()
        .unwrap();

        assert!(request.headers().get("Authorization").is_none());
    }

    #[test]
    fn model_request_keeps_authorization_for_keyed_providers() {
        let request = build_fetch_models_request(
            &reqwest::Client::new(),
            "openai",
            "https://api.openai.com/v1",
            "sk-test",
            "https://api.openai.com/v1/models",
        )
        .build()
        .unwrap();

        assert_eq!(
            request.headers().get("Authorization").unwrap(),
            "Bearer sk-test"
        );
    }
}

#[tauri::command]
pub async fn bench_llm_connection(
    api_key: String,
    provider: String,
    base_url: String,
    model: String,
    token_store: tauri::State<'_, SessionTokenStore>,
    client: tauri::State<'_, reqwest::Client>,
) -> Result<u32, String> {
    if provider.is_empty() {
        return Err("No provider specified".to_string());
    }

    if provider == "cloud" {
        let token = token_store
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        if token.is_empty() {
            return Err("Not signed in".to_string());
        }
        let api_base = api_base_url();
        let operation_id = synthetic_operation_id();
        let body = serde_json::json!({
            "messages": [{"role": "user", "content": "hi"}],
            "stream": false,
            "context": {
                "operationId": operation_id.clone(),
                "stageKey": format!("{operation_id}:llm"),
                "requestType": "connection_benchmark",
                "clientVersion": crate::desktop_client_version(),
                "rawTextChars": 2,
                "selectedTextChars": 0,
                "hasSelectedText": false,
                "translateEnabled": false
            }
        });
        let t0 = std::time::Instant::now();
        let resp = with_desktop_client_version(client.post(format!("{}/api/proxy/llm", api_base)))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let elapsed = t0.elapsed().as_millis() as u32;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        return Ok(elapsed);
    }

    let api_key = resolve_config_secret(&api_key, "llm", &provider, &SystemCredentialVault)
        .map_err(|e| e.to_string())?;

    if base_url.is_empty() || !crate::llm::has_usable_provider_credentials(&provider, &api_key) {
        return Err("API key or base URL is empty".to_string());
    }

    let parsed = url::Url::parse(&base_url).map_err(|e| format!("Invalid base URL: {e}"))?;
    if parsed.scheme() != "https" && parsed.scheme() != "http" {
        return Err("Base URL must use http or https scheme".to_string());
    }

    let url = crate::llm::protocol::chat_endpoint(&provider, &base_url)?;
    let body = crate::llm::protocol::build_chat_body(
        &provider,
        &base_url,
        &model,
        vec![serde_json::json!({"role": "user", "content": "hi"})],
        1,
        0.3,
        false,
    );

    let t0 = std::time::Instant::now();
    let request = client.post(&url).header("Content-Type", "application/json");
    let resp = crate::llm::protocol::apply_auth_headers(request, &provider, &base_url, &api_key)
        .json(&body)
        .timeout(crate::llm::protocol::request_timeout(
            &provider, &base_url, &model,
        ))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let elapsed = t0.elapsed().as_millis() as u32;

    if !resp.status().is_success() {
        let status = resp.status();
        let details: String = resp
            .text()
            .await
            .unwrap_or_default()
            .chars()
            .take(200)
            .collect();
        return Err(if details.is_empty() {
            format!("HTTP {status}")
        } else {
            format!("HTTP {status}: {details}")
        });
    }

    Ok(elapsed)
}
