//! Advanced thread debugging for async/await code with execution visualization and deadlock detection
//!
//! This module provides advanced debugging capabilities for multi-threaded and async code,
//! including execution visualization, deadlock detection, and async task tracking.

use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;

/// Represents an async task or future being executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncTask {
    /// Unique task ID
    pub id: u32,
    /// Human-readable task name
    pub name: String,
    /// Current execution state
    pub state: AsyncTaskState,
    /// Thread ID where this task is running
    pub thread_id: Option<u32>,
    /// Wakeup timestamp (for tracking scheduling)
    pub created_at: u64,
    /// Current wakeup source
    pub wakeup_source: Option<String>,
    /// Task dependencies (other tasks it depends on)
    pub dependencies: Vec<u32>,
    /// Position in the async call stack
    pub async_call_stack: Vec<AsyncFrame>,
}

/// Possible states of an async task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AsyncTaskState {
    /// Task is idle waiting to be run
    Pending,
    /// Task is currently executing
    Running,
    /// Task is waiting on future completion
    Suspended,
    /// Task has completed successfully
    Completed,
    /// Task failed with an error
    Error(String),
}

/// Frame in the async call stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncFrame {
    /// Future name or type
    pub future_name: String,
    /// Source location (file and line)
    pub location: String,
    /// Current state of this frame
    pub frame_state: String,
}

/// Thread information with async context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadInfo {
    /// Thread ID
    pub id: u32,
    /// Thread name
    pub name: String,
    /// Current native call stack
    pub call_stack: Vec<String>,
    /// Currently executing async task (if any)
    pub current_task: Option<u32>,
    /// CPU time consumed
    pub cpu_time: Duration,
    /// Thread state
    pub state: ThreadState,
    /// Locks currently held by this thread
    pub held_locks: HashSet<u64>,
    /// Locks this thread is waiting on
    pub waiting_locks: HashSet<u64>,
}

/// Possible states of a thread
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThreadState {
    /// Thread is running
    Running,
    /// Thread is blocked waiting for a resource
    Blocked,
    /// Thread is waiting for async I/O
    AsyncWait,
    /// Thread is sleeping
    Sleeping,
    /// Thread has terminated
    Terminated,
}

/// Deadlock detection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadlockInfo {
    /// Thread IDs involved in the deadlock
    pub involved_threads: Vec<u32>,
    /// Lock IDs that are causing the deadlock
    pub contested_locks: Vec<u64>,
    /// Description of the deadlock scenario
    pub description: String,
    /// Timestamp when deadlock was detected
    pub detected_at: u64,
}

/// Execution timeline for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTimeline {
    /// Thread ID
    pub thread_id: u32,
    /// Sequence of execution events
    pub events: Vec<TimelineEvent>,
    /// Total CPU time for this thread
    pub total_cpu_time: Duration,
}

/// Event in the execution timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    /// Event timestamp
    pub timestamp: u64,
    /// Event type
    pub event_type: TimelineEventType,
    /// Description of the event
    pub description: String,
    /// Associated async task ID
    pub task_id: Option<u32>,
}

/// Types of timeline events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimelineEventType {
    /// Task started execution
    TaskStarted,
    /// Task suspended (await point)
    TaskSuspended,
    /// Task resumed execution
    TaskResumed,
    /// Task completed
    TaskCompleted,
    /// Lock acquired
    LockAcquired(u64),
    /// Lock released
    LockReleased(u64),
    /// Lock waiting
    LockWaiting(u64),
    /// Function call
    FunctionCall(String),
    /// Function return
    FunctionReturn(String),
}

/// Advanced thread debugger for async/await code
pub struct ThreadDebugger {
    /// All known threads
    threads: HashMap<u32, ThreadInfo>,
    /// All active async tasks
    tasks: HashMap<u32, AsyncTask>,
    /// Execution timelines for all threads
    timelines: HashMap<u32, ExecutionTimeline>,
    /// Known locks and their current holders
    locks: HashMap<u64, Option<u32>>,
    /// Next task ID to assign
    next_task_id: u32,
    /// Event sender for debugger integration
    event_sender: Option<mpsc::UnboundedSender<ThreadDebuggerEvent>>,
    /// Last update timestamp
    last_update: Instant,
}

