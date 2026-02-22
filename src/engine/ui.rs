use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

pub trait UserInterface: Send + Sync {
    /// Ask user for confirmation before executing a tool
    /// Returns true if user approves, false otherwise
    fn confirm_execution(&self, tool_name: &str, args: &str) -> bool;
}

pub struct CliUserInterface {
    auto_approve: AtomicBool,
}

impl CliUserInterface {
    pub fn new() -> Self {
        Self {
            auto_approve: AtomicBool::new(false),
        }
    }
}

impl UserInterface for CliUserInterface {
    fn confirm_execution(&self, tool_name: &str, args: &str) -> bool {
        // If auto-approve is enabled, skip confirmation
        if self.auto_approve.load(Ordering::Relaxed) {
            info!("Auto-approving (session): {} {}", tool_name, args);
            return true;
        }

        // We use stderr for prompting to avoid polluting stdout if the user is piping output
        // (though in zene CLI mode, the final result goes to stdout)
        eprintln!("\n[CONFIRMATION REQUIRED]");
        eprintln!("The agent wants to execute the following tool:");
        eprintln!("Tool: {}", tool_name);
        eprintln!("Args: {}", args);
        eprint!("Do you approve this execution? (y/N/a [approve all]): ");

        // Flush stderr to ensure prompt is visible
        let _ = io::stderr().flush();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let input = input.trim().to_lowercase();
                if input == "y" || input == "yes" {
                    true
                } else if input == "a" || input == "all" {
                    self.auto_approve.store(true, Ordering::Relaxed);
                    eprintln!("Auto-approve enabled for this session.");
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }
}

pub struct AutoUserInterface;

impl UserInterface for AutoUserInterface {
    fn confirm_execution(&self, tool_name: &str, args: &str) -> bool {
        info!("Auto-approving tool execution: {} {}", tool_name, args);
        true
    }
}
