use log::info;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::redirect::Policy;
use serde_json::Value;
use std::fmt::Display;
use std::time::Duration;
use url::Url;

static APP_NAME: &str = env!("CARGO_PKG_NAME");
static APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum WebApiClientError {
    HeaderCreationError(String),
    ClientCreationError(String),
    PostFailed(String),
    InvalidApiKey(String),
    InvalidInput(String),
    ParseError(String),
}

impl Display for WebApiClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebApiClientError::HeaderCreationError(msg) => {
                write!(f, "Header creation error: {msg}")
            }
            WebApiClientError::ClientCreationError(msg) => {
                write!(f, "Client creation error: {msg}")
            }
            WebApiClientError::PostFailed(msg) => write!(f, "POST request failed: {msg}"),
            WebApiClientError::InvalidApiKey(msg) => write!(f, "Invalid API key: {msg}"),
            WebApiClientError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            WebApiClientError::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

#[derive(Debug)]
pub struct WebApiClient {
    headers: HeaderMap,
    user_agent: String,
    connection_timeout: Option<u64>,
    deadline_timeout: Option<u64>,
    client: Client,
}

impl WebApiClient {
    pub fn new(connection_timeout: Option<u64>, deadline_timeout: Option<u64>) -> Self {
        let mut headers = HeaderMap::new();

        headers.insert(
            "Content-Type",
            HeaderValue::from_str("application/json").expect("failed to create header"),
        );

        let mut web_api_client = WebApiClient {
            headers,
            user_agent: format!("{APP_NAME} {APP_VERSION}"),
            connection_timeout,
            deadline_timeout,
            client: Client::new(),
        };
        web_api_client.client = web_api_client
            .get_client()
            .expect("Failed to create HTTP client");

        web_api_client
    }

    pub fn add_header(
        &mut self,
        key: &str,
        value: String,
    ) -> Result<&WebApiClient, WebApiClientError> {
        let header_name = HeaderName::try_from(key).map_err(|e| {
            WebApiClientError::HeaderCreationError(format!("Invalid header name `{key}`: {e}"))
        })?;

        let header_value = HeaderValue::from_str(&value).map_err(|e| {
            WebApiClientError::HeaderCreationError(format!("Invalid header value for `{key}`: {e}"))
        })?;

        self.headers.insert(header_name, header_value);
        self.client = self.get_client()?;

        Ok(self)
    }

    fn get_client(&mut self) -> Result<Client, WebApiClientError> {
        let mut client_builder = Client::builder()
            .user_agent(self.user_agent.clone())
            .default_headers(self.headers.clone())
            .redirect(Policy::none());

        if let Some(timeout) = self.connection_timeout {
            client_builder = client_builder.connect_timeout(Duration::from_secs(timeout));
        }

        if let Some(timeout) = self.deadline_timeout {
            client_builder = client_builder.timeout(Duration::from_secs(timeout));
        }

        match client_builder.build() {
            Ok(client) => Ok(client),
            Err(e) => Err(WebApiClientError::ClientCreationError(format!(
                "Failed to create HTTP client: {e}"
            ))),
        }
    }

    pub async fn post_request(
        &self,
        url: Url,
        payload: &Value,
    ) -> Result<Value, WebApiClientError> {
        let response = self
            .client
            .post(url)
            .json(payload) // Send as JSON
            .send()
            .await
            .map_err(|e| WebApiClientError::PostFailed(format!("HTTP POST error: {e}")))?;

        let status = response.status();

        let text = response.text().await.map_err(|e| {
            WebApiClientError::PostFailed(format!("Error reading response body: {e}"))
        })?;

        info!("Response status: {status}");
        // debug!("Response: {}", text);

        if !status.is_success() {
            return Err(WebApiClientError::PostFailed(format!(
                "Server returned error status {status}: {text}"
            )));
        }

        serde_json::from_str(&text).map_err(|e| {
            WebApiClientError::PostFailed(format!("Failed to parse JSON response: {e}"))
        })
    }
}
