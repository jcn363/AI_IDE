use crate::error::TestError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a mock Tauri command for testing
#[derive(Debug)]
pub struct MockCommand {
    pub name: String,
    pub payload: serde_json::Value,
    pub result: Result<serde_json::Value, serde_json::Error>,
}

impl MockCommand {
    pub fn new(name: &str, payload: serde_json::Value) -> Self {
        Self {
            name: name.to_string(),
            payload,
            result: Ok(serde_json::Value::Null),
        }
    }

    pub fn with_result(mut self, result: serde_json::Value) -> Self {
        self.result = Ok(result);
        self
    }

    pub fn with_error(mut self, error: &str) -> Self {
        self.result = Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::Other,
            error,
        )));
        self
    }
}

/// Runner for testing Tauri commands
pub struct CommandTestRunner {
    commands: HashMap<String, MockCommand>,
    called_commands: Vec<String>,
}

impl CommandTestRunner {
    pub fn new() -> Self {
        CommandTestRunner {
            commands: HashMap::new(),
            called_commands: Vec::new(),
        }
    }

    /// Registers a mock command for testing
    pub fn register_command(mut self, command: MockCommand) -> Self {
        self.commands.insert(command.name.clone(), command);
        self
    }

    /// Registers multiple commands from a builder
    pub fn register_commands(mut self, commands: Vec<MockCommand>) -> Self {
        for command in commands {
            self.commands.insert(command.name.clone(), command);
        }
        self
    }

    /// Executes a command and tracks the call
    pub async fn execute_command<T: Serialize, R: for<'de> Deserialize<'de>>(
        &mut self,
        name: &str,
        payload: &T,
    ) -> Result<R, TestError> {
        self.called_commands.push(name.to_string());

        let command = self
            .commands
            .get(name)
            .ok_or_else(|| TestError::Tauri(format!("Command '{}' not registered", name)))?;

        // Validate payload matches (simplified for testing)
        let payload_value =
            serde_json::to_value(payload).map_err(|e| TestError::Serialization(e.to_string()))?;

        let expected_payload = &command.payload;
        if *expected_payload != serde_json::Value::Null && expected_payload != &payload_value {
            return Err(TestError::Validation(
                crate::error::ValidationError::invalid_setup(format!(
                    "Payload mismatch for command '{}': expected {:?}, got {:?}",
                    name, expected_payload, payload_value
                )),
            ));
        }

        match &command.result {
            Ok(result) => serde_json::from_value(result.clone())
                .map_err(|e| TestError::Serialization(e.to_string())),
            Err(error) => Err(TestError::Tauri(error.to_string())),
        }
    }

    /// Gets the list of called commands
    pub fn called_commands(&self) -> &[String] {
        &self.called_commands
    }

    /// Asserts that specific commands were called
    pub fn assert_commands_called(&self, expected: &[&str]) -> Result<(), TestError> {
        let called: Vec<&str> = self.called_commands.iter().map(|s| s.as_str()).collect();
        if called != expected {
            return Err(TestError::Validation(
                crate::error::ValidationError::invalid_setup(format!(
                    "Expected commands {:?}, but got {:?}",
                    expected, called
                )),
            ));
        }
        Ok(())
    }

    /// Clears the called commands list
    pub fn clear_calls(&mut self) {
        self.called_commands.clear();
    }
}

impl Default for CommandTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating command test scenarios
pub struct CommandTestBuilder {
    commands: Vec<MockCommand>,
}

impl CommandTestBuilder {
    pub fn new() -> Self {
        CommandTestBuilder {
            commands: Vec::new(),
        }
    }

    pub fn with_command(mut self, command: MockCommand) -> Self {
        self.commands.push(command);
        self
    }

    pub fn success_command<T: Serialize>(
        self,
        name: &str,
        payload: T,
        result: serde_json::Value,
    ) -> Self {
        self.with_command(
            MockCommand::new(name, serde_json::to_value(payload).unwrap()).with_result(result),
        )
    }

    pub fn error_command<T: Serialize>(self, name: &str, payload: T, error: &str) -> Self {
        self.with_command(
            MockCommand::new(name, serde_json::to_value(payload).unwrap()).with_error(error),
        )
    }

    pub fn build_runner(self) -> CommandTestRunner {
        let mut runner = CommandTestRunner::new();
        for command in self.commands {
            runner = runner.register_command(command);
        }
        runner
    }
}

impl Default for CommandTestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Common Tauri command test cases
pub struct CommandTestPresets;

impl CommandTestPresets {
    /// Preset for basic AI analysis commands
    pub fn ai_analysis() -> CommandTestBuilder {
        CommandTestBuilder::new()
            .success_command(
                "analyze_code",
                serde_json::json!({
                    "code": "fn hello() {}",
                    "language": "rust"
                }),
                serde_json::json!({
                    "analysis": {
                        "complexity": 1,
                        "issues": []
                    }
                }),
            )
            .error_command(
                "get_suggestions",
                serde_json::json!({"code": ""}),
                "Invalid code provided",
            )
    }

    /// Preset for filesystem commands
    pub fn filesystem() -> CommandTestBuilder {
        CommandTestBuilder::new()
            .success_command(
                "read_file",
                serde_json::json!({
                    "path": "/test/file.rs"
                }),
                serde_json::json!({
                    "content": "pub fn test() {}"
                }),
            )
            .error_command(
                "write_file",
                serde_json::json!({
                    "path": "/invalid/path"
                }),
                "Permission denied",
            )
    }

    /// Preset for Cargo operations
    pub fn cargo() -> CommandTestBuilder {
        CommandTestBuilder::new()
            .success_command(
                "cargo_build",
                serde_json::json!({
                    "release": false
                }),
                serde_json::json!({
                    "success": true,
                    "output": "Compiling..."
                }),
            )
            .error_command(
                "cargo_publish",
                serde_json::json!({"dry_run": false}),
                "Authentication required",
            )
    }
}

/// Macros for testing Tauri commands
#[macro_export]
macro_rules! setup_command_test {
    () => {
        $crate::command_tests::CommandTestRunner::new()
    };
    ($($setup:expr),*) => {{
        let mut runner = $crate::command_tests::CommandTestRunner::new();
        $(runner = runner.register_command($setup);)*
        runner
    }};
}

#[macro_export]
macro_rules! assert_command_called {
    ($runner:expr, $command:expr) => {
        assert!(
            $runner.called_commands().contains(&$command.to_string()),
            "Expected command '{}' to be called",
            $command
        );
    };
}

#[macro_export]
macro_rules! assert_commands_sequence {
    ($runner:expr, $($command:expr),*) => {
        $runner.assert_commands_called(&[$($command),*])
            .expect_test("Command sequence didn't match");
    };
}