/// Events generated by the thread debugger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreadDebuggerEvent {
    /// New thread detected
    ThreadCreated(ThreadInfo),
    /// Thread state changed
    ThreadStateChanged {
        thread_id: u32,
        new_state: ThreadState,
    },
    /// Thread terminated
    ThreadTerminated(u32),
    /// Async task created
    TaskCreated(AsyncTask),
    /// Async task state changed
    TaskStateChanged {
        task_id: u32,
        new_state: AsyncTaskState,
    },
    /// Async task completed
    TaskCompleted(u32),
    /// Deadlock detected
    DeadlockDetected(DeadlockInfo),
    /// Lock contention detected
    LockContention {
        lock_id: u64,
        contending_threads: Vec<u32>,
    },
}

impl ThreadDebugger {
    /// Create a new thread debugger instance
    pub fn new(event_sender: Option<mpsc::UnboundedSender<ThreadDebuggerEvent>>) -> Self {
        Self {
            threads: HashMap::new(),
            tasks: HashMap::new(),
            timelines: HashMap::new(),
            locks: HashMap::new(),
            next_task_id: 1,
            event_sender,
            last_update: Instant::now(),
        }
    }

    /// Send an event to the debugger system
    fn send_event(&self, event: ThreadDebuggerEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(sender) = &self.event_sender {
            sender.send(event)?;
        }
        Ok(())
    }

    /// Track a new thread
    pub fn track_thread(&mut self, thread_id: u32, name: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let thread_info = ThreadInfo {
            id: thread_id,
            name: name.clone(),
            call_stack: Vec::new(),
            current_task: None,
            cpu_time: Duration::new(0, 0),
            state: ThreadState::Running,
            held_locks: HashSet::new(),
            waiting_locks: HashSet::new(),
        };

        let timeline = ExecutionTimeline {
            thread_id,
            events: vec![TimelineEvent {
                timestamp: self.get_timestamp(),
                event_type: TimelineEventType::TaskStarted,
                description: format!("Thread '{}' started", name),
                task_id: None,
            }],
            total_cpu_time: Duration::new(0, 0),
        };

        self.threads.insert(thread_id, thread_info.clone());
        self.timelines.insert(thread_id, timeline);
        self.send_event(ThreadDebuggerEvent::ThreadCreated(thread_info))?;
        Ok(())
    }

    /// Track an async task
    pub fn track_async_task(&mut self, task_name: String, thread_id: Option<u32>) -> Result<u32, Box<dyn std::error::Error + Send + Sync>> {
        let task_id = self.next_task_id;
        self.next_task_id += 1;

        let task = AsyncTask {
            id: task_id,
            name: task_name.clone(),
            state: AsyncTaskState::Pending,
            thread_id,
            created_at: self.get_timestamp(),
            wakeup_source: None,
            dependencies: Vec::new(),
            async_call_stack: Vec::new(),
        };

        self.tasks.insert(task_id, task.clone());
        self.send_event(ThreadDebuggerEvent::TaskCreated(task))?;

        // Update timeline if we have a thread
        if let Some(tid) = thread_id {
            if let Some(timeline) = self.timelines.get_mut(&tid) {
                timeline.events.push(TimelineEvent {
                    timestamp: self.get_timestamp(),
                    event_type: TimelineEventType::TaskStarted,
                    description: format!("Async task '{}' started", task_name),
                    task_id: Some(task_id),
                });
            }
        }

        Ok(task_id)
    }

