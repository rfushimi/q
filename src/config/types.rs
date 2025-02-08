use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api_keys: ApiKeys,
    #[serde(default)]
    pub settings: Settings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_keys: ApiKeys::default(),
            settings: Settings::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ApiKeys {
    pub openai: Option<String>,
    pub gemini: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Gemini,
}

impl Default for Provider {
    fn default() -> Self {
        Self::Gemini
    }
}

impl Provider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai",
            Provider::Gemini => "gemini",
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for Provider {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Provider::OpenAI),
            "gemini" => Ok(Provider::Gemini),
            _ => Err(format!("Unknown provider: {}. Valid providers are: openai, gemini", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub default_provider: Provider,
    #[serde(default = "default_models")]
    pub models: HashMap<String, String>,
    pub temperature: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_provider: Provider::Gemini,
            models: default_models(),
            temperature: 0.7,
        }
    }
}

fn default_models() -> HashMap<String, String> {
    let mut models = HashMap::new();
    models.insert("openai".to_string(), "gpt-3.5-turbo".to_string());
    models.insert("gemini".to_string(), "gemini-pro".to_string());
    models
}

// Basic key format validation
pub fn validate_api_key(provider: Provider, key: &str) -> Result<(), String> {
    match provider {
        Provider::OpenAI => {
            if !key.starts_with("sk-") {
                return Err("OpenAI API key must start with 'sk-'".to_string());
            }
            if key.len() < 40 {
                return Err("OpenAI API key is too short".to_string());
            }
        }
        Provider::Gemini => {
            if key.len() < 20 {
                return Err("Gemini API key is too short".to_string());
            }
        }
    }
    Ok(())
}
