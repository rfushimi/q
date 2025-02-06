use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use shellexpand::tilde;

use super::{ContextConfig, ContextData, ContextError, ContextProvider, ContextResult, ContextType};
use super::validate_size;

pub struct HistoryProvider {
    config: ContextConfig,
}

impl HistoryProvider {
    pub fn new(config: ContextConfig) -> Self {
        Self { config }
    }

    fn get_history_path() -> ContextResult<PathBuf> {
        let home = PathBuf::from(env!("HOME"));
        let history_path = home.join(".zsh_history");
        
        if !history_path.exists() {
            return Err(ContextError::History(
                "Zsh history file not found".to_string()
            ));
        }
        
        Ok(history_path)
    }

    async fn read_history(&self) -> ContextResult<String> {
        let history_path = Self::get_history_path()?;

        // Check if we have permission to read
        let metadata = fs::metadata(&history_path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    ContextError::PermissionDenied(history_path.clone())
                }
                _ => ContextError::Io(e),
            })?;

        // Check file size before reading
        validate_size(
            metadata.len() as usize,
            self.config.max_size,
            "Shell history"
        )?;

        // Read history file
        let content = fs::read_to_string(&history_path)
            .await
            .map_err(ContextError::Io)?;

        // Parse and format history entries
        let mut output = String::from("Recent shell history:\n\n");
        
        // Process history entries
        for line in content.lines().rev().take(100) {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Parse Zsh history format
            // Format: ": timestamp:duration;command"
            if let Some(cmd) = line.split(';').last() {
                output.push_str(&format!("{}\n", cmd.trim()));
            }
        }

        Ok(output)
    }
}

#[async_trait]
impl ContextProvider for HistoryProvider {
    fn context_type(&self) -> ContextType {
        ContextType::History
    }

    async fn get_context(&self) -> ContextResult<ContextData> {
        let content = self.read_history().await?;
        
        Ok(ContextData {
            context_type: self.context_type(),
            content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    fn create_test_history() -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, ": 1707000000:0;ls -la").unwrap();
        writeln!(temp_file, ": 1707000001:0;git status").unwrap();
        writeln!(temp_file, ": 1707000002:0;cargo build").unwrap();
        temp_file
    }

    #[tokio::test]
    async fn test_history_reading() {
        let temp_file = create_test_history();
        
        let config = ContextConfig {
            max_size: 1024,
            include_hidden: false,
            max_depth: None,
        };

        let provider = HistoryProvider::new(config);
        
        // Temporarily override the history path for testing
        std::env::set_var("HOME", temp_file.path().parent().unwrap());
        std::fs::rename(temp_file.path(), temp_file.path().with_file_name(".zsh_history")).unwrap();

        let context = provider.get_context().await.unwrap();

        assert!(context.content.contains("ls -la"));
        assert!(context.content.contains("git status"));
        assert!(context.content.contains("cargo build"));
    }

    #[tokio::test]
    async fn test_size_limit() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let large_history = ": 1707000000:0;".to_string() + &"x".repeat(1000);
        writeln!(temp_file, "{}", large_history).unwrap();

        let config = ContextConfig {
            max_size: 100, // Small limit
            include_hidden: false,
            max_depth: None,
        };

        let provider = HistoryProvider::new(config);
        
        // Temporarily override the history path for testing
        std::env::set_var("HOME", temp_file.path().parent().unwrap());
        std::fs::rename(temp_file.path(), temp_file.path().with_file_name(".zsh_history")).unwrap();

        let result = provider.get_context().await;
        assert!(matches!(result, Err(ContextError::TooLarge(_))));
    }
}
