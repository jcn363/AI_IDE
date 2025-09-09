//! Rust AI IDE Debugger
//!
//! A modular debugger implementation for the Rust AI IDE, supporting GDB and LLDB backends.
//!
//! # Features
//! - Breakpoint management
//! - Watch expressions
//! - Variable inspection
//! - Call stack navigation
//! - Step debugging (over, into, out)
//! - Multi-threaded debugging

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod debugger;

pub use debugger::cache::DebuggerCache;

// Re-export the main debugger types for convenience
pub use debugger::{
    breakpoints::BreakpointManager,
    commands::{
        ContinueCommand, EvaluateExpressionCommand, PauseCommand, StepIntoCommand, StepOutCommand,
        StepOverCommand, StopCommand,
    },
    event_loop::{DebuggerCommand, DebuggerEventLoop},
    execution::DebuggerBackend,
    expressions::ExpressionManager,
    state::StateManager,
    types::{
        BreakpointInfo, DebuggerConfig, DebuggerEvent, DebuggerState, StackFrame, VariableInfo,
    },
    ui::DebuggerUI,
    Debugger,
};

// Re-export commonly used types from dependencies
pub use serde::{Deserialize, Serialize};
pub use debugger::memory_profiling::{
    MemoryProfiler, Allocation, AllocationType, LeakClassification, LeakType,
    FragmentationAnalysis, HeapVisualization, HeapStatistics, MemorySegment, MemorySegmentType,
    HistogramBin, TopConsumer, MemoryProfileEvent,
};
pub use debugger::thread_debugging::{
    ThreadDebugger, ThreadInfo, AsyncTask, AsyncTaskState, ThreadState, DeadlockInfo,
pub use debugger::performance_profiling::{
    PerformanceProfiler, FunctionCall, FunctionStats, CpuSample, BottleneckAnalysis, Bottleneck, BottleneckType,
    ThroughputAssessment, ResourceUtilization, MemoryUsageBreakdown, FlameGraph, FlameNode, PerformanceProfileEvent,
pub use debugger::types::{
    FrameInteraction, FrameInteractionType, VariableTimelineEntry, VariableChangeSource,
};
};
    ThreadDebuggerEvent, ExecutionTimeline, TimelineEvent, TimelineEventType,
};
pub use std::sync::Arc;
pub use tokio::sync::mpsc;
pub use tokio::sync::Mutex;
