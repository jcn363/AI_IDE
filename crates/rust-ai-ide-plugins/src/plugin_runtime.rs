use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::sync::Arc;

use rust_ai_ide_common::{IDEError, IDEErrorKind};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::spawn_blocking;
use tokio::time::{timeout, Duration};
use wasmtime::{Engine, Instance, Memory, Module, Store};

/// Plugin runtime for secure execution of IDE extensions
pub struct PluginRuntime {
    pub(crate) engine:             Engine,
    pub(crate) loaded_plugins:     Arc<RwLock<HashMap<String, PluginInstance>>>,
    pub(crate) plugin_permissions: Arc<RwLock<HashMap<String, PluginPermissions>>>,
    pub(crate) execution_contexts: Arc<Mutex<HashMap<String, ExecutionContext>>>,
    pub(crate) plugin_scheduler:   Arc<PluginScheduler>,
    pub(crate) plugin_monitor:     Arc<PluginMonitor>,
}

impl PluginRuntime {
    pub fn new() -> Self {
        let mut config = wasmtime::Config::new();
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        config.memory_init_cow(false);

        let engine = Engine::new(&config)
            .ok()
            .expect("Failed to create WebAssembly engine");

        Self {
            engine,
            loaded_plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_permissions: Arc::new(RwLock::new(HashMap::new())),
            execution_contexts: Arc::new(Mutex::new(HashMap::new())),
            plugin_scheduler: Arc::new(PluginScheduler::new()),
            plugin_monitor: Arc::new(PluginMonitor::new()),
        }
    }

    pub async fn load_plugin(&self, plugin_path: &str, permissions: PluginPermissions) -> Result<String, IDEError> {
        let plugin_id = uuid::Uuid::new_v4().to_string();

        // Validate plugin path
        self.validate_plugin_path(plugin_path).await?;

        // Load WebAssembly module
        let module = spawn_blocking(move || {
            std::fs::read(plugin_path)
                .map_err(|e| IDEError::new(IDEErrorKind::FileOperation, "Failed to read plugin file").with_source(e))
                .and_then(|bytes| {
                    //@ Validate WASM module
                    wasmtime::Module::validate(&Engine::default(), &bytes).map_err(|e| {
                        IDEError::new(IDEErrorKind::ValidationError, "Invalid WebAssembly module").with_source(e)
                    })?;
                    Ok(bytes)
                })
        })
        .await??;

        let wasm_module = Module::new(&self.engine, &module).map_err(|e| {
            IDEError::new(
                IDEErrorKind::ValidationError,
                "Failed to create WASM module",
            )
            .with_source(e)
        })?;

        // Create plugin instance
        let plugin_instance = PluginInstance {
            id:                plugin_id.clone(),
            module:            wasm_module,
            permissions:       permissions.clone(),
            memory_limit:      permissions.memory_limit,
            execution_timeout: permissions.execution_timeout,
            loaded_at:         std::time::SystemTime::now(),
        };

        // Store plugin
        {
            let mut plugins = self.loaded_plugins.write().await;
            plugins.insert(plugin_id.clone(), plugin_instance);
        }

        // Store permissions
        {
            let mut plugin_perms = self.plugin_permissions.write().await;
            plugin_perms.insert(plugin_id.clone(), permissions);
        }

        // Initialize execution context
        let context = ExecutionContext {
            plugin_id:       plugin_id.clone(),
            memory:          0,
            execution_count: 0,
            last_execution:  std::time::SystemTime::now(),
        };

        {
            let mut contexts = self.execution_contexts.lock().await;
            contexts.insert(plugin_id.clone(), context);
        }

        self.plugin_monitor.record_plugin_load(&plugin_id).await;

        Ok(plugin_id)
    }

