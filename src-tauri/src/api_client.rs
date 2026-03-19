use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = "http://localhost:8000/api/v1";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuotaInfo {
    pub used: i32,
    pub limit: i32,
    pub remaining: i32,
    pub resets_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranslateResponse {
    pub success: bool,
    pub original_text: Option<String>,
    pub translated_text: Option<String>,
    pub detected_language: Option<String>,
    pub target_language: Option<String>,
    pub processing_time_ms: Option<i32>,
    pub quota: QuotaInfo,
    pub error: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaResponse {
    pub quota: QuotaInfo,
    pub is_pro: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub platform: String,
    pub app_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub device_id: String,
    pub quota: QuotaInfo,
}

#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
    device_id: String,
}

impl ApiClient {
    pub fn new(device_id: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            device_id,
        }
    }

    pub async fn register(&self, platform: String) -> Result<RegisterResponse, String> {
        let url = format!("{}/register", API_BASE_URL);
        let request = RegisterRequest {
            platform,
            app_version: "1.0.0".to_string(),
        };

        match self
            .client
            .post(&url)
            .header("X-Device-ID", &self.device_id)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
        {
            Ok(response) => match response.json::<RegisterResponse>().await {
                Ok(data) => Ok(data),
                Err(e) => Err(format!("Failed to parse response: {}", e)),
            },
            Err(e) => Err(format!("Request failed: {}", e)),
        }
    }

    pub async fn fetch_quota(&self) -> Result<QuotaResponse, String> {
        let url = format!("{}/quota", API_BASE_URL);

        match self
            .client
            .get(&url)
            .header("X-Device-ID", &self.device_id)
            .send()
            .await
        {
            Ok(response) => match response.json::<QuotaResponse>().await {
                Ok(data) => Ok(data),
                Err(e) => Err(format!("Failed to parse response: {}", e)),
            },
            Err(_e) => Err(format!("No internet connection")),
        }
    }

    pub async fn report_usage(&self) -> Result<QuotaInfo, String> {
        let url = format!("{}/usage/report", API_BASE_URL);

        match self
            .client
            .post(&url)
            .header("X-Device-ID", &self.device_id)
            .send()
            .await
        {
            Ok(response) => {
                if response.status() == 403 {
                    return Err("quota_exceeded".to_string());
                }
                #[derive(Deserialize)]
                struct UsageResponse {
                    quota: QuotaInfo,
                }
                match response.json::<UsageResponse>().await {
                    Ok(data) => Ok(data.quota),
                    Err(e) => Err(format!("Failed to parse response: {}", e)),
                }
            }
            Err(_) => Err("Backend unavailable".to_string()),
        }
    }

    pub async fn reset_quota(&self) -> Result<QuotaInfo, String> {
        let url = format!("{}/quota/reset", API_BASE_URL);

        match self
            .client
            .post(&url)
            .header("X-Device-ID", &self.device_id)
            .send()
            .await
        {
            Ok(response) => {
                #[derive(Deserialize)]
                struct ResetResponse {
                    quota: QuotaInfo,
                }

                match response.json::<ResetResponse>().await {
                    Ok(data) => Ok(data.quota),
                    Err(e) => Err(format!("Failed to parse response: {}", e)),
                }
            }
            Err(e) => Err(format!("Request failed: {}", e)),
        }
    }
}
