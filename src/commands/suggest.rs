use colored::Colorize;
use super::{CommandError, CommandInfo, CommandResult};
use super::matcher::find_matches;

/// Format a list of command suggestions into a colored string
pub fn format_suggestions(commands: &[CommandInfo]) -> String {
    if commands.is_empty() {
        return format!("{}", "No matching commands found.".red());
    }

    let mut output = String::new();
    
    if commands.len() == 1 {
        output.push_str("Found the perfect tool for you:\n\n");
    } else {
        output.push_str(&format!("Found {} relevant tools:\n\n", commands.len()));
    }

    for (i, command) in commands.iter().enumerate() {
        output.push_str(&command.format_suggestion());
        if i < commands.len() - 1 {
            output.push_str("\n---\n\n");
        }
    }

    output
}

/// Process a command query and return formatted suggestions
pub async fn process_command_query(query: &str) -> CommandResult<String> {
    let matches = find_matches(query)?;
    
    if matches.is_empty() {
        return Err(CommandError::NoMatch);
    }

    Ok(format_suggestions(&matches))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Category;

    #[test]
    fn test_format_suggestions() {
        let command = CommandInfo {
            name: "test".to_string(),
            description: "A test command".to_string(),
            category: Category::Development,
            examples: vec!["test example".to_string()],
            keywords: vec!["test".to_string()],
        };

        let suggestions = format_suggestions(&[command]);
        assert!(suggestions.contains("test"));
        assert!(suggestions.contains("A test command"));
        assert!(suggestions.contains("Development"));
        assert!(suggestions.contains("test example"));
    }

    #[test]
    fn test_format_empty_suggestions() {
        let suggestions = format_suggestions(&[]);
        assert!(suggestions.contains("No matching commands found"));
    }

    #[tokio::test]
    async fn test_process_command_query() {
        let result = process_command_query("profile execution time").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("hyperfine"));
    }

    #[tokio::test]
    async fn test_process_invalid_query() {
        let result = process_command_query("xyzabc123").await;
        assert!(matches!(result, Err(CommandError::NoMatch)));
    }
}
