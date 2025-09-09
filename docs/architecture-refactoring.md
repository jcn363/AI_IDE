# Refactoring System Architecture

This document provides detailed architectural insights into the Rust AI IDE's Advanced Refactoring System, focusing on the layered approach, component interactions, and design patterns.

## System Architecture Overview

The refactoring system follows a layered architecture that ensures separation of concerns, maintainability, and extensibility.

```text
┌─────────────────────────────────────────────────────────────┐
│                      Frontend Layer                         │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ • RefactoringPanel: UI orchestration                    │ │
│  │ • Wizard Components: Specialized operation UIs          │ │
│  │ • useRefactoring Hook: React state management           │ │
│  │ • RefactoringService: Backend API client                │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │ HTTP/WebSocket
┌─────────────────────────────────────────────────────────────┐
│                   Command Bridge Layer                      │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ • Protocol Translation: JSON ↔ Rust types               │ │
│  │ • Progress Reporting: Real-time operation status        │ │
│  │ • Error Mapping: Platform-agnostic error handling       │ │
│  │ • Command Routing: Dynamic command dispatch             │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │ RPC/Streams
┌─────────────────────────────────────────────────────────────┐
│                   Core Refactoring Engine                   │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ • Analysis Module: Context understanding, impact        │ │
│  │ • Operations Module: Concrete refactoring logic         │ │
│  │ • Batch Module: Multi-file orchestration                │ │
│  │ • Test Generation: Automated quality assurance          │ │
│  │ • Utils Module: Shared utilities & helpers              │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                │ LSP/Text/Code
┌─────────────────────────────────────────────────────────────┐
│               External Integration Layer                   │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ • LSP Client: Symbol resolution & navigation           │ │
│  │ • AST Processing: Syntax tree manipulation             │ │
│  │ • AI Service: Intelligent suggestions & analysis        │ │
│  │ • Code Generation: Template-based transformations       │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Component Architecture

### Frontend Layer

#### RefactoringPanel (`web/src/features/ai/components/RefactoringPanel.tsx`)

The main UI orchestrator that manages the refactoring workflow:

```typescript
interface RefactoringPanelProps {
    selectedCode: string;
    filePath: string;
    position: Position;
    availableOperations: RefactoringType[];
    onOperationSelect: (operation: RefactoringOperation) => void;
}

class RefactoringPanel extends React.Component<RefactoringPanelProps> {
    // State management for operation selection
    // Progress tracking for long-running operations
    // Error display and recovery
    // Preview integration
}
```

**Key Responsibilities:**

- Operation discovery and presentation
- User interaction orchestration
- Progress visualization
- Error handling and recovery
- Preview pane integration

#### Wizard Components

Specialized UIs for complex operations:

- **AsyncAwaitWizard**: Function selection, dependency analysis, error handling configuration
- **ExtractInterfaceWizard**: Method selection, interface design, implementation updates
- **BatchRefactoringWizard**: Operation queue management, conflict resolution, dependency ordering
- **PatternConversionWizard**: Pattern detection, conversion options, validation

#### useRefactoring Hook (`web/src/features/ai/hooks/useRefactoring.ts`)

```typescript
interface UseRefactoringOptions {
    autoSave?: boolean;
    previewEnabled?: boolean;
    onProgress?: ProgressCallback;
    onError?: ErrorCallback;
}

function useRefactoring(options: UseRefactoringOptions) {
    const [context, setContext] = useState<RefactoringContext>();
    const [availableOps, setAvailableOps] = useState<RefactoringType[]>();
    const [history, setHistory] = useState<RefactoringHistory[]>([]);

    // State management for:
    // - Current refactoring context
    // - Available operations
    // - Operation history and undo/redo
    // - Progress and error states
}
```

#### RefactoringService (`web/src/features/ai/services/RefactoringService.ts`)

Backend API client with intelligent caching and error recovery:

```typescript
class RefactoringService {
    private cache: Map<string, CacheEntry>;
    private retryQueue: RetryQueue;

    async analyzeContext(context: CodeContext): Promise<AnalysisResult> {
        // Intelligent caching based on content hash
        // Automatic retry with exponential backoff
        // Progress event streaming
    }

