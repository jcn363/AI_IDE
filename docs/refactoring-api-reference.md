# Refactoring System API Reference

This document provides comprehensive API reference for the Rust AI IDE's Advanced Refactoring System, covering all public interfaces, data types, and usage patterns.

## Table of Contents

- [Frontend APIs](#frontend-apis)
- [Backend APIs](#backend-apis)
- [Type Definitions](#type-definitions)
- [Command Reference](#command-reference)
- [Operation Reference](#operation-reference)
- [Error Handling](#error-handling)

## Frontend APIs

### RefactoringService Class

The main service class for interacting with the refactoring system from the frontend.

```typescript
import { RefactoringService } from '@/features/ai/services/RefactoringService';

const service = new RefactoringService();
```

#### Methods

##### analyzeContext(context: CodeContext): Promise<AnalysisResult>

Analyzes the provided code context to determine available refactoring operations.

**Parameters:**

- `context: CodeContext` - The code context to analyze

**Returns:** `Promise<AnalysisResult>` - Available operations and analysis data

**Example:**

```typescript
const context: CodeContext = {
    filePath: './src/user.ts',
    selectedText: 'function getUser(id: string) { ... }',
    cursorPosition: { line: 10, column: 5 }
};

const result = await service.analyzeContext(context);
console.log('Available operations:', result.operations);
```

##### executeOperation(operation: RefactoringOperation): Promise<RefactoringResult>

Executes a single refactoring operation.

**Parameters:**

- `operation: RefactoringOperation` - The operation to execute

**Returns:** `Promise<RefactoringResult>` - Execution result with changes

**Example:**

```typescript
const operation: RefactoringOperation = {
    type: 'extract-method',
    context: context,
    options: {
        methodName: 'getUserDetails',
        visibility: 'private'
    }
};

const result = await service.executeOperation(operation);
if (result.success) {
    console.log('Refactoring completed:', result.changes);
}
```

##### executeBatch(batch: BatchRefactoring): Promise<BatchResult>

Executes multiple refactoring operations as a batch.

**Parameters:**

- `batch: BatchRefactoring` - The batch operation configuration

**Returns:** `Promise<BatchResult>` - Batch execution results

**Example:**

```typescript
const batch: BatchRefactoring = {
    operations: [
        { type: 'rename-variable', oldName: 'user', newName: 'userInfo' },
        { type: 'extract-method', methodName: 'formatUser' }
    ],
    conflictResolution: 'merge'
};

const result = await service.executeBatch(batch);
```

##### getImpactAnalysis(operation: RefactoringOperation): Promise<ImpactAnalysis>

Analyzes the potential impact of a refactoring operation.

**Parameters:**

- `operation: RefactoringOperation` - The operation to analyze

**Returns:** `Promise<ImpactAnalysis>` - Detailed impact assessment

##### generateTests(refactoredCode: string, options?: TestGenerationOptions): Promise<GeneratedTests>

Generates tests for refactored code.

**Parameters:**

- `refactoredCode: string` - The refactored code
- `options?: TestGenerationOptions` - Test generation options

**Returns:** `Promise<GeneratedTests>` - Generated test suite

### useRefactoring Hook

React hook for managing refactoring state and operations.

```typescript
import { useRefactoring } from '@/features/ai/hooks/useRefactoring';

const {
    context,
    operations,
    progress,
    execute,
    batchExecute,
    clearHistory
} = useRefactoring({
    autoSave: true,
    previewEnabled: true
});
```

#### Return Values

```typescript
interface UseRefactoringReturn {
    // Current context
    context: RefactoringContext | null;

    // Available operations
    operations: RefactoringType[];

    // Current operation state
    currentOperation: RefactoringOperation | null;

    // Operation history
    history: RefactoringHistory[];

    // Loading states
    isAnalyzing: boolean;
    isExecuting: boolean;

    // Progress tracking
    progress: ProgressState | null;

    // Error state
    error: RefactoringError | null;

    // Methods
    analyze: (context: CodeContext) => Promise<void>;
    execute: (operation: RefactoringOperation) => Promise<RefactoringResult>;
    batchExecute: (batch: BatchRefactoring) => Promise<BatchResult>;
    clearHistory: () => void;
    undoLast: () => Promise<void>;
}
```

## Backend APIs

### Rust Backend

#### RefactoringAnalyzer

Main analysis engine for refactoring operations.

```rust
use rust_ai_ide_ai::refactoring::analysis::RefactoringAnalyzer;

let analyzer = RefactoringAnalyzer::new(lsp_client, ai_service);

// Analyze code context
let analysis = analyzer.analyze_context(context).await?;

// Get refactoring suggestions
let suggestions = analyzer.get_suggestions(&analysis).await?;
```

#### Methods

##### async fn analyze_context(&self, context: RefactoringContext) -> Result<AnalysisResult>

Performs comprehensive analysis of refactoring context.

##### async fn check_applicability(&self, operation: &RefactoringOperation) -> Result<bool>

Checks if an operation can be applied to the current context.

##### async fn analyze_impact(&self, operation: &RefactoringOperation) -> Result<ImpactAnalysis>

Analyzes the impact of a refactoring operation.

### RefactoringOperation Trait

Trait that all refactoring operations must implement.

```rust
pub trait RefactoringOperation {
    /// Analyze if operation can be applied
    async fn analyze(&self, context: RefactoringContext) -> Result<RefactoringResult>;

    /// Check basic applicability
    fn can_apply(&self, context: &RefactoringContext) -> bool;

    /// Apply the refactoring operation
    async fn apply(&self, context: RefactoringContext) -> Result<CodeChanges>;

    /// Get operation priority for batch processing
    fn priority(&self) -> OperationPriority;
}
```

#### Built-in Operations

##### RenameOperation

```rust
pub struct RenameOperation {
    analyzer: Arc<RefactoringAnalyzer>,
}

impl RenameOperation {
    pub fn new(analyzer: Arc<RefactoringAnalyzer>) -> Self;

    // Rename symbols across files
    pub async fn rename_symbol(
        &self,
        symbol: &str,
        new_name: &str,
        scope: RenameScope
    ) -> Result<CodeChanges>;
}
```

##### ExtractMethodOperation

```rust
pub struct ExtractMethodOperation {
    analyzer: Arc<RefactoringAnalyzer>,
}

impl ExtractMethodOperation {
    pub fn new(analyzer: Arc<RefactoringAnalyzer>) -> Self;

    // Extract code block to method
    pub async fn extract_method(
        &self,
        code_block: &str,
        method_name: &str,
        parameters: Vec<Parameter>
    ) -> Result<CodeChanges>;
}
```

### BatchRefactoringManager

Handles batch refactoring operations with dependency management.

```rust
use rust_ai_ide_ai::refactoring::batch::BatchRefactoringManager;

let manager = BatchRefactoringManager::new();

// Execute batch with dependency ordering
let result = manager.execute_batch(batch_config).await?;
```

#### Methods

##### async fn execute_batch(&self, batch: BatchRefactoring) -> Result<BatchResult>

Executes multiple operations with automatic dependency ordering.

##### async fn validate_batch(&self, operations: &[RefactoringOperation]) -> Result<ValidationResult>

Validates batch operations for conflicts and dependencies.

##### async fn rollback_batch(&self, batch_id: &str) -> Result<()>

Rolls back all operations in a batch.

## Type Definitions

### Core Types

```typescript
// Refactoring operation types
export type RefactoringType =
    | 'rename-variable'
    | 'rename-function'
    | 'rename-class'
    | 'extract-variable'
    | 'extract-method'
    | 'extract-interface'
    | 'move-method'
    | 'move-class'
    | 'inline-variable'
    | 'inline-method'
    | 'convert-to-async'
    | 'change-signature'
    | 'introduce-parameter'
    | 'split-class'
    | 'merge-classes'
    | 'pattern-conversion';

// Code context for analysis
export interface CodeContext {
    filePath: string;
    selectedText: string;
    cursorPosition: Position;
    projectRoot: string;
    languageId: string;
}

// Refactoring operation configuration
export interface RefactoringOperation {
    type: RefactoringType;
    context: CodeContext;
    options: Record<string, any>;
    metadata?: OperationMetadata;
}

// Batch refactoring configuration
export interface BatchRefactoring {
    name: string;
    description?: string;
    operations: RefactoringOperation[];
    conflictResolution: ConflictResolutionStrategy;
    rollbackOnFailure: boolean;
    maxConcurrency: number;
}
```

### Result Types

```typescript
export interface RefactoringResult {
    success: boolean;
    changes: CodeChange[];
    warnings: string[];
    errors: string[];
    duration: number;
    rollbackSupported: boolean;
}

export interface BatchResult {
    success: boolean;
    operationResults: OperationResult[];
    overallChanges: CodeChange[];
    conflictsResolved: ConflictResolution[];
    rollbackAvailable: boolean;
    executionTime: number;
}

export interface AnalysisResult {
    context: AnalysisContext;
    availableOperations: RefactoringType[];
    impactAssessment: ImpactAssessment;
    aiSuggestions: AISuggestion[];
    confidence: number;
}
```

### Data Structures

```typescript
export interface CodeChange {
    filePath: string;
    changes: TextChange[];
    originalContent: string;
    newContent: string;
    description: string;
}

export interface TextChange {
    range: Range;
    newText: string;
    oldText: string;
}

export interface ImpactAssessment {
    filesAffected: string[];
    linesChanged: number;
    symbolsAffected: SymbolInfo[];
    riskLevel: 'low' | 'medium' | 'high';
    breakingChanges: BreakingChange[];
    dependencies: DependencyImpact[];
}

export interface SymbolInfo {
    name: string;
    kind: SymbolKind;
    location: Location;
    references: number;
}
```

## Command Reference

### Tauri Commands

#### analyze_refactoring_context

Analyzes the current code context for refactoring opportunities.

```typescript
// Command signature
invoke('analyze_refactoring_context', {
    filePath: string,
    selectedText: string,
    cursorPosition: { line: number, column: number },
    projectRoot: string
}): Promise<AnalysisResult>
```

#### get_available_refactorings

Returns a list of available refactoring operations for the current context.

```typescript
invoke('get_available_refactorings', {
    context: CodeContext
}): Promise<RefactoringType[]>
```

#### execute_refactoring

Executes a single refactoring operation.

```typescript
invoke('execute_refactoring', {
    operation: RefactoringOperation,
    previewOnly: boolean
}): Promise<RefactoringResult>
```

#### analyze_refactoring_impact

Analyzes the potential impact of a refactoring operation.

```typescript
invoke('analyze_refactoring_impact', {
    operation: RefactoringOperation,
    depth: 'file' | 'project',
    includeDependencies: boolean
}): Promise<ImpactAnalysis>
```

#### batch_refactoring

Executes multiple refactoring operations as a batch.

```typescript
invoke('batch_refactoring', {
    batch: BatchRefactoring,
    progressCallback?: (progress: ProgressState) => void
}): Promise<BatchResult>
```

#### generate_refactoring_tests

Generates tests for refactored code.

```typescript
invoke('generate_refactoring_tests', {
    refactoredCode: string,
    originalCode: string,
    testType: 'unit' | 'integration' | 'both',
    framework: 'cargo-test' | 'proptest'
}): Promise<GeneratedTests>
```

## Operation Reference

### Basic Operations

#### Rename Operations

| Operation | Description | Parameters |
|-----------|-------------|------------|
| `rename-variable` | Rename variable/function parameter | newName: string, scope: RenameScope |
| `rename-function` | Rename function/method | newName: string, updateReferences: boolean |
| `rename-class` | Rename class/struct | newName: string, updateImplementations: boolean |
| `rename-field` | Rename struct field | newName: string, updateAccesses: boolean |

#### Extract Operations

| Operation | Description | Parameters |
|-----------|-------------|------------|
| `extract-variable` | Extract expression to variable | varName: string, type?: string |
| `extract-method` | Extract statements to method | methodName: string, parameters: Parameter[] |
| `extract-interface` | Extract methods to interface | interfaceName: string, methods: string[] |
| `extract-superclass` | Extract common fields to superclass | superName: string, fields: string[] |

#### Move Operations

| Operation | Description | Parameters |
|-----------|-------------|------------|
| `move-method` | Move method to another class | targetClass: string, newName?: string |
| `move-class` | Move class to another module | targetModule: string |
| `move-field` | Move field between classes | targetClass: string |

#### Inline Operations

| Operation | Description | Parameters |
|-----------|-------------|------------|
| `inline-variable` | Inline variable usage | removeDeclaration: boolean |
| `inline-method` | Inline method call | preserveOriginal: boolean |
| `inline-constant` | Inline constant value | replaceAll: boolean |

### Advanced Operations

#### Async/Await Conversion

```typescript
interface AsyncConvertOptions {
    errorHandling: 'propagate' | 'wrap' | 'unwrap';
    asyncPrefix: string;
    awaitPrefix: string;
    generateTests: boolean;
}
```

#### Pattern Conversion

```typescript
interface PatternConversionOptions {
    fromPattern: DesignPattern;
    toPattern: DesignPattern;
    preserveBehavior: boolean;
    generateValidation: boolean;
}

type DesignPattern =
    | 'strategy'
    | 'factory'
    | 'builder'
    | 'singleton'
    | 'observer'
    | 'decorator'
    | 'composite'
    | 'template-method';
```

#### Signature Changes

```typescript
interface ChangeSignatureOptions {
    target: 'function' | 'method';
    changes: SignatureChange[];
    updateCalls: boolean;
    preserveCompatibility: boolean;
}

interface SignatureChange {
    action: 'add' | 'remove' | 'rename' | 'reorder';
    parameter?: string;
    newName?: string;
    defaultValue?: string;
}
```

## Error Handling

### Error Types

```typescript
export enum RefactoringErrorCode {
    CONTEXT_INVALID = 'CONTEXT_INVALID',
    OPERATION_UNSUPPORTED = 'OPERATION_UNSUPPORTED',
    DEPENDENCY_NOT_FOUND = 'DEPENDENCY_NOT_FOUND',
    IMPACT_TOO_HIGH = 'IMPACT_TOO_HIGH',
    LSP_UNAVAILABLE = 'LSP_UNAVAILABLE',
    AI_SERVICE_FAILED = 'AI_SERVICE_FAILED',
    VALIDATION_FAILED = 'VALIDATION_FAILED',
    CONFLICT_DETECTED = 'CONFLICT_DETECTED',
    EXECUTION_FAILED = 'EXECUTION_FAILED',
    ROLLBACK_FAILED = 'ROLLBACK_FAILED'
}

export interface RefactoringError {
    code: RefactoringErrorCode;
    message: string;
    details?: Record<string, any>;
    recoverable: boolean;
    suggestedActions?: string[];
}
```

### Error Recovery

```typescript
// Error with recovery suggestions
try {
    await service.executeOperation(operation);
} catch (error) {
    if (error.recoverable) {
        // Show recovery options
        const recoveryOptions = await service.getRecoveryOptions(error);

        if (recoveryOptions.length > 0) {
            // Present options to user
            const selectedOption = await presentRecoveryOptions(recoveryOptions);
            await service.applyRecoveryOption(selectedOption);
        }
    } else {
        // Show error message
        showErrorDialog(error.message);
    }
}
```

### Batch Error Handling

```typescript
// Continue on individual operation failures
const batchOptions = {
    continueOnIndividualFailure: true,
    rollbackOnBatchFailure: true,
    maxFailures: 3
};

const result = await service.executeBatch(batch, batchOptions);

if (result.partialSuccess) {
    console.log(`${result.successfulOps} succeeded, ${result.failedOps} failed`);
    // Handle partial success
}
```

## Best Practices

### API Usage Patterns

#### Progressive Enhancement

```typescript
// Start with basic analysis
const analysis = await service.analyzeContext(context);

// Apply user preferences
const operation = analysis.operations.find(op => op.type === preferredType);

// Preview before execution
const preview = await service.getPreview(operation);
await showPreviewDialog(preview);

// Execute with proper error handling
try {
    const result = await service.executeOperation(operation);
    await handleSuccess(result);
} catch (error) {
    await handleError(error);
}
```

#### Resource Management

```typescript
// Clean up resources
const service = new RefactoringService();

// Use try/finally for cleanup
try {
    const result = await service.executeOperation(operation);
} finally {
    await service.cleanup();
}

// Or use the hook's cleanup
useEffect(() => {
    return () => {
        // Cleanup on unmount
    };
}, []);
```

#### Performance Considerations

```typescript
// Prefer batch operations for multiple changes
const batch = {
    operations: [op1, op2, op3],
    maxConcurrency: 2, // Limit concurrent operations
    progressCallback: (progress) => {
        // Update progress UI
    }
};

const result = await service.executeBatch(batch);
```

This API reference provides comprehensive coverage of the refactoring system's capabilities. For additional examples and advanced usage patterns, refer to the main documentation.
