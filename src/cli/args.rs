use clap::{Parser, Subcommand};
use crate::utils::errors::QError;
use crate::config::types::Provider;
use crate::api::LLMApi;
use crate::api::openai::OpenAIClient;
use crate::context::{ContextConfig, ContextProvider, ContextType};
use crate::context::directory::DirectoryProvider;
use crate::context::file::FileProvider;
use crate::context::history::HistoryProvider;
use std::path::PathBuf;
use std::env;

#[derive(Parser)]
#[command(name = "q")]
#[command(author, version, about = "CLI tool for querying LLMs", long_about = None)]
pub struct Cli {
    /// The prompt to send to the LLM
    #[arg(help = "The prompt to send to the LLM", value_parser = validate_prompt)]
    pub prompt: Option<String>,

    /// Include shell history context
    #[arg(long = "hist", short = 'H')]
    pub history: bool,

    /// Include current directory listing
    #[arg(long = "here", short = 'D')]
    pub directory: bool,

    /// Include file content
    #[arg(long = "file", short = 'F', value_name = "FILE")]
    pub file: Option<PathBuf>,

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

impl Cli {
    pub async fn run(&self) -> Result<(), QError> {
        if let Some(cmd) = &self.command {
            cmd.execute()?;
            return Ok(());
        }

        // Handle the prompt if present
        if let Some(prompt) = &self.prompt {
            let home = PathBuf::from(env!("HOME"));
            let key_path = home.join("keys").join("openai.key");
            let api_key = std::fs::read_to_string(key_path)
                .map_err(|e| QError::Config(format!("Failed to read API key: {}", e)))?;
            let api_key = api_key.trim().to_string();

            // Create OpenAI client
            let client = OpenAIClient::new(api_key);

            // Validate the key before using
            client.validate_key().await
                .map_err(|e| QError::Api(format!("API key validation failed: {}", e)))?;

            // Gather context if requested
            let mut context = String::new();
            let config = ContextConfig::default();

            // Add shell history context
            if self.history {
                let provider = HistoryProvider::new(config.clone());
                let history_context = provider.get_context().await
                    .map_err(|e| QError::Context(format!("Failed to get history context: {}", e)))?;
                context.push_str(&history_context.content);
                context.push_str("\n\n");
            }

            // Add directory listing context
            if self.directory {
                let current_dir = env::current_dir()
                    .map_err(|e| QError::Context(format!("Failed to get current directory: {}", e)))?;
                let provider = DirectoryProvider::new(current_dir, config.clone());
                let dir_context = provider.get_context().await
                    .map_err(|e| QError::Context(format!("Failed to get directory context: {}", e)))?;
                context.push_str(&dir_context.content);
                context.push_str("\n\n");
            }

            // Add file content context
            if let Some(file_path) = &self.file {
                let provider = FileProvider::new(file_path.clone(), config.clone());
                let file_context = provider.get_context().await
                    .map_err(|e| QError::Context(format!("Failed to get file context: {}", e)))?;
                context.push_str(&file_context.content);
                context.push_str("\n\n");
            }

            // Build the final prompt with context
            let final_prompt = if context.is_empty() {
                prompt.clone()
            } else {
                format!("Context:\n{}\nPrompt: {}", context.trim(), prompt)
            };

            // Send the query
            let response = client.send_query(&final_prompt).await
                .map_err(|e| QError::Api(format!("Query failed: {}", e)))?;

            // Print the response
            println!("{}", response);
            return Ok(());
        }

        // If we get here, no prompt was provided
        Err(QError::Usage("No prompt provided. Use --help for usage information.".into()))
    }
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
