use crate::settings::ServerConfig;
use crate::web_api_client::{WebApiClient, WebApiClientError};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    system: Option<String>,
    raw: bool,
    stream: bool,
    temperature: Option<f32>,
    suffix: Option<String>,
    format: Option<String>,
    keep_alive: Option<String>,
}

// set default value
impl Default for GenerateRequest {
    fn default() -> Self {
        Self {
            model: String::new(),
            prompt: String::new(),
            raw: false,
            stream: false,
            temperature: Some(0.3),
            system: None,
            suffix: None,
            format: None,
            keep_alive: Some("10m".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct GenerateResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    pub done_reason: Option<String>,
    pub context: Vec<usize>,
    pub total_duration: Option<u64>,
    pub load_duration: Option<u64>,
    pub prompt_eval_count: Option<usize>,
    pub prompt_eval_duration: Option<u64>,
    pub eval_count: Option<usize>,
    pub eval_duration: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub prompt: String,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingResponse {
    pub _embedding: Vec<f32>,
}

#[derive(Debug)]
pub struct OllamaClient {
    auth_api_client: WebApiClient,
    base_url: Url,
}

impl OllamaClient {
    pub fn new(setting: &ServerConfig, api_key: Option<String>) -> Result<Self, WebApiClientError> {
        let api_key: String = api_key.unwrap_or_default();

        debug!(
            "Setting Connection Timeout: {}",
            setting.connection_timeout.unwrap_or(u64::MAX)
        );

        debug!(
            "Setting Deadline Timeout: {}",
            setting.deadline_timeout.unwrap_or(u64::MAX)
        );

        let mut auth_api_client =
            WebApiClient::new(setting.connection_timeout, setting.deadline_timeout);

        match auth_api_client.add_header("Authorization", format!("Bearer {api_key}")) {
            Ok(client) => client,
            Err(e) => {
                return Err(WebApiClientError::InvalidApiKey(format!(
                    "Failed to add header to WebApiClient: {e}"
                )));
            }
        };

        let base_url = match Url::parse(&setting.base_api_url) {
            Ok(url) => url,
            Err(e) => {
                return Err(WebApiClientError::InvalidInput(format!(
                    "Failed to parse base API URL ({}): {}",
                    setting.base_api_url, e
                )));
            }
        };

        Ok(Self {
            auth_api_client,
            base_url,
        })
    }

    pub async fn generate(
        &self,
        model: &str,
        system_prompt: &str,
        prompt: &str,
        json: bool,
    ) -> Result<GenerateResponse, WebApiClientError> {
        let format = if json { Some("json".to_string()) } else { None };

        let url = match self.base_url.join("/api/generate") {
            Ok(url) => url,
            Err(e) => {
                return Err(WebApiClientError::InvalidInput(format!("Invalid URL: {e}")));
            }
        };

        let json_value = self
            .auth_api_client
            .post_request(
                url,
                &json!(GenerateRequest {
                    model: model.to_string(),
                    system: Some(system_prompt.to_string()),
                    prompt: prompt.to_string(),
                    format,
                    ..Default::default()
                }),
            )
            .await?;

        let parsed: GenerateResponse = match serde_json::from_value(json_value) {
            Ok(response) => response,
            Err(e) => {
                return Err(WebApiClientError::ParseError(format!(
                    "Failed to parse generate response: {e}"
                )));
            }
        };

        Ok(parsed)
    }

    pub async fn _embeddings(
        &self,
        model: &str,
        text: &str,
    ) -> Result<EmbeddingResponse, WebApiClientError> {
        let url = match self.base_url.join("/api/embeddings") {
            Ok(url) => url,
            Err(e) => {
                return Err(WebApiClientError::InvalidInput(format!("Invalid URL: {e}")));
            }
        };

        let json_value = self
            .auth_api_client
            .post_request(
                url,
                &json!(EmbeddingRequest {
                    model: model.to_string(),
                    prompt: text.to_string(),
                }),
            )
            .await?;

        let parsed: EmbeddingResponse = match serde_json::from_value(json_value) {
            Ok(parsed) => parsed,
            Err(e) => {
                return Err(WebApiClientError::ParseError(format!(
                    "Failed to parse response: {e}"
                )));
            }
        };

        Ok(parsed)
    }
}
