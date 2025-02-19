pub mod paths;
pub mod types;

use std::fs;
use crate::utils::errors::QError;
use paths::ConfigPaths;
use types::{Config, Provider};

pub struct ConfigManager {
    paths: ConfigPaths,
    config: Config,
}

impl ConfigManager {
    pub fn new(verbose: bool) -> Result<Self, QError> {
        let paths = ConfigPaths::new(verbose)?;
        // Ensure the config directory exists immediately upon creation
        paths.ensure_config_dir()?;
        let config = Self::load_or_create_config(&paths, verbose)?;
        
        Ok(Self { paths, config })
    }

    fn load_or_create_config(paths: &ConfigPaths, verbose: bool) -> Result<Config, QError> {
        if paths.config_file().exists() {
            let contents = fs::read_to_string(paths.config_file())
                .map_err(|e| QError::Io(e))?;
            toml::from_str(&contents)
                .map_err(|e| QError::Config(format!("Failed to parse config: {}", e)))
        } else {
            let config = Config::default();
            Self::save_config(paths, &config)?;
            Ok(config)
        }
    }

    fn save_config(paths: &ConfigPaths, config: &Config) -> Result<(), QError> {
        // Double-check that the directory exists
        paths.ensure_config_dir()?;

        let toml = toml::to_string_pretty(config)
            .map_err(|e| QError::Config(format!("Failed to serialize config: {}", e)))?;
        
        if paths.verbose {
            eprintln!("Debug: Saving config to {:?}", paths.config_file());
            eprintln!("Debug: Config content:\n{}", toml);
        }
        
        // Create parent directories if they don't exist
        if let Some(parent) = paths.config_file().parent() {
            fs::create_dir_all(parent)
                .map_err(|e| QError::Io(e))?;
        }

        fs::write(paths.config_file(), toml)
            .map_err(|e| QError::Io(e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(paths.config_file())
                .map_err(|e| QError::Io(e))?
                .permissions();
            perms.set_mode(0o600); // User read/write only
            fs::set_permissions(paths.config_file(), perms)
                .map_err(|e| QError::Io(e))?;
        }

        Ok(())
    }

    pub fn set_api_key(&mut self, provider: Provider, key: String) -> Result<(), QError> {
        eprintln!("Debug: Setting {} API key", provider);
        
        // Validate key format
        types::validate_api_key(provider, &key)
            .map_err(|e| QError::Config(e))?;

        // Update the key
        match provider {
            Provider::OpenAI => self.config.api_keys.openai = Some(key),
            Provider::Gemini => self.config.api_keys.gemini = Some(key),
        }

        // Save the updated config
        Self::save_config(&self.paths, &self.config)
    }

    pub fn get_api_key(&self, provider: Provider) -> Option<&str> {
        match provider {
            Provider::OpenAI => self.config.api_keys.openai.as_deref(),
            Provider::Gemini => self.config.api_keys.gemini.as_deref(),
        }
    }

    pub fn set_default_provider(&mut self, provider: Provider) -> Result<(), QError> {
        self.config.settings.default_provider = provider;
        Self::save_config(&self.paths, &self.config)
    }

    pub fn set_model(&mut self, provider: Provider, model: String) -> Result<(), QError> {
        self.config.settings.models.insert(provider.as_str().to_string(), model);
        Self::save_config(&self.paths, &self.config)
    }

    pub fn get_model(&self, provider: Provider) -> &str {
        self.config.settings.models
            .get(provider.as_str())
            .map(String::as_str)
            .unwrap_or_else(|| match provider {
                Provider::OpenAI => "gpt-3.5-turbo",
                Provider::Gemini => "gemini-pro",
            })
    }

    #[cfg(test)]
    pub fn with_root(root: std::path::PathBuf, verbose: bool) -> Result<Self, QError> {
        let paths = ConfigPaths::with_root(root);
        paths.ensure_config_dir()?;
        let config = Self::load_or_create_config(&paths, verbose)?;
        Ok(Self { paths, config })
    }
}
