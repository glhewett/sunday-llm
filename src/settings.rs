use serde::Deserialize;
use std::fs::read_to_string;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

#[derive(Deserialize, Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl From<&str> for Method {
    fn from(method: &str) -> Self {
        match method.to_lowercase().as_str() {
            "get" => Method::Get,
            "post" => Method::Post,
            "put" => Method::Put,
            "delete" => Method::Delete,
            _ => panic!("Invalid HTTP method"),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Endpoint {
    pub server: String,
    pub template: String,
    pub system_prompt: String,
    pub user_prompt: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EndpointConfig {
    pub path: String,
    pub template: String,
    pub server: String,
    pub system_prompt: String,
    pub user_prompt: String,
}

impl EndpointConfig {
    pub fn get_public(&self) -> Endpoint {
        Endpoint {
            server: self.server.clone(),
            template: self.template.clone(),
            system_prompt: self.system_prompt.clone(),
            user_prompt: self.user_prompt.clone(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub model: String,
    pub api_type: String,
    pub base_api_url: String,
    pub secret: Option<String>,
    pub connection_timeout: Option<u64>,
    pub deadline_timeout: Option<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub servers: Vec<ServerConfig>,
    pub endpoints: Vec<EndpointConfig>,
}

impl Settings {
    pub fn get_endpoint_by_path(&self, path: &str) -> Result<Endpoint, Error> {
        for endpoint in &self.endpoints {
            if endpoint.path == path {
                return Ok(endpoint.get_public());
            }
        }
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Endpoint {path} not found"),
        ))
    }

    pub fn get_server_config_by_name(&self, name: &str) -> Result<ServerConfig, Error> {
        for server in &self.servers {
            if server.name == name {
                return Ok(server.clone());
            }
        }
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Server {name} not found"),
        ))
    }

    pub fn load(path: &PathBuf) -> Result<Settings, Error> {
        if !path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Configuration was not found.",
            ));
        }

        let config_file_contents = match read_to_string(path) {
            Ok(contents) => contents,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Unable to read configuration. {e}"),
                ));
            }
        };

        let settings: Settings = match toml::from_str(config_file_contents.as_str()) {
            Ok(token) => token,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Unable to parse configuration. {e}"),
                ));
            }
        };

        Ok(settings)
    }
}
