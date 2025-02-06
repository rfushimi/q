use regex::Regex;
use super::{CommandInfo, CommandResult};
use super::database::get_all_commands;
use super::Category;

/// Score for a command match
#[derive(Debug)]
struct MatchScore {
    command: &'static CommandInfo,
    score: u32,
}

/// Find matching commands for a given query
pub fn find_matches(query: &str) -> CommandResult<Vec<CommandInfo>> {
    let query = query.to_lowercase();
    let mut scores: Vec<MatchScore> = Vec::new();

    // Get all commands
    let commands = get_all_commands();

    // Score each command
    for command in commands {
        let score = calculate_match_score(command, &query);
        if score > 0 {
            scores.push(MatchScore { command, score });
        }
    }

    // Sort by score in descending order
    scores.sort_by(|a, b| b.score.cmp(&a.score));

    // Take top 3 matches
    let matches: Vec<CommandInfo> = scores
        .into_iter()
        .take(3)
        .map(|ms| ms.command.clone())
        .collect();

    Ok(matches)
}

/// Calculate how well a command matches a query
fn calculate_match_score(command: &CommandInfo, query: &str) -> u32 {
    let mut score = 0;

    // Direct name match
    if command.name.to_lowercase().contains(query) {
        score += 100;
    }

    // Category match
    if command.category.to_string().to_lowercase().contains(query) {
        score += 50;
    }

    // Keyword matches
    for keyword in &command.keywords {
        if query.contains(&keyword.to_lowercase()) {
            score += 30;
        }
    }

    // Description match
    if command.description.to_lowercase().contains(query) {
        score += 20;
    }

    // Pattern matching for common queries
    let patterns = [
        (r"(?i)profile|benchmark|time", Category::Performance),
        (r"(?i)monitor|process|cpu|memory", Category::Process),
        (r"(?i)disk|storage|space|file", Category::FileSystem),
        (r"(?i)network|ping|connection", Category::Network),
        (r"(?i)develop|code|program", Category::Development),
    ];

    for (pattern, category) in patterns.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(query) && command.category == *category {
                score += 40;
            }
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::database::COMMAND_DATABASE;

    #[test]
    fn test_find_matches_performance() {
        let matches = find_matches("tool to profile execution time").unwrap();
        assert!(!matches.is_empty());
        
        // hyperfine should be the top match
        assert_eq!(matches[0].name, "hyperfine");
    }

    #[test]
    fn test_find_matches_disk() {
        let matches = find_matches("analyze disk usage").unwrap();
        assert!(!matches.is_empty());
        
        // ncdu should be in the matches
        assert!(matches.iter().any(|m| m.name == "ncdu"));
    }

    #[test]
    fn test_find_matches_no_results() {
        let matches = find_matches("xyzabc123").unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_match_scoring() {
        // Get hyperfine command info
        let command = COMMAND_DATABASE.get("hyperfine").unwrap();
        
        // Test exact name match
        let score1 = calculate_match_score(command, "hyperfine");
        
        // Test category match
        let score2 = calculate_match_score(command, "performance tool");
        
        // Test keyword match
        let score3 = calculate_match_score(command, "benchmark");
        
        assert!(score1 > score2); // Direct name match should score higher
        assert!(score2 > score3); // Category match should score higher than keyword
    }
}
