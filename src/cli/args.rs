use clap::{Parser, Subcommand};
use crate::utils::errors::QError;
use crate::config::types::Provider;

#[derive(Parser)]
#[command(name = "q")]
#[command(author, version, about = "CLI tool for querying LLMs", long_about = None)]
pub struct Cli {
    /// The prompt to send to the LLM
    #[arg(help = "The prompt to send to the LLM", value_parser = validate_prompt)]
    pub prompt: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Set API key for LLM service
    SetKey {
        /// The LLM provider (openai or gemini)
        #[arg(help = "The LLM provider (openai or gemini)")]
        provider: String,

        /// The API key to set
        #[arg(help = "The API key to set")]
        key: String,
    },
}

impl Commands {
    pub fn execute(&self) -> Result<(), QError> {
        match self {
            Commands::SetKey { provider, key } => {
                let provider = Provider::try_from(provider.as_str())
                    .map_err(|e| QError::Config(e))?;
                
                let mut config = crate::config::ConfigManager::new()?;
                config.set_api_key(provider, key.clone())?;
                
                println!("API key for {} has been set successfully", provider);
                Ok(())
            }
        }
    }
}

fn validate_prompt(s: &str) -> Result<String, String> {
    // If the input looks like a command (starts with '-' or contains subcommand names),
    // reject it to ensure proper error handling
    if s.starts_with('-') || s == "set-key" {
        Err(format!("'{}' is not a valid prompt. Use --help to see available commands.", s))
    } else {
        Ok(s.to_string())
    }
}
