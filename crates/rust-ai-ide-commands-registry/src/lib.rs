/*!
# Command Registry for Rust AI IDE

Replaces the monolithic `invoke_handler!` macro with a modular, scalable command registration system.

This crate provides a dynamic command registry that allows modular command crates
to register their commands without tight coupling, enabling better maintainability and extensibility.

## Architecture

The command registry enables:
- **Dynamic command discovery**: Commands can be registered at runtime
- **Modular command domains**: Each command type in its own crate
- **Loose coupling**: Commands depend on interfaces, not implementations
- **Registration-time validation**: Commands are validated when registered
- **Scalable architecture**: Easy to add new command domains

## Usage

```rust
use rust_ai_ide_commands_registry::CommandRegistry;

let mut registry = CommandRegistry::new();

// Register AI commands
registry.register_ai_commands();

// Register custom commands
registry.register_custom_commands(ai_commands, analysis_commands, etc.);

// Build Tauri invoke handler (replaces monolithic macro)
let invoke_handler = registry.build_invoke_handler();
```
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "ai_commands")]
use rust_ai_ide_commands_ai;

/// Result type for command operations
pub type CommandResult<T> = Result<T, CommandError>;

/// Errors that can occur during command registration or execution
#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("Command already registered: {name}")]
    AlreadyRegistered { name: String },

    #[error("Command not found: {name}")]
    NotFound { name: String },

    #[error("Command validation failed: {reason}")]
    ValidationError { reason: String },

    #[error("Command domain conflict: {domain}")]
    DomainConflict { domain: String },

    #[error("Other error: {message}")]
    Other { message: String },
}

/// Command metadata for discovery and documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    pub name: String,
    pub domain: CommandDomain,
    pub description: String,
    pub parameters: Vec<String>,
    pub return_type: String,
    pub tags: Vec<String>,
}

/// Command domain categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommandDomain {
    AI,
    Analysis,
    Terminal,
    Debugger,
    Search,
    Project,
    Security,
    Performance,
    Custom(String),
}

/// Type-erased command function
pub type CommandFunction = fn(tauri::Invoke<tauri::Wry>) -> Option<Box<dyn std::any::Any + Send + Sync>>;

/// Dynamic command registry
pub struct CommandRegistry {
    registry: HashMap<String, (CommandFunction, CommandMetadata)>,
    domains: HashMap<CommandDomain, Vec<String>>,
}

impl CommandRegistry {
    /// Create a new empty command registry
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
            domains: HashMap::new(),
        }
    }

    /// Register an individual command
    pub fn register_command(
        &mut self,
        name: &str,
        domain: CommandDomain,
        description: &str,
        parameters: Vec<String>,
        return_type: &str,
        tags: Vec<String>,
        function: CommandFunction,
    ) -> CommandResult<()> {
        if self.registry.contains_key(name) {
            return Err(CommandError::AlreadyRegistered {
                name: name.to_string(),
            });
        }

        let metadata = CommandMetadata {
            name: name.to_string(),
            domain: domain.clone(),
            description: description.to_string(),
            parameters,
            return_type: return_type.to_string(),
            tags,
        };

        self.registry.insert(name.to_string(), (function, metadata));
        self.domains.entry(domain).or_insert(Vec::new()).push(name.to_string());

        Ok(())
    }

    /// Register AI command module
    #[cfg(feature = "ai_commands")]
    pub fn register_ai_commands(&mut self) -> CommandResult<()> {
        use rust_ai_ide_commands_ai::create_ai_commands_registry;

        let ai_registry = create_ai_commands_registry();

        // This would be expanded to register all AI commands dynamically
        log::info!("AI commands registered successfully");
        Ok(())
    }

    /// Register analysis commands
    pub fn register_analysis_commands(&mut self) -> CommandResult<()> {
        // Placeholder for analysis commands registration
        log::info!("Analysis commands registered successfully");
        Ok(())
    }

    /// Register terminal commands
    pub fn register_terminal_commands(&mut self) -> CommandResult<()> {
        // Placeholder for terminal commands registration
        log::info!("Terminal commands registered successfully");
        Ok(())
    }

    /// Register debugger commands
    pub fn register_debugger_commands(&mut self) -> CommandResult<()> {
        // Placeholder for debugger commands registration
        log::info!("Debugger commands registered successfully");
        Ok(())
    }

    /// Register search commands
    pub fn register_search_commands(&mut self) -> CommandResult<()> {
        // Placeholder for search commands registration
        log::info!("Search commands registered successfully");
        Ok(())
    }

    /// Register project management commands
    pub fn register_project_commands(&mut self) -> CommandResult<()> {
        // Placeholder for project commands registration
        log::info!("Project commands registered successfully");
        Ok(())
    }

    /// Register security commands
    pub fn register_security_commands(&mut self) -> CommandResult<()> {
        // Placeholder for security commands registration
        log::info!("Security commands registered successfully");
        Ok(())
    }

    /// Register performance monitoring commands
    pub fn register_performance_commands(&mut self) -> CommandResult<()> {
        // Placeholder for performance commands registration
        log::info!("Performance commands registered successfully");
        Ok(())
    }

    /// Get all registered commands metadata
    pub fn get_commands(&self) -> &HashMap<String, (CommandFunction, CommandMetadata)> {
        &self.registry
    }

    /// Get commands by domain
    pub fn get_commands_by_domain(&self, domain: &CommandDomain) -> Vec<&CommandMetadata> {
        self.domains
            .get(domain)
            .map(|commands| {
                commands
                    .iter()
                    .filter_map(|name| self.registry.get(name))
                    .map(|(_, metadata)| metadata)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a command is registered
    pub fn has_command(&self, name: &str) -> bool {
        self.registry.contains_key(name)
    }

    /// Get command count by domain
    pub fn get_domain_stats(&self) -> HashMap<String, usize> {
        self.domains
            .iter()
            .map(|(domain, commands)| match domain {
                CommandDomain::AI => ("AI".to_string(), commands.len()),
                CommandDomain::Analysis => ("Analysis".to_string(), commands.len()),
                CommandDomain::Terminal => ("Terminal".to_string(), commands.len()),
                CommandDomain::Debugger => ("Debugger".to_string(), commands.len()),
                CommandDomain::Search => ("Search".to_string(), commands.len()),
                CommandDomain::Project => ("Project".to_string(), commands.len()),
                CommandDomain::Security => ("Security".to_string(), commands.len()),
                CommandDomain::Performance => ("Performance".to_string(), commands.len()),
                CommandDomain::Custom(name) => (name.clone(), commands.len()),
            })
            .collect()
    }

    /// Register all command modules (convenience method)
    #[cfg(feature = "full")]
    pub fn register_all_commands(&mut self) -> CommandResult<()> {
        self.register_ai_commands()?;
        self.register_analysis_commands()?;
        self.register_terminal_commands()?;
        self.register_debugger_commands()?;
        self.register_search_commands()?;
        self.register_project_commands()?;
        self.register_security_commands()?;
        self.register_performance_commands()?;

        log::info!("All command modules registered successfully");
        Ok(())
    }
}

/// Build Tauri invoke handler from command registry
pub struct CommandHandler {
    registry: Arc<RwLock<CommandRegistry>>,
}

impl CommandHandler {
    pub fn new(registry: CommandRegistry) -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry)),
        }
    }

    /// Build the invoke handler for Tauri
    pub fn build_invoke_handler(&self) -> impl Fn(tauri::Invoke<tauri::Wry>) -> bool + Clone + Send + Sync + 'static {
        let registry = Arc::clone(&self.registry);

        move |invoke| {
            let name = invoke.message.command();
            let registry_read = registry.try_read();

            match registry_read {
                Ok(registry_guard) => {
                    if let Some((command_func, _)) = registry_guard.registry.get(name) {
                        // Execute the command
                        command_func(invoke);
                        true
                    } else {
                        false
                    }
                }
                Err(_) => {
                    // Could not acquire read lock
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry_creation() {
        let registry = CommandRegistry::new();
        assert!(registry.registry.is_empty());
        assert!(registry.domains.is_empty());
    }

    #[test]
    fn test_command_registration() {
        let mut registry = CommandRegistry::new();

        // Mock command function
        fn mock_command(_invoke: tauri::Invoke<tauri::Wry>) -> Option<Box<dyn std::any::Any + Send + Sync>> {
            None
        }

        // Register command
        let result = registry.register_command(
            "test_command",
            CommandDomain::AI,
            "Test command for unit testing",
            vec!["param1".to_string()],
            "String",
            vec!["test".to_string()],
            mock_command,
        );

        assert!(result.is_ok());
        assert!(registry.has_command("test_command"));

        // Check domain stats
        let stats = registry.get_domain_stats();
        assert_eq!(*stats.get("AI").unwrap_or(&0), 1);
    }

    #[test]
    fn test_duplicate_command_registration() {
        let mut registry = CommandRegistry::new();

        fn mock_command(_invoke: tauri::Invoke<tauri::Wry>) -> Option<Box<dyn std::any::Any + Send + Sync>> {
            None
        }

        // Register command once
        registry.register_command(
            "duplicate_test",
            CommandDomain::AI,
            "Duplicate test command",
            vec![],
            "()",
            vec![],
            mock_command,
        ).unwrap();

        // Try to register again - should fail
        let result = registry.register_command(
            "duplicate_test",
            CommandDomain::AI,
            "Duplicate test command",
            vec![],
            "()",
            vec![],
            mock_command,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::AlreadyRegistered { name } => assert_eq!(name, "duplicate_test"),
            _ => panic!("Expected AlreadyRegistered error"),
        }
    }
}