    async executeOperation(operation: RefactoringOperation): Promise<Result> {
        // Pre-flight validation
        // Operation execution with progress tracking
        // Automatic rollback on failure
    }
}
```

### Command Bridge Layer

#### Command Routing (`src-tauri/src/commands/refactoring_commands.rs`)

The Tauri command layer translates between frontend TypeScript and backend Rust:

```rust
#[tauri::command]
pub async fn execute_refactoring(
    operation: Json<RefactoringRequest>,
    state: tauri::State<AppState>
) -> Result<Json<RefactoringResponse>, ApiError> {
    // Input validation and deserialization
    // Permission checking
    // Operation dispatch to backend
    // Progress event emission
    // Result serialization
}
```

**Key Functions:**

- `analyze_refactoring_context`: Context analysis for available operations
- `get_available_refactorings`: Return applicable refactoring types
- `execute_refactoring`: Execute single refactoring operation
- `analyze_refactoring_impact`: Impact assessment and risk analysis
- `identify_refactoring_target`: Symbol and scope identification
- `batch_refactoring`: Multi-operation batch processing
- `generate_refactoring_tests`: Automated test generation

#### Progress Reporting

Real-time progress updates via Tauri events:

```rust
pub struct ProgressEmitter {
    window: tauri::Window,
    operation_id: String,
}

impl ProgressEmitter {
    pub async fn emit_progress(&self, phase: ProgressPhase, percentage: f32) {
        self.window.emit("refactoring:progress", ProgressEvent {
            operation_id: self.operation_id.clone(),
            phase: phase.to_string(),
            percentage,
            message: phase.description(),
        }).await?;
    }
}
```

### Core Refactoring Engine

#### Analysis Module (`crates/rust-ai-ide-ai/src/refactoring/analysis.rs`)

Context understanding and impact assessment:

```rust
pub struct RefactoringAnalyzer {
    lsp_client: Arc<LspClient>,
    ai_service: Arc<AiService>,
    symbol_cache: Arc<RwLock<SymbolCache>>,
}

impl RefactoringAnalyzer {
    pub async fn analyze_context(
        &self,
        context: RefactoringContext
    ) -> Result<AnalysisResult> {
        // Symbol resolution
        // Dependency analysis
        // Impact assessment
        // Risk scoring
        // AI-powered suggestions
    }
}
```

**Components:**

- **SymbolAnalyzer**: LSP-based symbol resolution and usage detection
- **ImpactAnalyzer**: Comprehensive dependency tracking and change assessment
- **RefactoringValidator**: Pre-flight validation and safety checks
- **PatternDetector**: AI-powered code pattern recognition
- **ConflictDetector**: Conflict detection for batch operations

#### Operations Module (`crates/rust-ai-ide-ai/src/refactoring/operations.rs`)

Concrete refactoring implementations:

```rust
pub trait RefactoringOperation {
    async fn analyze(&self, context: RefactoringContext) -> Result<RefactoringResult>;
    fn can_apply(&self, context: &RefactoringContext) -> bool;
    async fn apply(&self, context: RefactoringContext) -> Result<CodeChanges>;
    fn priority(&self) -> OperationPriority;
}
```

**Available Operations:**

- **RenameOperation**: File, class, function, variable renaming
- **ExtractVariableOperation**: Expression to variable extraction
- **ExtractMethodOperation**: Statement block to method extraction
- **ExtractInterfaceOperation**: Common interface extraction
- **MoveOperation**: File, class, method moving between namespaces
- **InlineOperation**: Variable/method inlining
- **AsyncAwaitConversion**: Synchronous to asynchronous conversion
- **PatternConversionOperation**: Design pattern transformations

#### Batch Module (`crates/rust-ai-ide-ai/src/refactoring/batch.rs`)

Multi-file orchestration:

```rust
pub struct BatchRefactoringManager {
    operations: Vec<PlannedOperation>,
    dependency_graph: DependencyGraph,
    conflict_detector: ConflictDetector,
    rollback_manager: RollbackManager,
}