    pub async fn execute_plugin(&self, plugin_id: &str, input: &[u8]) -> Result<Vec<u8>, IDEError> {
        // Check if plugin is loaded
        let plugin_instance = {
            let plugins = self.loaded_plugins.read().await;
            plugins.get(plugin_id).cloned()
        }
        .ok_or_else(|| {
            IDEError::new(
                IDEErrorKind::ResourceNotFound,
                format!("Plugin {} not loaded", plugin_id),
            )
        })?;

        // Check permissions
        self.check_permissions(&plugin_instance, "execute").await?;

        // Get execution context
        let mut context = {
            let mut contexts = self.execution_contexts.lock().await;
            contexts.get_mut(plugin_id).cloned().unwrap()
        };

        // Check memory limit
        if context.memory + input.len() > plugin_instance.memory_limit {
            return Err(IDEError::new(
                IDEErrorKind::ResourceExhausted,
                format!(
                    "Plugin memory limit exceeded: {}/{}",
                    context.memory, plugin_instance.memory_limit
                ),
            ));
        }

        // Schedule execution
        let execution_token = self
            .plugin_scheduler
            .schedule_execution(plugin_id.to_string(), ExecutionPriority::Normal)
            .await?;

        // Execute with timeout
        let timeout_duration = plugin_instance.execution_timeout;
        let result = timeout(
            timeout_duration,
            self.execute_plugin_internal(plugin_instance, input, &mut context),
        )
        .await
        .map_err(|_| IDEError::new(IDEErrorKind::Timeout, "Plugin execution timeout"))??;

        // Update context
        context.memory = context.memory.max(result.len());
        context.execution_count += 1;
        context.last_execution = std::time::SystemTime::now();

        {
            let mut contexts = self.execution_contexts.lock().await;
            contexts.insert(plugin_id.to_string(), context);
        }

        // Complete execution
        self.plugin_scheduler
            .complete_execution(execution_token)
            .await?;

        self.plugin_monitor
            .record_plugin_execution(plugin_id, result.len())
            .await;

        Ok(result)
    }

    async fn execute_plugin_internal(
        &self,
        mut plugin_instance: PluginInstance,
        input: &[u8],
        context: &mut ExecutionContext,
    ) -> Result<Vec<u8>, IDEError> {
        // Create WASM store
        let mut store = Store::new(&self.engine, WasmtimeRuntimeData {
            input:  input.to_vec(),
            output: Vec::new(),
            memory: Memory::new(&mut store, wasmtime::MemoryType::new(1, Some(2)))
                .map_err(|e| IDEError::new(IDEErrorKind::MemoryError, "").with_source(e))?,
        });

        // Instantiate module
        let instance = Instance::new(&mut store, &plugin_instance.module, &[])
            .map_err(|e| IDEError::new(IDEErrorKind::ExecutionError, "Failed to instantiate plugin").with_source(e))?;

        // Call main function
        let main_fn = instance
            .get_typed_func::<(u64, u64), u64>(&mut store, "_plugin_main")
            .map_err(|e| IDEError::new(IDEErrorKind::ExecutionError, "Plugin missing main function").with_source(e))?;

        let (input_ptr, input_len) = self.allocate_input(&store, input)?;
        let result_ptr = main_fn
            .call(&mut store, (input_ptr, input_len as u64))
            .map_err(|e| IDEError::new(IDEErrorKind::ExecutionError, "Plugin execution failed").with_source(e))?;

        // Extract output
        let output = self.extract_output(&store, result_ptr)?;

        Ok(output)
    }

    fn allocate_input(&self, _store: &Store<WasmtimeRuntimeData>, input: &[u8]) -> Result<(u64, u64), IDEError> {
        // Placeholder for WASM memory allocation
        // In a real implementation, this would allocate WASM memory and copy input
        Ok((0, input.len() as u64))
    }

    fn extract_output(&self, _store: &Store<WasmtimeRuntimeData>, result_ptr: u64) -> Result<Vec<u8>, IDEError> {
        // Placeholder for extracting output from WASM memory
        Ok(vec![])
    }

    pub async fn unload_plugin(&self, plugin_id: &str) -> Result<(), IDEError> {
        // Remove from loaded plugins
        {
            let mut plugins = self.loaded_plugins.write().await;
            plugins.remove(plugin_id);
        }

        // Remove permissions
        {
            let mut perms = self.plugin_permissions.write().await;
            perms.remove(plugin_id);
        }

        // Remove execution context
        {
            let mut contexts = self.execution_contexts.lock().await;
            contexts.remove(plugin_id);
        }

        self.plugin_monitor.record_plugin_unload(plugin_id).await;

        Ok(())
    }

    async fn validate_plugin_path(&self, path: &str) -> Result<(), IDEError> {
        // Security validation for plugin paths
        if path.contains("..") || path.contains("\\") {
            return Err(IDEError::new(
                IDEErrorKind::SecurityViolation,
                "Plugin path contains invalid characters",
            ));
        }

        if !std::path::Path::new(path).exists() {
            return Err(IDEError::new(
                IDEErrorKind::FileOperation,
                format!("Plugin file does not exist: {}", path),
            ));
        }

        Ok(())
    }

