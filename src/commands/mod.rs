pub mod database;
pub mod matcher;
pub mod suggest;

use colored::Colorize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Pattern matching error: {0}")]
    Pattern(String),

    #[error("No matching commands found")]
    NoMatch,

    #[error("Other error: {0}")]
    Other(String),
}

pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Clone, PartialEq)]
pub enum Category {
    System,
    Network,
    FileSystem,
    Process,
    Performance,
    Development,
    Other,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::System => write!(f, "System"),
            Category::Network => write!(f, "Network"),
            Category::FileSystem => write!(f, "File System"),
            Category::Process => write!(f, "Process"),
            Category::Performance => write!(f, "Performance"),
            Category::Development => write!(f, "Development"),
            Category::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub category: Category,
    pub examples: Vec<String>,
    pub keywords: Vec<String>,
}

impl CommandInfo {
    pub fn format_suggestion(&self) -> String {
        let mut output = String::new();

        // Tool name in green
        output.push_str(&format!("{}\n", self.name.green().bold()));
        
        // Category in blue
        output.push_str(&format!("Category: {}\n", self.category.to_string().blue()));
        
        // Description
        output.push_str(&format!("{}\n", self.description));
        
        // Examples in yellow
        if !self.examples.is_empty() {
            output.push_str("\nExamples:\n");
            for example in &self.examples {
                output.push_str(&format!("  {}\n", example.yellow()));
            }
        }

        output
    }
}

/// Function to suggest commands based on a query
pub async fn suggest_command(query: &str) -> CommandResult<Vec<CommandInfo>> {
    let matches = matcher::find_matches(query)?;
    
    if matches.is_empty() {
        return Err(CommandError::NoMatch);
    }

    Ok(matches)
}
