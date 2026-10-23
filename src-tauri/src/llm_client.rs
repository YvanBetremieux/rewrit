use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use futures_util::StreamExt;
use crate::api_client::{TranslateResponse, QuotaInfo};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    Groq,
    Mistral,
    Gemini,
    Ollama,
}

impl LlmProvider {
    pub fn api_url(&self) -> &str {
        match self {
            LlmProvider::Groq => "https://api.groq.com/openai/v1/chat/completions",
            LlmProvider::Mistral => "https://api.mistral.ai/v1/chat/completions",
            LlmProvider::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
            LlmProvider::Ollama => "http://localhost:11434/api/chat",
        }
    }

    pub fn default_model(&self) -> &str {
        match self {
            LlmProvider::Groq => "openai/gpt-oss-120b",
            LlmProvider::Mistral => "mistral-small-latest",
            LlmProvider::Gemini => "gemini-2.5-flash",
            LlmProvider::Ollama => "llama3.2:3b",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            LlmProvider::Groq => "Groq",
            LlmProvider::Mistral => "Mistral",
            LlmProvider::Gemini => "Gemini",
            LlmProvider::Ollama => "Ollama",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mistral" => LlmProvider::Mistral,
            "gemini" => LlmProvider::Gemini,
            "ollama" => LlmProvider::Ollama,
            _ => LlmProvider::Groq,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            LlmProvider::Groq => "groq",
            LlmProvider::Mistral => "mistral",
            LlmProvider::Gemini => "gemini",
            LlmProvider::Ollama => "ollama",
        }
    }

    pub fn requires_api_key(&self) -> bool {
        !matches!(self, LlmProvider::Ollama)
    }