    async fn check_permissions(&self, plugin: &PluginInstance, operation: &str) -> Result<(), IDEError> {
        let perms = self.plugin_permissions.read().await;
        let permissions = perms.get(&plugin.id).ok_or_else(|| {
            IDEError::new(
                IDEErrorKind::SecurityViolation,
                "No permissions found for plugin",
            )
        })?;

        match operation {
            "execute" =>
                if !permissions.can_execute {
                    return Err(IDEError::new(
                        IDEErrorKind::SecurityViolation,
                        "Plugin does not have execution permission",
                    ));
                },
            "filesystem" =>
                if !permissions.can_access_filesystem {
                    return Err(IDEError::new(
                        IDEErrorKind::SecurityViolation,
                        "Plugin does not have filesystem access permission",
                    ));
                },
            "network" =>
                if !permissions.can_make_network_requests {
                    return Err(IDEError::new(
                        IDEErrorKind::SecurityViolation,
                        "Plugin does not have network access permission",
                    ));
                },
            _ => {}
        }

        Ok(())
    }

    pub async fn get_plugin_status(&self) -> HashMap<String, PluginStatus> {
        let plugins = self.loaded_plugins.read().await;
        let contexts = self.execution_contexts.lock().await;

        plugins
            .keys()
            .map(|id| {
                let status = if let Some(context) = contexts.get(id) {
                    PluginStatus {
                        id:             id.clone(),
                        loaded:         true,
                        executing:      false, // Could be improved to track current execution
                        memory_usage:   context.memory,
                        last_execution: context.last_execution,
                    }
                } else {
                    PluginStatus {
                        id:             id.clone(),
                        loaded:         true,
                        executing:      false,
                        memory_usage:   0,
                        last_execution: std::time::SystemTime::UNIX_EPOCH,
                    }
                };
                (id.clone(), status)
            })
            .collect()
    }

    pub async fn get_plugin_metrics(&self) -> Vec<PluginMetrics> {
        self.plugin_monitor.get_metrics().await
    }
}

/// Plugin instance representing a loaded WebAssembly module
#[derive(Clone)]
pub struct PluginInstance {
    pub id:                String,
    pub module:            Module,
    pub permissions:       PluginPermissions,
    pub memory_limit:      usize,
    pub execution_timeout: Duration,
    pub loaded_at:         std::time::SystemTime,
}

/// Plugin permissions configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginPermissions {
    pub can_execute:               bool,
    pub can_access_filesystem:     bool,
    pub can_make_network_requests: bool,
    pub can_interact_with_ui:      bool,
    pub memory_limit:              usize,
    pub execution_timeout:         Duration,
    pub allowed_domains:           Vec<String>,
    pub allowed_file_patterns:     Vec<String>,
}

/// Execution context tracking plugin state
#[derive(Clone, Debug)]
pub struct ExecutionContext {
    pub plugin_id:       String,
    pub memory:          usize,
    pub execution_count: u64,
    pub last_execution:  std::time::SystemTime,
}

/// Plugin scheduler for managing plugin execution
pub struct PluginScheduler {
    pub(crate) execution_queue:   Arc<Mutex<Vec<ExecutionRequest>>>,
    pub(crate) active_executions: Arc<RwLock<HashSet<String>>>,
}