impl BatchRefactoringManager {
    pub async fn execute_batch(&self, batch: BatchRefactoring) -> Result<BatchResult> {
        // Dependency ordering
        // Conflict resolution
        // Progress tracking
        // Concurrent execution where safe
        // Rollback on failure
    }
}
```

**Features:**

- **Dependency Ordering**: Automatic operation sequencing
- **Conflict Detection**: Early identification of conflicting changes
- **Concurrent Execution**: Safe parallel processing
- **Rollback Support**: Complete undo capability
- **Progress Aggregation**: Unified progress across all operations

#### Test Generation Module (`crates/rust-ai-ide-ai/src/refactoring/test_generation.rs`)

Automated quality assurance:

```rust
pub struct RefactoringTestGenerator {
    ai_service: Arc<AiService>,
    codebase_analyzer: Arc<CodebaseAnalyzer>,
    test_framework_detector: TestFrameworkDetector,
}

impl RefactoringTestGenerator {
    pub async fn generate_tests(
        &self,
        refactored_code: &RefactoredCode,
        original_code: &OriginalCode
    ) -> Result<GeneratedTests> {
        // Analyze refactored code structure
        // Identify critical paths and edge cases
        // Generate unit tests
        // Generate integration tests if needed
        // Validate test coverage
    }
}
```

**Generated Test Types:**

- **Unit Tests**: Individual function/method testing
- **Integration Tests**: Multi-component interaction testing
- **Regression Tests**: Behavior preservation validation
- **Performance Tests**: Benchmark and load testing

#### Utils Module (`crates/rust-ai-ide-ai/src/refactoring/utils.rs`)

Shared utilities:

```rust
pub struct AstManipulator {
    parser: syn::parse::Parser,
    writer: CodeWriter,
}

impl AstManipulator {
    pub fn parse_rust_code(&self, code: &str) -> Result<syn::File> {
        // AST parsing with error recovery
    }

    pub fn format_code(&self, ast: &syn::File) -> String {
        // Consistent code formatting
    }

    pub fn generate_imports(&self, dependencies: Dependencies) -> TokenStream {
        // Automatic import generation
    }
}
```

### External Integration Layer

#### LSP Integration

Symbol resolution and navigation:

```rust
pub struct LspIntegrator {
    client: Arc<LspClient>,
    capabilities: ServerCapabilities,
}

impl LspIntegrator {
    pub async fn resolve_symbol(&self, position: Position) -> Result<SymbolInfo> {
        // Symbol lookup
        // Reference tracking
        // Definition location
    }

    pub async fn find_usages(&self, symbol: &SymbolInfo) -> Result<Vec<Location>> {
        // Usage detection across files
        // Reference counting
        // Impact assessment data
    }
}
```

#### AI Service Integration

Intelligent analysis and suggestions:

```rust
pub struct AiPoweredAnalyzer {
    ai_service: Arc<AiService>,
    context_builder: ContextBuilder,
    suggestion_engine: SuggestionEngine,
}

impl AiPoweredAnalyzer {
    pub async fn analyze_pattern(&self, code: &str) -> Result<PatternAnalysis> {
        // ML-based pattern recognition
        // Code quality assessment
        // Refactoring opportunity identification
    }