    /// Groq's default model is a reasoning model: keep reasoning short to stay fast.
    fn reasoning_effort(&self) -> Option<&'static str> {
        match self {
            LlmProvider::Groq => Some("low"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LlmClient {
    client: Arc<reqwest::Client>,
    api_key: String,
    provider: LlmProvider,
    model: String,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'static str>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

// Ollama uses a different request/response format
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    message: Message,
}

const SYSTEM_PROMPT: &str = "You are a helpful text transformation assistant. Apply the user's instruction to the provided text. Return ONLY the transformed result, nothing else. No explanations, no quotes, no parentheses. Preserve the exact formatting: keep all line breaks and internal whitespace structure.";

impl LlmClient {
    pub fn new(provider: LlmProvider, api_key: String) -> Self {
        let model = crate::device_id::get_model(provider.as_str())
            .unwrap_or_else(|| provider.default_model().to_string());
        // Ollama needs longer timeout (local inference is slow)
        let timeout_secs = if matches!(provider, LlmProvider::Ollama) { 120 } else { 15 };
        let client = Arc::new(
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout_secs))
                .build()
                .expect("Failed to create HTTP client")
        );
        Self { client, api_key, provider, model }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub async fn execute_prompt(&self, text: String, user_prompt: String) -> Result<TranslateResponse, String> {
        let start_time = Instant::now();
        let provider_name = self.provider.display_name();

        let system_prompt = SYSTEM_PROMPT.to_string();

        // Extract leading/trailing whitespace to restore after transformation
        let leading_ws = &text[..text.len() - text.trim_start().len()];
        let trailing_ws = &text[text.trim_end().len()..];
        let trimmed_text = text.trim().to_string();

        let user_message = format!("Instruction: {}\n\nText to transform:\n{}", user_prompt, trimmed_text);

        let messages = vec![
            ChatMessage { role: "system".to_string(), content: system_prompt },
            ChatMessage { role: "user".to_string(), content: user_message },
        ];

        let api_url = self.provider.api_url();
        let masked_key = if self.api_key.len() > 8 {
            format!("{}...{}", &self.api_key[..4], &self.api_key[self.api_key.len()-4..])
        } else if self.api_key.is_empty() {
            "(none)".to_string()
        } else {
            "****".to_string()
        };
        log::info!("[rewrit] LLM request: provider={}, model={}", provider_name, self.model);
        #[cfg(debug_assertions)]
        log::info!("[rewrit] LLM debug: url={}, key={}, text={} chars", api_url, masked_key, text.len());

        // Send request (Ollama uses a different format)
        let response = if matches!(self.provider, LlmProvider::Ollama) {
            let request = OllamaRequest {
                model: self.model.clone(),
                messages,
                stream: false,
                options: OllamaOptions { temperature: 0.3, num_predict: 2000 },
            };
            self.client.post(api_url)
                .json(&request)
                .send()
                .await
        } else {
            let request = ChatCompletionRequest {
                model: self.model.clone(),
                messages,
                temperature: 0.3,
                max_tokens: 2000,
                reasoning_effort: self.provider.reasoning_effort(),
            };
            self.client.post(api_url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&request)
                .send()
                .await
        }.map_err(|e| {
            log::error!("[rewrit] LLM error (send): {}", e);
            if e.is_timeout() {
                format!("{} timeout - the request took too long", provider_name)
            } else if e.is_connect() {
                if matches!(self.provider, LlmProvider::Ollama) {
                    "Cannot connect to Ollama - make sure it's running (ollama serve)".to_string()
                } else {
                    format!("Cannot connect to {} API - check your internet connection", provider_name)
                }
            } else {
                format!("{} API error: {}", provider_name, e)
            }
        })?;

        let status = response.status();
        log::info!("[rewrit] LLM response: HTTP {}", status);

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            log::error!("[rewrit] LLM error body: {}", body);
            return Err(http_error_message(provider_name, &self.model, status.as_u16(), &body));
        }

        // Parse response (Ollama uses a different format)
        let raw_content = if matches!(self.provider, LlmProvider::Ollama) {
            let ollama_response: OllamaResponse = response.json().await
                .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;
            ollama_response.message.content
        } else {
            let chat_response: ChatCompletionResponse = response.json().await
                .map_err(|e| format!("Failed to parse {} response: {}", provider_name, e))?;
            chat_response.choices.first()
                .map(|c| c.message.content.clone())
                .ok_or_else(|| format!("Empty response from {}", provider_name))?
        };

        let translated = {
            let cleaned = clean_response(&raw_content);
            format!("{}{}{}", leading_ws, cleaned, trailing_ws)
        };

        let processing_time = start_time.elapsed().as_millis() as i32;

        let quota = QuotaInfo {
            used: 0,
            limit: i32::MAX,
            remaining: i32::MAX,
            resets_at: "Never".to_string(),
        };

        Ok(TranslateResponse {
            success: true,
            original_text: Some(text),
            translated_text: Some(translated),
            detected_language: None,
            target_language: None,
            processing_time_ms: Some(processing_time),
            quota,
            error: None,
            message: None,
        })
    }

    /// Streaming version — calls `on_chunk` with accumulated text as tokens arrive.
    /// Only used in preview mode when streaming is enabled.
    pub async fn execute_prompt_stream<F>(
        &self,
        text: String,
        user_prompt: String,
        on_chunk: F,
    ) -> Result<TranslateResponse, String>
    where
        F: Fn(&str) + Send + 'static,
    {
        let start_time = Instant::now();
        let provider_name = self.provider.display_name();

        let leading_ws = &text[..text.len() - text.trim_start().len()];
        let trailing_ws = &text[text.trim_end().len()..];
        let trimmed_text = text.trim().to_string();
        let user_message = format!("Instruction: {}\n\nText to transform:\n{}", user_prompt, trimmed_text);

        let messages = vec![
            ChatMessage { role: "system".to_string(), content: SYSTEM_PROMPT.to_string() },
            ChatMessage { role: "user".to_string(), content: user_message },
        ];

        let api_url = self.provider.api_url();
        log::info!("[rewrit] LLM stream request: provider={}, model={}", provider_name, self.model);

        let response = if matches!(self.provider, LlmProvider::Ollama) {
            // Ollama streaming
            let request = OllamaRequest {
                model: self.model.clone(),
                messages,
                stream: true,
                options: OllamaOptions { temperature: 0.3, num_predict: 2000 },
            };
            self.client.post(api_url)
                .json(&request)
                .send()
                .await
        } else {
            // OpenAI-compatible streaming
            let mut body = serde_json::json!({
                "model": self.model,
                "messages": messages,
                "temperature": 0.3,
                "max_tokens": 2000,
                "stream": true,
            });
            if let Some(effort) = self.provider.reasoning_effort() {
                body["reasoning_effort"] = effort.into();
            }
            self.client.post(api_url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&body)
                .send()
                .await
        }.map_err(|e| {
            log::error!("[rewrit] LLM stream error (send): {}", e);
            if e.is_timeout() {
                format!("{} timeout", provider_name)
            } else if e.is_connect() {
                if matches!(self.provider, LlmProvider::Ollama) {
                    "Cannot connect to Ollama - make sure it's running".to_string()
                } else {
                    format!("Cannot connect to {} API", provider_name)
                }
            } else {
                format!("{} API error: {}", provider_name, e)
            }
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::error!("[rewrit] LLM stream error: HTTP {} {}", status, body);
            return Err(http_error_message(provider_name, &self.model, status.as_u16(), &body));
        }

        // Read the stream
        let mut accumulated = String::new();
        let mut stream = response.bytes_stream();

        // Buffer for incomplete lines (SSE data can be split across chunks)
        let mut line_buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;
            let chunk_str = String::from_utf8_lossy(&chunk);
            line_buffer.push_str(&chunk_str);

            // Process complete lines
            while let Some(newline_pos) = line_buffer.find('\n') {
                let line = line_buffer[..newline_pos].trim().to_string();
                line_buffer = line_buffer[newline_pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                let content = if matches!(self.provider, LlmProvider::Ollama) {
                    // Ollama: each line is a JSON object with message.content
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                        val["message"]["content"].as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                } else {
                    // OpenAI SSE: lines start with "data: "
                    if line.starts_with("data: ") {
                        let data = &line[6..];
                        if data == "[DONE]" {
                            None
                        } else if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                            val["choices"][0]["delta"]["content"].as_str().map(|s| s.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                if let Some(token) = content {
                    accumulated.push_str(&token);
                    on_chunk(&accumulated);
                }
            }
        }

        let cleaned = clean_response(&accumulated);
        let translated = format!("{}{}{}", leading_ws, cleaned, trailing_ws);
        let processing_time = start_time.elapsed().as_millis() as i32;

        log::info!("[rewrit] LLM stream complete: {}ms, {} chars", processing_time, translated.len());

        let quota = QuotaInfo {
            used: 0, limit: i32::MAX, remaining: i32::MAX, resets_at: "Never".to_string(),
        };

        Ok(TranslateResponse {
            success: true,
            original_text: Some(text),
            translated_text: Some(translated),
            detected_language: None,
            target_language: None,
            processing_time_ms: Some(processing_time),
            quota,
            error: None,
            message: None,
        })
    }
}

/// Turn an HTTP error from a provider into a message the user can act on.
fn http_error_message(provider_name: &str, model: &str, status: u16, body: &str) -> String {
    let api_message = serde_json::from_str::<serde_json::Value>(body).ok().and_then(|v| {
        v["error"]["message"].as_str()
            .or_else(|| v["message"].as_str())
            .or_else(|| v[0]["error"]["message"].as_str())
            .map(|s| s.to_string())
    });
    let model_missing = body.contains("model_not_found")
        || body.contains("does not exist")
        || body.contains("Invalid model")
        || (status == 404 && body.contains("model"));

    if model_missing {
        format!("{}: the model '{}' is not available anymore. Choose another model in Settings › AI Provider.", provider_name, model)
    } else if status == 401 || status == 403 {
        format!("{}: API key rejected (HTTP {}). Check your API key in Settings › AI Provider.", provider_name, status)
    } else if status == 429 {
        format!("{}: rate limit or quota reached. Wait a moment or switch provider.", provider_name)
    } else if status >= 500 {
        format!("{} is having trouble (HTTP {}). Try again in a moment.", provider_name, status)
    } else {
        let detail = api_message.unwrap_or_else(|| body.chars().take(200).collect());
        format!("{} error (HTTP {}): {}", provider_name, status, detail)
    }
}

/// Model lists include speech, moderation and embedding models: keep text generation ones.
fn is_chat_model(id: &str) -> bool {
    const EXCLUDED: [&str; 12] = [
        "whisper", "tts", "orpheus", "guard", "safeguard", "embed",
        "moderation", "ocr", "transcribe", "imagen", "veo", "aqa",
    ];
    let id = id.to_lowercase();
    !EXCLUDED.iter().any(|x| id.contains(x))
}

/// List the chat models the API key can use, sorted by id.
pub async fn list_models(provider: &LlmProvider, api_key: &str) -> Result<Vec<String>, String> {
    let url = match provider {
        LlmProvider::Groq => "https://api.groq.com/openai/v1/models",
        LlmProvider::Mistral => "https://api.mistral.ai/v1/models",
        LlmProvider::Gemini => "https://generativelanguage.googleapis.com/v1beta/openai/models",
        LlmProvider::Ollama => return Ok(vec![provider.default_model().to_string()]),
    };
    let name = provider.display_name();
    let response = reqwest::Client::new()
        .get(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Cannot reach {} to list models: {}", name, e))?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        log::error!("[rewrit] list models failed: HTTP {} {}", status, body);
        return Err(http_error_message(name, "", status.as_u16(), &body));
    }
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Invalid model list from {}: {}", name, e))?;
    let mut models: Vec<String> = json["data"].as_array().into_iter().flatten()
        .filter_map(|m| m["id"].as_str())
        // Gemini prefixes ids with "models/"
        .map(|id| id.trim_start_matches("models/").to_string())
        .filter(|id| is_chat_model(id))
        .collect();
    models.sort();
    models.dedup();
    Ok(models)
}

fn clean_response(response: &str) -> String {
    let mut cleaned = response.to_string();

    // Trim leading/trailing blank lines added by the LLM
    while cleaned.starts_with('\n') || cleaned.starts_with("\r\n") {
        cleaned = cleaned.trim_start_matches("\r\n").trim_start_matches('\n').to_string();
    }
    while cleaned.ends_with('\n') || cleaned.ends_with("\r\n") {
        cleaned = cleaned.trim_end_matches('\n').trim_end_matches("\r\n").to_string();
    }

    // Remove wrapping quotes the LLM sometimes adds
    if cleaned.len() > 1
        && ((cleaned.starts_with('"') && cleaned.ends_with('"'))
        || (cleaned.starts_with('\'') && cleaned.ends_with('\'')))
    {
        cleaned = cleaned[1..cleaned.len()-1].to_string();
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retired_model_says_to_pick_another() {
        let body = r#"{"error":{"message":"The model `llama-3.3-70b-versatile` does not exist or you do not have access to it.","type":"invalid_request_error","code":"model_not_found"}}"#;
        let msg = http_error_message("Groq", "llama-3.3-70b-versatile", 404, body);
        assert!(msg.contains("llama-3.3-70b-versatile"), "{}", msg);
        assert!(msg.contains("Choose another model"), "{}", msg);
    }

    #[test]
    fn rejected_key_says_to_check_key() {
        let msg = http_error_message("Mistral", "mistral-small-latest", 401, r#"{"message":"Unauthorized"}"#);
        assert!(msg.contains("API key"), "{}", msg);
    }

    #[test]
    fn rate_limit_is_explained() {
        let msg = http_error_message("Groq", "m", 429, "");
        assert!(msg.contains("rate limit"), "{}", msg);
    }

    #[test]
    fn unknown_error_keeps_api_message() {
        let body = r#"{"error":{"message":"context too long"}}"#;
        let msg = http_error_message("Gemini", "m", 400, body);
        assert!(msg.contains("context too long"), "{}", msg);
        assert!(msg.contains("400"), "{}", msg);
    }

    #[test]
    fn chat_model_filter_skips_audio_and_guard_models() {
        assert!(is_chat_model("openai/gpt-oss-120b"));
        assert!(is_chat_model("mistral-small-latest"));
        assert!(!is_chat_model("whisper-large-v3"));
        assert!(!is_chat_model("meta-llama/llama-prompt-guard-2-22m"));
        assert!(!is_chat_model("mistral-embed"));
        assert!(!is_chat_model("canopylabs/orpheus-v1-english"));
    }
}
