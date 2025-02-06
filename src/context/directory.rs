use async_trait::async_trait;
use std::path::PathBuf;
use walkdir::WalkDir;

use super::{ContextConfig, ContextData, ContextError, ContextProvider, ContextResult, ContextType};
use super::{format_path_for_display, should_include_path, validate_size};

pub struct DirectoryProvider {
    path: PathBuf,
    config: ContextConfig,
}

impl DirectoryProvider {
    pub fn new(path: PathBuf, config: ContextConfig) -> Self {
        Self { path, config }
    }

    fn format_directory_listing(&self) -> ContextResult<String> {
        let mut output = String::new();
        let mut total_size = 0;

        // Add current directory header
        output.push_str(&format!("Directory listing for {}:\n\n", format_path_for_display(&self.path)));

        // Walk the directory
        let walker = WalkDir::new(&self.path)
            .min_depth(1)
            .max_depth(self.config.max_depth.unwrap_or(1))
            .follow_links(false);

        for entry in walker {
            let entry = entry.map_err(|e| ContextError::Other(e.to_string()))?;
            let path = entry.path().to_path_buf();

            if !should_include_path(&path, &self.config) {
                continue;
            }

            // Format the entry
            let relative_path = path.strip_prefix(&self.path)
                .map_err(|_| ContextError::InvalidPath(format_path_for_display(&path)))?;
            
            let entry_str = format!("{}\n", relative_path.display());
            total_size += entry_str.len();

            // Check size before adding
            validate_size(total_size, self.config.max_size, "Directory listing")?;
            output.push_str(&entry_str);
        }

        Ok(output)
    }
}

#[async_trait]
impl ContextProvider for DirectoryProvider {
    fn context_type(&self) -> ContextType {
        ContextType::Directory
    }

    async fn get_context(&self) -> ContextResult<ContextData> {
        let content = self.format_directory_listing()?;
        
        Ok(ContextData {
            context_type: self.context_type(),
            content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_directory_listing() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        // Create some test files and directories
        fs::create_dir(base_path.join("subdir")).unwrap();
        fs::write(base_path.join("file1.txt"), "content").unwrap();
        fs::write(base_path.join("subdir/file2.txt"), "content").unwrap();
        fs::write(base_path.join(".hidden"), "content").unwrap();

        let config = ContextConfig {
            max_size: 1024,
            include_hidden: false,
            max_depth: Some(2),
        };

        let provider = DirectoryProvider::new(base_path.to_path_buf(), config);
        let context = provider.get_context().await.unwrap();

        assert!(context.content.contains("file1.txt"));
        assert!(context.content.contains("subdir/file2.txt"));
        assert!(!context.content.contains(".hidden"));
    }

    #[tokio::test]
    async fn test_size_limit() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        // Create many files to exceed size limit
        for i in 0..100 {
            fs::write(base_path.join(format!("file{}.txt", i)), "content").unwrap();
        }

        let config = ContextConfig {
            max_size: 50, // Very small limit
            include_hidden: false,
            max_depth: Some(1),
        };

        let provider = DirectoryProvider::new(base_path.to_path_buf(), config);
        let result = provider.get_context().await;

        assert!(matches!(result, Err(ContextError::TooLarge(_))));
    }
}