    /// Update thread state
    pub fn update_thread_state(&mut self, thread_id: u32, new_state: ThreadState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            let old_state = thread.state.clone();
            thread.state = new_state.clone();

            // Update timeline
            if let Some(timeline) = self.timelines.get_mut(&thread_id) {
                timeline.events.push(TimelineEvent {
                    timestamp: self.get_timestamp(),
                    event_type: TimelineEventType::TaskStarted, // Using as generic thread event
                    description: format!("Thread state: {:?} -> {:?}", old_state, new_state),
                    task_id: None,
                });
            }

            self.send_event(ThreadDebuggerEvent::ThreadStateChanged {
                thread_id,
                new_state,
            })?;
        }
        Ok(())
    }

    /// Update async task state
    pub fn update_async_task_state(&mut self, task_id: u32, new_state: AsyncTaskState) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            let old_state = task.state.clone();
            task.state = new_state.clone();

            self.send_event(ThreadDebuggerEvent::TaskStateChanged {
                task_id,
                new_state: new_state.clone(),
            })?;

            // If task completed, send completion event
            if matches!(new_state, AsyncTaskState::Completed | AsyncTaskState::Error(_)) {
                self.send_event(ThreadDebuggerEvent::TaskCompleted(task_id))?;
            }

            // Update timeline if we have a thread
            if let Some(thread_id) = task.thread_id {
                if let Some(timeline) = self.timelines.get_mut(&thread_id) {
                    let event_type = match new_state {
                        AsyncTaskState::Suspended => TimelineEventType::TaskSuspended,
                        AsyncTaskState::Running => TimelineEventType::TaskResumed,
                        AsyncTaskState::Completed => TimelineEventType::TaskCompleted,
                        _ => TimelineEventType::TaskStarted, // Default for other states
                    };

                    timeline.events.push(TimelineEvent {
                        timestamp: self.get_timestamp(),
                        event_type,
                        description: format!("Task '{}' state: {:?} -> {:?}", task.name, old_state, new_state),
                        task_id: Some(task_id),
                    });
                }
            }
        }
        Ok(())
    }

    /// Track lock acquisition
    pub fn acquire_lock(&mut self, thread_id: u32, lock_id: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check for potential deadlock
        if let Some(contending) = self.check_lock_contention(thread_id, lock_id) {
            self.send_event(ThreadDebuggerEvent::LockContention {
                lock_id,
                contending_threads: contending,
            })?;
        }

        // Update thread locks
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            thread.held_locks.insert(lock_id);
            thread.waiting_locks.remove(&lock_id);
        }

        // Update lock ownership
        self.locks.insert(lock_id, Some(thread_id));

        // Update timeline
        if let Some(timeline) = self.timelines.get_mut(&thread_id) {
            timeline.events.push(TimelineEvent {
                timestamp: self.get_timestamp(),
                event_type: TimelineEventType::LockAcquired(lock_id),
                description: format!("Lock {} acquired", lock_id),
                task_id: self.threads.get(&thread_id).and_then(|t| t.current_task),
            });
        }

        Ok(())
    }

    /// Track lock release
    pub fn release_lock(&mut self, thread_id: u32, lock_id: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Update thread locks
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            thread.held_locks.remove(&lock_id);
        }

        // Update lock ownership
        if self.locks.get(&lock_id) == Some(&Some(thread_id)) {
            self.locks.insert(lock_id, None);
        }

        // Update timeline
        if let Some(timeline) = self.timelines.get_mut(&thread_id) {
            timeline.events.push(TimelineEvent {
                timestamp: self.get_timestamp(),
                event_type: TimelineEventType::LockReleased(lock_id),
                description: format!("Lock {} released", lock_id),
                task_id: self.threads.get(&thread_id).and_then(|t| t.current_task),
            });
        }

        // Check if any threads were waiting for this lock
        self.wake_waiting_threads(lock_id);
        Ok(())
    }

    /// Wait for a lock
    pub fn wait_for_lock(&mut self, thread_id: u32, lock_id: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            thread.waiting_locks.insert(lock_id);
            thread.state = ThreadState::Blocked;
        }

        // Update timeline
        if let Some(timeline) = self.timelines.get_mut(&thread_id) {
            timeline.events.push(TimelineEvent {
                timestamp: self.get_timestamp(),
                event_type: TimelineEventType::LockWaiting(lock_id),
                description: format!("Waiting for lock {}", lock_id),
                task_id: self.threads.get(&thread_id).and_then(|t| t.current_task),
            });
        }

        self.update_thread_state(thread_id, ThreadState::Blocked)?;
        Ok(())
    }

    /// Check for lock contention and potential deadlocks
    fn check_lock_contention(&self, requesting_thread: u32, target_lock: u64) -> Option<Vec<u32>> {
        let mut contending = Vec::new();
        let mut checked = HashSet::new();
        let mut stack = vec![requesting_thread];

        // Build wait-for graph and detect cycles
        while let Some(thread_id) = stack.pop() {
            if !checked.insert(thread_id) {
                continue; // Already processed
            }

            if let Some(thread) = self.threads.get(&thread_id) {
                contending.push(thread_id);

                // Add threads waiting on locks held by this thread
                for lock_id in &thread.held_locks {
                    contending.extend(
                        self.threads.values()
                            .filter(|t| t.waiting_locks.contains(lock_id))
                            .map(|t| t.id)
                    );
                }
            }
        }

        if contending.len() > 1 {
            Some(contending)
        } else {
            None
        }
    }

    /// Wake threads waiting for a released lock
    fn wake_waiting_threads(&mut self, lock_id: u64) {
        let waiting_threads: Vec<_> = self.threads.values()
            .filter(|t| t.waiting_locks.contains(&lock_id) && matches!(t.state, ThreadState::Blocked))
            .map(|t| t.id)
            .collect();

        for thread_id in waiting_threads {
            if let Some(thread) = self.threads.get_mut(&thread_id) {
                thread.waiting_locks.remove(&lock_id);
                if thread.waiting_locks.is_empty() {
                    let _ = self.update_thread_state(thread_id, ThreadState::Running);
                }
            }
        }
    }

    /// Get current timestamp
    fn get_timestamp(&self) -> u64 {
        self.last_update.elapsed().as_millis() as u64
    }

    /// Get all threads
    pub fn get_threads(&self) -> Vec<&ThreadInfo> {
        self.threads.values().collect()
    }

    /// Get all async tasks
    pub fn get_async_tasks(&self) -> Vec<&AsyncTask> {
        self.tasks.values().collect()
    }

    /// Get execution timeline for a thread
    pub fn get_timeline(&self, thread_id: u32) -> Option<&ExecutionTimeline> {
        self.timelines.get(&thread_id)
    }

    /// Detect deadlocks by analyzing the wait-for graph
    pub fn detect_deadlocks(&self) -> Vec<DeadlockInfo> {
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        let mut deadlocks = Vec::new();

        // Standard cycle detection in directed graph
        for thread in self.threads.keys() {
            if !visited.contains(thread) {
                if let Some(deadlock) = self.detect_cycle(*thread, &mut visited, &mut recursion_stack) {
                    deadlocks.push(deadlock);
                }
            }
        }

        deadlocks
    }

    /// Helper method for cycle detection
    fn detect_cycle(&self, thread_id: u32, visited: &mut HashSet<u32>, recursion_stack: &mut HashSet<u32>) -> Option<DeadlockInfo> {
        visited.insert(thread_id);
        recursion_stack.insert(thread_id);

        if let Some(thread) = self.threads.get(&thread_id) {
            // For each lock this thread is waiting on, check the holder
            for waiting_lock in &thread.waiting_locks {
                if let Some(Some(holder_id)) = self.locks.get(waiting_lock) {
                    if !visited.contains(holder_id) {
                        if let Some(deadlock) = self.detect_cycle(*holder_id, visited, recursion_stack) {
                            return Some(deadlock);
                        }
                    } else if recursion_stack.contains(holder_id) {
                        // Cycle detected!
                        let involved_threads = recursion_stack.iter().cloned().collect();
                        let contested_locks: Vec<u64> = self.threads.values()
                            .flat_map(|t| t.waiting_locks.iter().cloned())
                            .collect();

                        return Some(DeadlockInfo {
                            involved_threads,
                            contested_locks,
                            description: format!("Deadlock detected involving threads: {:?}", recursion_stack),
                            detected_at: self.get_timestamp(),
                        });
                    }
                }
            }
        }

        recursion_stack.remove(&thread_id);
        None
    }

    /// Get visualization data for async/await execution
    pub fn get_async_visualization_data(&self) -> String {
        let mut output = String::from("Async Task Execution Visualization:\n");

        for task in self.tasks.values() {
            output.push_str(&format!("Task {} ({}): {:?}\n", task.id, task.name, task.state));

            for frame in &task.async_call_stack {
                output.push_str(&format!("  {} - {}\n", frame.future_name, frame.location));
            }
        }

        output.push_str("\nThread States:\n");
        for thread in self.threads.values() {
            output.push_str(&format!("Thread {} ({}): {:?}\n", thread.id, thread.name, thread.state));
        }

        output
    }
}