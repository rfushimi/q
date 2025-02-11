use colored::*;

pub fn format_markdown(text: &str) -> String {
    let mut result = String::new();
    let mut in_code_block = false;
    let mut code_block_content = String::new();

    for line in text.lines() {
        if line.starts_with("```") {
            if in_code_block {
                // End of code block
                result.push_str(&code_block_content.cyan().to_string());
                result.push('\n');
                code_block_content.clear();
            }
            in_code_block = !in_code_block;
            continue;
        }

        if in_code_block {
            code_block_content.push_str(line);
            code_block_content.push('\n');
        } else if line.starts_with("**") && line.ends_with("**") {
            // Bold text
            let content = &line[2..line.len()-2];
            result.push_str(&content.bold().to_string());
            result.push('\n');
        } else if line.starts_with("* ") {
            // List item
            result.push_str(&format!("• {}\n", &line[2..]).yellow().to_string());
        } else {
            // Normal text
            result.push_str(line);
            result.push('\n');
        }
    }

    // Handle any remaining code block
    if !code_block_content.is_empty() {
        result.push_str(&code_block_content.cyan().to_string());
    }

    result
}