impl PluginScheduler {
    pub fn new() -> Self {
        Self {
            execution_queue:   Arc::new(Mutex::new(Vec::new())),
            active_executions: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn schedule_execution(&self, plugin_id: String, priority: ExecutionPriority) -> Result<String, IDEError> {
        let execution_token = uuid::Uuid::new_v4().to_string();

        let request = ExecutionRequest {
            token: execution_token.clone(),
            plugin_id,
            priority,
            scheduled_at: std::time::SystemTime::now(),
        };

        {
            let mut queue = self.execution_queue.lock().await;
            queue.push(request);
            // Sort by priority (simple implementation)
            queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        }

        Ok(execution_token)
    }

    pub async fn complete_execution(&self, token: String) -> Result<(), IDEError> {
        let mut active = self.active_executions.write().await;
        active.remove(&token);

        let mut queue = self.execution_queue.lock().await;
        queue.retain(|req| req.token != token);

        Ok(())
    }

    pub async fn is_execution_active(&self, token: &str) -> bool {
        let active = self.active_executions.read().await;
        active.contains(token)
    }
}

/// Plugin monitor for tracking plugin metrics and health
pub struct PluginMonitor {
    pub(crate) load_events:      Arc<Mutex<Vec<PluginLoadEvent>>>,
    pub(crate) execution_events: Arc<Mutex<Vec<PluginExecutionEvent>>>,
}

impl PluginMonitor {
    pub fn new() -> Self {
        Self {
            load_events:      Arc::new(Mutex::new(Vec::new())),
            execution_events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn record_plugin_load(&self, plugin_id: &str) {
        let event = PluginLoadEvent {
            plugin_id:  plugin_id.to_string(),
            timestamp:  std::time::SystemTime::now(),
            event_type: LoadEventType::Loaded,
        };

        let mut events = self.load_events.lock().await;
        events.push(event);
    }

    pub async fn record_plugin_unload(&self, plugin_id: &str) {
        let event = PluginLoadEvent {
            plugin_id:  plugin_id.to_string(),
            timestamp:  std::time::SystemTime::now(),
            event_type: LoadEventType::Unloaded,
        };

        let mut events = self.load_events.lock().await;
        events.push(event);
    }

    pub async fn record_plugin_execution(&self, plugin_id: &str, output_size: usize) {
        let event = PluginExecutionEvent {
            plugin_id: plugin_id.to_string(),
            timestamp: std::time::SystemTime::now(),
            execution_time: Duration::from_millis(1), // Placeholder
            output_size,
            success: true,
        };

        let mut events = self.execution_events.lock().await;
        events.push(event);
    }

    pub async fn get_metrics(&self) -> Vec<PluginMetrics> {
        let load_events = self.load_events.lock().await;
        let execution_events = self.execution_events.lock().await;

        let plugin_ids: HashSet<String> = load_events.iter().map(|e| e.plugin_id.clone()).collect();

        plugin_ids
            .iter()
            .map(|plugin_id| {
                let loads = load_events
                    .iter()
                    .filter(|e| &e.plugin_id == plugin_id)
                    .count();
                let executions = execution_events
                    .iter()
                    .filter(|e| &e.plugin_id == plugin_id);

                let total_executions = executions.clone().count();
                let successful_executions = executions.filter(|e| e.success).count();
                let avg_execution_time = if total_executions > 0 {
                    executions
                        .map(|e| e.execution_time.as_millis())
                        .sum::<u128>()
                        / total_executions as u128
                } else {
                    0
                };

                PluginMetrics {
                    plugin_id: plugin_id.clone(),
                    load_count: loads,
                    total_executions,
                    successful_executions,
                    avg_execution_time: Duration::from_millis(avg_execution_time as u64),
                    error_rate: if total_executions > 0 {
                        (total_executions - successful_executions) as f64 / total_executions as f64
                    } else {
                        0.0
                    },
                }
            })
            .collect()
    }
}

// Data structures

#[derive(Clone)]
pub struct WasmtimeRuntimeData {
    pub input:  Vec<u8>,
    pub output: Vec<u8>,
    pub memory: Memory,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginStatus {
    pub id:             String,
    pub loaded:         bool,
    pub executing:      bool,
    pub memory_usage:   usize,
    pub last_execution: std::time::SystemTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginMetrics {
    pub plugin_id:             String,
    pub load_count:            usize,
    pub total_executions:      usize,
    pub successful_executions: usize,
    pub avg_execution_time:    Duration,
    pub error_rate:            f64,
}

#[derive(Clone, Debug)]
pub struct PluginLoadEvent {
    pub plugin_id:  String,
    pub timestamp:  std::time::SystemTime,
    pub event_type: LoadEventType,
}

#[derive(Clone, Debug)]
pub enum LoadEventType {
    Loaded,
    Unloaded,
}

#[derive(Clone, Debug)]
pub struct PluginExecutionEvent {
    pub plugin_id:      String,
    pub timestamp:      std::time::SystemTime,
    pub execution_time: Duration,
    pub output_size:    usize,
    pub success:        bool,
}

#[derive(Clone, Debug)]
pub struct ExecutionRequest {
    pub token:        String,
    pub plugin_id:    String,
    pub priority:     ExecutionPriority,
    pub scheduled_at: std::time::SystemTime,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum ExecutionPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Eq for ExecutionPriority {}

impl Ord for ExecutionPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (ExecutionPriority::Low, ExecutionPriority::Low) => std::cmp::Ordering::Equal,
            (ExecutionPriority::Low, _) => std::cmp::Ordering::Less,
            (ExecutionPriority::Normal, ExecutionPriority::Low) => std::cmp::Ordering::Greater,
            (ExecutionPriority::Normal, ExecutionPriority::Normal) => std::cmp::Ordering::Equal,
            (ExecutionPriority::Normal, _) => std::cmp::Ordering::Less,
            (ExecutionPriority::High, ExecutionPriority::Critical) => std::cmp::Ordering::Less,
            (ExecutionPriority::High, _) => std::cmp::Ordering::Greater,
            (ExecutionPriority::Critical, _) => std::cmp::Ordering::Greater,
        }
    }
}