    pub async fn suggest_improvements(
        &self,
        analysis: &PatternAnalysis
    ) -> Result<Vec<RefactoringSuggestion>> {
        // AI-powered recommendation generation
        // Confidence scoring
        // Risk assessment
    }
}
```

## Data Flow Architecture

### Single Operation Flow

1. **User Selection** → Context capture
2. **Analysis** → Context understanding, impact assessment
3. **Validation** → Safety checks, conflict detection
4. **Preview** → Before/after visualization
5. **Execution** → Change application with rollback support
6. **Verification** → Post-execution validation
7. **Test Generation** → Automated quality assurance

### Batch Operation Flow

1. **Operation Collection** → Multiple operations queued
2. **Dependency Analysis** → Operation ordering and conflicts
3. **Validation** → Batch-level safety checks
4. **Preview** → Multi-file change preview
5. **Concurrent Execution** → Safe parallel processing
6. **Progress Aggregation** → Unified progress reporting
7. **Rollback** → Batch-level undo on failure
8. **Verification** → Comprehensive batch validation

## Error Handling Architecture

### Layered Error Handling

```rust
#[derive(thiserror::Error, Debug)]
pub enum RefactoringError {
    #[error("Analysis failed: {0}")]
    Analysis(#[from] AnalysisError),

    #[error("Operation failed: {0}")]
    Operation(#[from] OperationError),

    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationError),

    #[error("LSP communication failed: {0}")]
    Lsp(#[from] LspError),

    #[error("AI service error: {0}")]
    Ai(#[from] AiError),
}

impl RefactoringError {
    pub fn is_recoverable(&self) -> bool {
        // Determine if error allows continuation
    }

    pub fn suggested_action(&self) -> String {
        // Provide user-friendly recovery suggestions
    }
}
```

### Recovery Strategies

- **Automatic Retry**: Transient failures (network, resource contention)
- **Fallback Execution**: Alternative approaches for failed operations
- **Partial Rollback**: Selective undo for batch operations
- **User Intervention**: Clear error messages with recovery options

## Performance Optimizations

### Caching Strategy

```rust
pub struct RefactoringCache {
    analysis_cache: DashMap<String, AnalysisResult>,
    symbol_cache: DashMap<String, SymbolInfo>,
    operation_cache: DashMap<String, OperationResult>,
}

impl RefactoringCache {
    pub fn get_or_compute<T, F>(
        &self,
        key: &str,
        compute: F
    ) -> Result<T>
    where
        F: FnOnce() -> Result<T> {
        // Cache-first computation with TTL
    }
}
```

### Concurrent Processing

```rust
pub struct ConcurrentExecutor {
    thread_pool: ThreadPool,
    semaphore: Arc<Semaphore>,
    progress_collector: Arc<Mutex<ProgressAggregator>>,
}

impl ConcurrentExecutor {
    pub async fn execute_batch(&self, operations: Vec<Operation>) -> Result<BatchResult> {
        // Safe concurrent execution with resource limits
        // Progress aggregation across threads
        // Error propagation and handling
    }
}
```

## Monitoring and Observability

### Metrics Collection

```rust
pub struct RefactoringMetrics {
    operations_completed: Counter,
    operations_failed: Counter,
    average_execution_time: Histogram,
    cache_hit_rate: Gauge,
    concurrent_operations: Gauge,
}

impl RefactoringMetrics {
    pub fn record_operation(&self, operation: &RefactoringOperation, duration: Duration) {
        // Performance tracking
        // Success/failure ratios
        // Operation frequency analysis
    }
}
```

### Health Checks

```rust
pub struct RefactoringHealthChecker {
    lsp_health: LspHealthCheck,
    ai_health: AiHealthCheck,
    disk_space: DiskSpaceCheck,
    memory_usage: MemoryUsageCheck,
}

impl RefactoringHealthChecker {
    pub async fn check_system_health(&self) -> HealthStatus {
        // Component availability
        // Resource utilization
        // Performance degradation detection
    }
}
```

## Extension Architecture

### Plugin System

```rust
pub trait RefactoringPlugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn supported_operations(&self) -> Vec<RefactoringType>;

    async fn analyze(
        &self,
        context: &RefactoringContext
    ) -> Result<Option<PluginAnalysis>>;

    async fn execute(
        &self,
        operation: &RefactoringOperation
    ) -> Result<Option<CodeChanges>>;
}
```

### Custom Operations

Developers can extend the system with custom refactoring operations:

```rust
#[derive(Clone)]
pub struct CustomExtractPattern {
    pattern_matcher: PatternMatcher,
    code_generator: CodeGenerator,
}

impl RefactoringOperation for CustomExtractPattern {
    async fn analyze(&self, context: RefactoringContext) -> Result<RefactoringResult> {
        // Custom analysis logic
    }

    async fn apply(&self, context: RefactoringContext) -> Result<CodeChanges> {
        // Custom transformation logic
    }
}
```

This architecture ensures the refactoring system is robust, extensible, and performance-optimized while maintaining safety and reliability throughout the entire development workflow.
