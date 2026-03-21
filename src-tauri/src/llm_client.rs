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
            LlmProvider::Groq => "llama-3.3-70b-versatile",
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

impl LlmClient {
    pub fn new(provider: LlmProvider, api_key: String) -> Self {
        let model = provider.default_model().to_string();
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

    pub async fn execute_prompt(&self, text: String, user_prompt: String) -> Result<TranslateResponse, String> {
        let start_time = Instant::now();
        let provider_name = self.provider.display_name();

        let system_prompt = "You are a helpful text transformation assistant. Apply the user's instruction to the provided text. Return ONLY the transformed result, nothing else. No explanations, no quotes, no parentheses. Preserve the exact formatting: keep all line breaks and internal whitespace structure.".to_string();

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
        println!("[rewrit] LLM request: provider={}, model={}", provider_name, self.model);
        #[cfg(debug_assertions)]
        println!("[rewrit] LLM debug: url={}, key={}, text={} chars", api_url, masked_key, text.len());

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
            };
            self.client.post(api_url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&request)
                .send()
                .await
        }.map_err(|e| {
            println!("[rewrit] LLM error (send): {}", e);
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
        println!("[rewrit] LLM response: HTTP {}", status);

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            println!("[rewrit] LLM error body: {}", body);
            return Err(format!("{} API HTTP {}: {}", provider_name, status, body));
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

        let system_prompt = "You are a helpful text transformation assistant. Apply the user's instruction to the provided text. Return ONLY the transformed result, nothing else. No explanations, no quotes, no parentheses. Preserve the exact formatting: keep all line breaks and internal whitespace structure.".to_string();

        let leading_ws = &text[..text.len() - text.trim_start().len()];
        let trailing_ws = &text[text.trim_end().len()..];
        let trimmed_text = text.trim().to_string();
        let user_message = format!("Instruction: {}\n\nText to transform:\n{}", user_prompt, trimmed_text);

        let messages = vec![
            ChatMessage { role: "system".to_string(), content: system_prompt },
            ChatMessage { role: "user".to_string(), content: user_message },
        ];

        let api_url = self.provider.api_url();
        println!("[rewrit] LLM stream request: provider={}, model={}", provider_name, self.model);

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
            let body = serde_json::json!({
                "model": self.model,
                "messages": messages,
                "temperature": 0.3,
                "max_tokens": 2000,
                "stream": true,
            });
            self.client.post(api_url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&body)
                .send()
                .await
        }.map_err(|e| {
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
            return Err(format!("{} API HTTP {}: {}", provider_name, status, body));
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

        println!("[rewrit] LLM stream complete: {}ms, {} chars", processing_time, translated.len());

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
