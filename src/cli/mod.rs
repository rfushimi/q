pub mod args;

use clap::CommandFactory;
use crate::utils::errors::QError;

impl args::Cli {
    pub fn run(&self) -> Result<(), QError> {
        match &self.command {
            Some(cmd) => cmd.execute(),
            None => {
                // Handle direct prompt if provided
                if let Some(prompt) = &self.prompt {
                    // TODO: Implement LLM query handling in future milestone
                    println!("Query handling will be implemented in future milestone");
                    println!("Received prompt: {}", prompt);
                    Ok(())
                } else {
                    // No command and no prompt - show help
                    Self::command().print_help()?;
                    Ok(())
                }
            }
        }
    }
}
