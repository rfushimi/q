use lazy_static::lazy_static;
use std::collections::HashMap;
use super::{Category, CommandInfo};

lazy_static! {
    pub static ref COMMAND_DATABASE: HashMap<String, CommandInfo> = {
        let mut m = HashMap::new();

        // Performance Tools
        m.insert(
            "hyperfine".to_string(),
            CommandInfo {
                name: "hyperfine".to_string(),
                description: "A command-line benchmarking tool that measures command execution time with statistical analysis".to_string(),
                category: Category::Performance,
                examples: vec![
                    "hyperfine 'sleep 0.3'".to_string(),
                    "hyperfine --warmup 3 'grep -R TODO ./'".to_string(),
                ],
                keywords: vec![
                    "benchmark".to_string(),
                    "performance".to_string(),
                    "timing".to_string(),
                    "profiling".to_string(),
                ],
            }
        );

        // System Monitoring
        m.insert(
            "htop".to_string(),
            CommandInfo {
                name: "htop".to_string(),
                description: "An interactive process viewer and system monitor".to_string(),
                category: Category::Process,
                examples: vec![
                    "htop".to_string(),
                    "htop -u username".to_string(),
                ],
                keywords: vec![
                    "process".to_string(),
                    "monitor".to_string(),
                    "cpu".to_string(),
                    "memory".to_string(),
                    "system".to_string(),
                ],
            }
        );

        // Disk Usage
        m.insert(
            "ncdu".to_string(),
            CommandInfo {
                name: "ncdu".to_string(),
                description: "NCurses Disk Usage - a disk usage analyzer with an ncurses interface".to_string(),
                category: Category::FileSystem,
                examples: vec![
                    "ncdu /home".to_string(),
                    "ncdu -x /".to_string(),
                ],
                keywords: vec![
                    "disk".to_string(),
                    "storage".to_string(),
                    "space".to_string(),
                    "usage".to_string(),
                    "files".to_string(),
                ],
            }
        );

        // Network Tools
        m.insert(
            "mtr".to_string(),
            CommandInfo {
                name: "mtr".to_string(),
                description: "A network diagnostic tool that combines ping and traceroute".to_string(),
                category: Category::Network,
                examples: vec![
                    "mtr google.com".to_string(),
                    "mtr --report example.com".to_string(),
                ],
                keywords: vec![
                    "network".to_string(),
                    "ping".to_string(),
                    "traceroute".to_string(),
                    "diagnostic".to_string(),
                ],
            }
        );

        // File Search
        m.insert(
            "fd".to_string(),
            CommandInfo {
                name: "fd".to_string(),
                description: "A simple, fast and user-friendly alternative to find".to_string(),
                category: Category::FileSystem,
                examples: vec![
                    "fd pattern".to_string(),
                    "fd -e txt".to_string(),
                ],
                keywords: vec![
                    "find".to_string(),
                    "search".to_string(),
                    "files".to_string(),
                    "locate".to_string(),
                ],
            }
        );

        // Development Tools
        m.insert(
            "ripgrep".to_string(),
            CommandInfo {
                name: "ripgrep".to_string(),
                description: "An extremely fast alternative to grep that respects gitignore rules".to_string(),
                category: Category::Development,
                examples: vec![
                    "rg pattern".to_string(),
                    "rg -t rust 'fn main'".to_string(),
                ],
                keywords: vec![
                    "search".to_string(),
                    "grep".to_string(),
                    "code".to_string(),
                    "find".to_string(),
                ],
            }
        );

        // Process Management
        m.insert(
            "fzf".to_string(),
            CommandInfo {
                name: "fzf".to_string(),
                description: "A command-line fuzzy finder".to_string(),
                category: Category::Process,
                examples: vec![
                    "fzf".to_string(),
                    "vim $(fzf)".to_string(),
                ],
                keywords: vec![
                    "search".to_string(),
                    "filter".to_string(),
                    "fuzzy".to_string(),
                    "find".to_string(),
                ],
            }
        );

        m
    };
}

pub fn get_all_commands() -> Vec<&'static CommandInfo> {
    COMMAND_DATABASE.values().collect()
}

pub fn get_command(name: &str) -> Option<&'static CommandInfo> {
    COMMAND_DATABASE.get(name)
}
