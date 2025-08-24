use serde::Deserialize;
use std::fs::read_to_string;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
struct SecretsConfig {
    secret: Vec<SecretConfig>,
}

#[derive(Deserialize, Debug, Clone)]
struct SecretConfig {
    name: String,
    value: String,
}

#[derive(Debug, Clone)]
pub struct Secret {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Secrets {
    config: SecretsConfig,
}

impl Secrets {
    pub fn load(path: &PathBuf) -> Result<Secrets, Error> {
        if !path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Configuration was not found.",
            ));
        }
        let config = read_config(path)?;

        Ok(Secrets { config })
    }

    pub fn get_by_name(&self, name: &str) -> Result<Secret, Error> {
        for secret_config in &self.config.secret {
            if secret_config.name == name {
                return Ok(secret_config.get_public());
            }
        }
        Err(Error::new(
            ErrorKind::NotFound,
            format!("Secret {} not found", name),
        ))
    }
}

impl SecretConfig {
    // get the persona by cleaning the text
    pub fn get_public(&self) -> Secret {
        Secret {
            name: self.name.clone(),
            value: self.value.clone(),
        }
    }
}

fn read_config(path: &PathBuf) -> Result<SecretsConfig, Error> {
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
                format!("Unable to read configuration. {}", e),
            ))
        }
    };

    let settings: SecretsConfig = match toml::from_str(config_file_contents.as_str()) {
        Ok(token) => token,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Unable to parse configuration. {}", e),
            ))
        }
    };

    Ok(settings)
}
