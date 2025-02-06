use directories::ProjectDirs;
use std::path::PathBuf;
use crate::utils::errors::QError;

pub struct ConfigPaths {
    config_dir: PathBuf,
    config_file: PathBuf,
}

impl ConfigPaths {
    pub fn new() -> Result<Self, QError> {
        // Check for XDG_CONFIG_HOME environment variable first (mainly for testing)
        let config_dir = if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
            eprintln!("Debug: Using XDG_CONFIG_HOME: {}", xdg_config_home);
            let mut path = PathBuf::from(xdg_config_home);
            path.push("q");
            path
        } else {
            eprintln!("Debug: Using ProjectDirs");
            let proj_dirs = ProjectDirs::from("com", "ryohei", "q")
                .ok_or_else(|| QError::Config("Could not determine config directory".to_string()))?;
            proj_dirs.config_dir().to_path_buf()
        };

        eprintln!("Debug: Config dir path: {:?}", config_dir);
        let config_file = config_dir.join("config.toml");
        eprintln!("Debug: Config file path: {:?}", config_file);

        Ok(Self {
            config_dir,
            config_file,
        })
    }

    pub fn ensure_config_dir(&self) -> Result<(), QError> {
        if !self.config_dir.exists() {
            eprintln!("Debug: Creating config directory: {:?}", self.config_dir);
            std::fs::create_dir_all(&self.config_dir)
                .map_err(|e| QError::Io(e))?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&self.config_dir)
                    .map_err(|e| QError::Io(e))?
                    .permissions();
                perms.set_mode(0o700); // User read/write/execute only
                std::fs::set_permissions(&self.config_dir, perms)
                    .map_err(|e| QError::Io(e))?;
            }
        }
        Ok(())
    }

    pub fn config_file(&self) -> &PathBuf {
        &self.config_file
    }

    #[cfg(test)]
    pub fn with_root(root: PathBuf) -> Self {
        let config_dir = root.clone();
        let config_file = root.join("config.toml");
        Self {
            config_dir,
            config_file,
        }
    }
}
