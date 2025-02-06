use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;

use super::{ContextConfig, ContextData, ContextError, ContextProvider, ContextResult, ContextType};
use super::{format_path_for_display, validate_size};

pub struct FileProvider {
    path: PathBuf,
    config: ContextConfig,
}

impl FileProvider {
    pub fn new(path: PathBuf, config: ContextConfig) -> Self {
        Self { path, config }
    }

    async fn read_file_content(&self) -> ContextResult<String> {
        // Check if file exists
        if !self.path.exists() {
            return Err(ContextError::FileNotFound(self.path.clone()));
        }

        // Check if we have permission to read
        let metadata = fs::metadata(&self.path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::PermissionDenied => ContextError::PermissionDenied(self.path.clone()),
                _ => ContextError::Io(e),
            })?;

        // Check file size before reading
        validate_size(
            metadata.len() as usize,
            self.config.max_size,
            "File content"
        )?;

        // Read file content
        let content = fs::read_to_string(&self.path)
            .await
            .map_err(ContextError::Io)?;

        // Format the output with file information
        let output = format!(
            "File: {}\nSize: {} bytes\n\nContent:\n{}\n",
            format_path_for_display(&self.path),
            metadata.len(),
            content
        );

        Ok(output)
    }
}

#[async_trait]
impl ContextProvider for FileProvider {
    fn context_type(&self) -> ContextType {
        ContextType::File(self.path.clone())
    }

    async fn get_context(&self) -> ContextResult<ContextData> {
        let content = self.read_file_content().await?;
        
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
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    #[tokio::test]
    async fn test_read_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content").unwrap();

        let config = ContextConfig {
            max_size: 1024,
            include_hidden: false,
            max_depth: None,
        };

        let provider = FileProvider::new(temp_file.path().to_path_buf(), config);
        let context = provider.get_context().await.unwrap();

        assert!(context.content.contains("Test content"));
        assert!(context.content.contains("File:"));
        assert!(context.content.contains("Size:"));
    }

    #[tokio::test]
    async fn test_file_not_found() {
        let config = ContextConfig::default();
        let provider = FileProvider::new(PathBuf::from("/nonexistent"), config);
        let result = provider.get_context().await;

        assert!(matches!(result, Err(ContextError::FileNotFound(_))));
    }

    #[tokio::test]
    async fn test_permission_denied() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::set_permissions(temp_file.path(), Permissions::from_mode(0o000)).unwrap();

        let config = ContextConfig::default();
        let provider = FileProvider::new(temp_file.path().to_path_buf(), config);
        let result = provider.get_context().await;

        assert!(matches!(result, Err(ContextError::PermissionDenied(_))));
    }

    #[tokio::test]
    async fn test_size_limit() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let large_content = "x".repeat(1000);
        writeln!(temp_file, "{}", large_content).unwrap();

        let config = ContextConfig {
            max_size: 100, // Small limit
            include_hidden: false,
            max_depth: None,
        };

        let provider = FileProvider::new(temp_file.path().to_path_buf(), config);
        let result = provider.get_context().await;

        assert!(matches!(result, Err(ContextError::TooLarge(_))));
    }
}
