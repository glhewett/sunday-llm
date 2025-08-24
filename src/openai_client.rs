use crate::settings::ServerConfig;
use crate::web_api_client::WebApiClient;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt::Display;
use url::Url;

#[derive(Debug)]
pub enum OpenAiClientError {
    InvalidApiKey(String),
    InvalidInput(String),
    CompletionFailed(String),
}

impl Display for OpenAiClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAiClientError::InvalidApiKey(msg) => write!(f, "Invalid API Key: {}", msg),
            OpenAiClientError::InvalidInput(msg) => write!(f, "Invalid Input: {}", msg),
            OpenAiClientError::CompletionFailed(msg) => write!(f, "Completion Failed: {}", msg),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct NewChatCompletion {
    model: String,
    system: Option<String>,
    prompt: String,
    format: Option<String>,
}

impl Default for NewChatCompletion {
    fn default() -> Self {
        Self {
            model: String::new(),
            system: None,
            prompt: String::new(),
            format: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Debug)]
pub struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionChoice {
    pub index: i64,
    pub message: ChatMessage,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionResponse {
    choices: Vec<ChatCompletionChoice>,
}

pub struct OpenAiClient {
    auth_api_client: WebApiClient,
    base_url: Url,
}

impl OpenAiClient {
    pub fn new(
        setting: &ServerConfig,
        api_key: Option<&String>,
    ) -> Result<Self, OpenAiClientError> {
        // check if the API key is empty
        if api_key.unwrap_or(&String::new()).is_empty() {
            return Err(OpenAiClientError::InvalidApiKey(format!(
                "API key cannot be empty"
            )));
        }
        let api_key = api_key.unwrap();

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

        match auth_api_client.add_header("Authorization", format!("Bearer {}", api_key)) {
            Ok(client) => client,
            Err(e) => {
                return Err(OpenAiClientError::InvalidApiKey(format!(
                    "Failed to add header to WebApiClient: {}",
                    e
                )))
            }
        };

        let base_url = match Url::parse(&setting.base_api_url) {
            Ok(url) => url,
            Err(e) => {
                return Err(OpenAiClientError::InvalidInput(format!(
                    "Failed to parse base API URL ({}): {}",
                    setting.base_api_url, e
                )))
            }
        };

        Ok(Self {
            auth_api_client,
            base_url,
        })
    }
    pub async fn generate(
        &self,
        model: &String,
        system_prompt: &String,
        prompt: &String,
        json: bool,
    ) -> Result<String, OpenAiClientError> {
        self.chat_completion(model, system_prompt, prompt, json)
            .await
    }

    pub async fn chat_completion(
        &self,
        model: &String,
        system_prompt: &String,
        prompt: &String,
        json: bool,
    ) -> Result<String, OpenAiClientError> {
        let _format = if json { Some("json".to_string()) } else { None };

        let url = match self.base_url.join("/v1/chat/completions") {
            Ok(url) => url,
            Err(e) => {
                return Err(OpenAiClientError::InvalidInput(format!(
                    "Invalid URL: {}",
                    e
                )))
            }
        };

        let request = ChatCompletionRequest {
            model: model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.clone(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.clone(),
                },
            ],
        };

        let json_value: &Value = match self
            .auth_api_client
            .post_request(url, &json!(request))
            .await
        {
            Ok(json_value) => &json_value.to_owned(),
            Err(e) => {
                return Err(OpenAiClientError::CompletionFailed(format!(
                    "POST request failed: {}",
                    e
                )))
            }
        };

        let parsed: ChatCompletionResponse = match serde_json::from_value(json_value.clone()) {
            Ok(response) => response,
            Err(e) => {
                return Err(OpenAiClientError::CompletionFailed(format!(
                    "Failed to parse chat_completion response: {}",
                    e
                )))
            }
        };

        // find the message from "assistant"
        let response: Option<&ChatCompletionChoice> = parsed
            .choices
            .iter()
            .find(|o| o.message.role == "assistant");

        if response.is_none() {
            return Err(OpenAiClientError::CompletionFailed(
                "No assistant response found".to_string(),
            ));
        }

        Ok(response.unwrap().message.content.clone())
    }
}
