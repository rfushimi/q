use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use colored::*;

use crate::utils::errors::QError;
use crate::utils::format::format_markdown;
use crate::config::types::Provider;
use crate::api::{openai::OpenAIClient, gemini::GeminiClient, LLMApi};
use crate::context::{ContextConfig, ContextProvider};
use crate::context::directory::DirectoryProvider;
use crate::context::file::FileProvider;
use crate::context::history::HistoryProvider;
use crate::commands::suggest::process_command_query;
use crate::core::{QueryEngine, QueryConfig};
use crate::config::ConfigManager;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Verbosity {
    /// Concise responses with essential information only
    Concise,
    /// Balanced responses with moderate detail
    Normal,
    /// Detailed responses with comprehensive information
    Detailed,
}

impl Default for Verbosity {
    fn default() -> Self {
        Self::Concise
    }
}

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

    /// Get command suggestions
    #[arg(long = "cmd", short = 'C')]
    pub cmd_suggest: bool,

    /// Disable response caching
    #[arg(long = "no-cache")]
    pub no_cache: bool,

    /// Maximum retry attempts
    #[arg(long = "retries", default_value = "3")]
    pub max_retries: u32,

    /// Show debug information
    #[arg(long = "debug")]
    pub debug: bool,

    /// Show verbose output
    #[arg(long = "verbose", short = 'v')]
    pub verbose: bool,

    /// Select LLM provider (openai or gemini)
    #[arg(long = "provider", short = 'P', default_value = "gemini")]
    pub provider: String,

    /// Select model name (e.g., gemini-pro, gpt-3.5-turbo)
    #[arg(long = "model", short = 'M')]
    pub model: Option<String>,

    /// Control response verbosity
    #[arg(long = "detail", short = 'd', value_enum, default_value = "concise")]
    pub verbosity: Verbosity,

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

    /// Set default LLM provider
    SetProvider {
        /// The LLM provider (openai or gemini)
        #[arg(help = "The LLM provider (openai or gemini)")]
        provider: String,
    },

    /// Set model for LLM provider
    SetModel {
        /// The LLM provider (openai or gemini)
        #[arg(help = "The LLM provider (openai or gemini)")]
        provider: String,

        /// The model name to set
        #[arg(help = "The model name to set")]
        model: String,
    },
}

impl Cli {
    pub async fn run(&self) -> Result<(), QError> {
        if let Some(cmd) = &self.command {
            cmd.execute(&self)?;
            return Ok(());
        }

        // Handle the prompt if present
        if let Some(prompt) = &self.prompt {
            // Handle command suggestions
            if self.cmd_suggest {
                let suggestions = process_command_query(prompt)
                    .await
                    .map_err(|e| QError::Command(format!("Failed to get command suggestions: {}", e)))?;
                println!("{}", format_markdown(&suggestions));
                return Ok(());
            }

            // Get provider from command line
            let provider = Provider::try_from(self.provider.as_str())
                .map_err(|e| QError::Config(format!("Invalid provider: {}", e)))?;

            // Get API key from config
            let config = ConfigManager::new(self.verbose)?;
            let api_key = config.get_api_key(provider)
                .ok_or_else(|| QError::Config(format!("{} API key not found. Use 'q set-key {} <key>' to set it.", provider, provider)))?;

            // Gather context if requested
            let mut context = String::new();
            let context_config = ContextConfig::default();

            // Add shell history context
            if self.history {
                let provider = HistoryProvider::new(context_config.clone());
                let history_context = provider.get_context().await
                    .map_err(|e| QError::Context(format!("Failed to get history context: {}", e)))?;
                context.push_str(&history_context.content);
                context.push_str("\n\n");
            }

            // Add directory listing context
            if self.directory {
                let current_dir = env::current_dir()
                    .map_err(|e| QError::Context(format!("Failed to get current directory: {}", e)))?;
                let provider = DirectoryProvider::new(current_dir, context_config.clone());
                let dir_context = provider.get_context().await
                    .map_err(|e| QError::Context(format!("Failed to get directory context: {}", e)))?;
                context.push_str(&dir_context.content);
                context.push_str("\n\n");
            }

            // Add file content context
            if let Some(file_path) = &self.file {
                let provider = FileProvider::new(file_path.clone(), context_config.clone());
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

            // Create client based on provider
            let client: Arc<dyn LLMApi> = match provider {
                Provider::OpenAI => {
                    let mut builder = OpenAIClient::builder(api_key.to_string());
                    if let Some(model) = &self.model {
                        builder = builder.with_model(model.clone());
                    }
                    builder = builder.with_verbosity(self.verbosity);
                    Arc::new(builder.build())
                }
                Provider::Gemini => {
                    let mut builder = GeminiClient::builder(api_key.to_string());
                    if let Some(model) = &self.model {
                        builder = builder.with_model(model.clone());
                    }
                    builder = builder.with_verbosity(self.verbosity);
                    Arc::new(builder.build())
                }
            };

            // Show connecting message with provider and model info
            eprintln!("{}", format!("provider: {}, model: {}", provider, client.model()).dimmed());

            // Create query engine config
            let config = QueryConfig {
                max_retries: self.max_retries,
                show_progress: !self.debug,
                cache_ttl: Duration::from_secs(3600),
                max_cache_size: 1000,
                retry_delay: Duration::from_secs(1),
                max_retry_delay: Duration::from_secs(30),
                verbosity: self.verbosity,
            };

            // Create query engine
            let mut engine = QueryEngine::new(client, config);

            // Send the query through the engine
            let response = engine.query(&final_prompt)
                .await
                .map_err(|e| QError::Core(format!("Query failed: {}", e)))?;

            println!("{}", format_markdown(&response));
            return Ok(());
        }

        // If we get here, no prompt was provided
        Err(QError::Usage("No prompt provided. Use --help for usage information.".into()))
    }
}

impl Commands {
    pub fn execute(&self, cli: &Cli) -> Result<(), QError> {
        match self {
            Commands::SetKey { provider, key } => {
                let provider = Provider::try_from(provider.as_str())
                    .map_err(|e| QError::Config(e))?;
                
                let mut config = ConfigManager::new(cli.verbose)?;
                config.set_api_key(provider, key.clone())?;
                
                println!("{}", format_markdown(&format!("# API key for {} has been set successfully", provider)));
                Ok(())
            }
            Commands::SetProvider { provider } => {
                let provider = Provider::try_from(provider.as_str())
                    .map_err(|e| QError::Config(e))?;
                
                let mut config = ConfigManager::new(cli.verbose)?;
                config.set_default_provider(provider)?;
                
                println!("{}", format_markdown(&format!("# Default provider has been set to {}", provider)));
                Ok(())
            }
            Commands::SetModel { provider, model } => {
                let provider = Provider::try_from(provider.as_str())
                    .map_err(|e| QError::Config(e))?;
                
                let mut config = ConfigManager::new(cli.verbose)?;
                config.set_model(provider, model.clone())?;
                
                println!("{}", format_markdown(&format!("# Model for {} has been set to {}", provider, model)));
                Ok(())
            }
        }
    }
}

fn validate_prompt(s: &str) -> Result<String, String> {
    // If the input looks like a command (starts with '-' or contains subcommand names),
    // reject it to ensure proper error handling
    if s.starts_with('-') || s == "set-key" || s == "set-provider" || s == "set-model" {
        Err(format!("'{}' is not a valid prompt. Use --help to see available commands.", s))
    } else {
        Ok(s.to_string())
    }
}
