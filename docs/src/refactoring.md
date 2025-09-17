# Advanced Refactoring System Documentation

Welcome to the comprehensive documentation for the Rust AI IDE's Advanced Refactoring System - a sophisticated, AI-powered refactoring platform.

## Overview

The Advanced Refactoring System transforms code restructuring through intelligent analysis, safety-first operations, and AI-enhanced capabilities.

### Key Features

| Feature | Description | AI-Enhanced |
|---------|-------------|-------------|
| 🔄 **Smart Operations** | Rename, extract, move, convert operations | With impact analysis |
| 🎯 **Pattern Recognition** | Detect code patterns and anti-patterns | ML-based detection |
| 🔍 **Safety Validation** | Pre-flight checks and validation | Risk assessment |
| 📦 **Batch Processing** | Multi-file, multi-operation batches | Dependency ordering |
| 🧪 **Test Generation** | Automated test creation | Coverage optimization |

## Architecture

### Layered Design

```text
Frontend (React/TypeScript)
├── RefactoringPanel: UI orchestration
├── Wizard Components: Specialized UIs
├── useRefactoring Hook: State management
└── RefactoringService: Backend API

Backend (Rust/Tauri)
├── Command Layer: Serialization & routing
├── Refactoring Engine: Core logic
│   ├── analysis: Context & impact assessment
│   ├── operations: Concrete implementations
│   ├── batch: Multi-file management
│   ├── test_generation: Automated tests
│   └── utils: Shared utilities
└── AI Integration: Intelligent suggestions
```

## Core Operations

### Basic Refactoring

#### Rename Operations

```typescript
await refactor({
    operation: 'rename',
    target: {
        type: 'variable',
        name: 'oldName',
        newName: 'newName',
        scope: 'local'
    }
});
```

#### Extract Operations

**Extract Variable**

```typescript
// Before
const result = complexCalculation() + anotherValue;

// After
const calculationResult = complexCalculation();
const result = calculationResult + anotherValue;
```

**Extract Method**

```typescript
function processUserData(users: User[]) {
    for (let user of users) {
        if (isActiveUser(user)) {
            notifyUser(user);
        }
    }
}

function isActiveUser(user: User): boolean {
    return user.active;
}

function notifyUser(user: User) {
    console.log(user.name);
    sendEmail(user.email, "Welcome");
}
```

### Advanced Operations

#### Async/Await Conversion

**Pattern Recognition**

```rust
// Detected pattern
fn fetch_user(id: u64) -> Result<User, Error> {
    let data = make_http_call("/user/${id}")?;
    let parsed = parse_json(&data)?;
    Ok(parsed)
}

// AI suggests async conversion for better performance
```

**Automated Conversion**

```typescript
const conversion = await convertToAsync({
    function: selectedFunction,
    dependencies: await analyzeDependencies(selectedFunction),
    errorHandling: 'propagate',
    testGeneration: true
});
```

#### Interface Extraction

```typescript
// From class analysis
class UserService {
    async getUser(id: string): Promise<User>
    async createUser(user: UserInput): Promise<User>
    async updateUser(id: string, user: UserInput): Promise<User>
    async deleteUser(id: string): Promise<void>
}

// AI detects: "High cohesion - extract interface"
interface IUserService {
    getUser(id: string): Promise<User>;
    createUser(user: UserInput): Promise<User>;
    updateUser(id: string, user: UserInput): Promise<User>;
    deleteUser(id: string): Promise<void>;
}

class UserService implements IUserService {
    // Implementation remains unchanged
}
```

#### Design Pattern Conversion

**Strategy Pattern Example**

```typescript
// Before: Conditional logic
if (paymentType === 'credit') {
    processCreditCard(amount);
} else if (paymentType === 'paypal') {
    processPayPal(amount);
}

// After: Strategy pattern
interface PaymentStrategy {
    process(amount: number): Promise<void>;
}

class PaymentProcessor {
    constructor(private strategy: PaymentStrategy) {}

    async process(amount: number) {
        return this.strategy.process(amount);
    }
}
```

## Batch Refactoring

### Workflow Example

```typescript
const batchRefactoring = {
    name: "Modernization Batch",
    operations: [
        {
            type: 'convert-to-async',
            scope: 'file',
            path: './services/user-service.ts'
        },
        {
            type: 'extract-interface',
            className: 'UserService',
            newInterface: 'IUserOperations'
        }
    ],
    validation: {
        checkConflicts: true,
        previewChanges: true
    }
};

const batchResult = await executeBatch(batchRefactoring);
```

### Conflict Resolution

```typescript
if (batchResult.conflicts.length > 0) {
    const resolution = await resolveConflicts(batchResult.conflicts, {
        strategy: 'merge',
        prioritize: 'last-operation'
    });
    await applyResolution(resolution);
}
```

## AI-Powered Features

### Pattern Recognition

- **Code Smell Detection**: Long methods, large classes, duplicate code
- **Anti-pattern Recognition**: Singleton abuse, God object, feature envy
- **Improvement Suggestions**: Performance optimizations, maintainability enhancements

### Impact Analysis

```typescript
const impact = await analyzeRefactoringImpact({
    operation: selectedOperation,
    analysisDepth: 'full'
});

console.log(`
Files Affected: ${impact.filesAffected.length}
Risk Level: ${impact.riskLevel}
Breaking Changes: ${impact.breakingChanges}
`);
```

### Test Generation

```typescript
const tests = await generateTestsForRefactoring({
    refactoredCode: newImplementation,
    originalCode: originalImplementation,
    testType: 'both',
    coverage: 'high'
});
```

## Getting Started

### Configuration

```json
{
    "refactoring": {
        "enabled": true,
        "aiProvider": "local",
        "safetyChecks": true,
        "testGeneration": true
    }
}
```

### Basic Workflow

1. **Select Code**: Highlight target code
2. **Choose Operation**: Pick refactoring type
3. **Configure**: Set options and scope
4. **Preview**: Review before/after changes
5. **Execute**: Apply with safety checks

## User Interface

### Refactoring Panel

The main UI provides:

- Operation selection with AI suggestions
- Context-aware recommendations
- Before/after preview with syntax highlighting
- Impact assessment with risk indicators
- Progress tracking for long operations

### Wizards

#### Async/Await Wizard

- Function selection and dependency analysis
- Error handling strategy configuration
- Preview of generated async code
- Optional test generation

#### Extract Interface Wizard

- Class/method selection interface
- Interface naming and location options
- Preview of generated interface
- Implementation update options

#### Pattern Conversion Wizard

- Pattern type selection (Strategy, Factory, Builder, etc.)
- Code analysis and candidate identification
- Conversion preview and validation
- Migration path suggestions

## Developer Guide

### Adding New Operations

#### Backend Implementation

```rust
pub struct NewOperation<'a> {
    analyzer: &'a RefactoringAnalyzer,
}

impl<'a> NewOperation<'a> {
    pub fn analyze(&self, context: RefactoringContext) -> Result<RefactoringResult> {
        // Analysis logic
        Ok(RefactoringResult { /* results */ })
    }

    pub fn apply(&self, context: RefactoringContext) -> Result<CodeChanges> {
        // Transformation logic
        Ok(changes)
    }
}
```

#### Frontend Integration

```typescript
// Add to wizard components
export const NewOperationWizard: React.FC = () => {
    return (
        <div>
            <h2>New Operation</h2>
            {/* Implementation */}
        </div>
    );
};
```

### Testing

#### Unit Tests

```rust
#[tokio::test]
async fn test_new_operation() {
    let analyzer = RefactoringAnalyzer::new();
    let operation = NewOperation::new(&analyzer);
    let context = create_test_context();

    let result = operation.analyze(context).await;
    assert!(result.is_ok());
}
```

#### Integration Tests

```typescript
test('new operation works correctly', async () => {
    const service = new RefactoringService();
    const result = await service.executeOperation({
        type: 'NewOperation',
        context: mockContext
    });

    expect(result.success).toBe(true);
    expect(result.changes).toHaveLength(1);
});
```

## Best Practices

### Code Preparation

1. Ensure code compiles before refactoring
2. Commit changes to version control
3. Write tests for complex logic
4. Keep backups of critical files

### Operation Selection

1. Start with smaller, safer refactorings
2. Use AI suggestions for complex operations
3. Validate scope and dependencies
4. Preview changes before applying

### Safety Guidelines

1. Always review impact analysis
2. Test after refactoring
3. Apply complex changes gradually
4. Use batch operations for related changes

### Performance Optimization

1. Use batch operations for multiple changes
2. Apply appropriate scopes (local vs project-wide)
3. Leverage caching for repeated operations
4. Monitor memory usage for large operations

## Troubleshooting

### Common Issues

#### Operation Fails to Apply

- **Cause**: Invalid code selection or missing dependencies
- **Solution**: Check code compiles and all dependencies resolve

#### High Risk Assessment

- **Cause**: Complex dependencies or side effects
- **Solution**: Preview changes, reduce scope, apply incrementally

#### Performance Issues

- **Cause**: Large scope or complex analysis
- **Solution**: Use local scope, batch operations, ensure AI service configured

### Error Messages

| Error Message | Cause | Solution |
|---------------|-------|----------|
| Cannot analyze context | Invalid selection | Select complete statement/function |
| Dependencies not resolved | Missing imports | Ensure all dependencies available |
| Type system conflict | Type mismatches | Review type annotations |

## API Reference

### Core Types

```typescript
interface RefactoringContext {
    filePath: string;
    selection: SelectionRange;
    symbolInfo?: SymbolInformation;
    projectRoot: string;
}

interface RefactoringOperation {
    type: RefactoringType;
    context: RefactoringContext;
    options: Record<string, any>;
}

interface RefactoringResult {
    success: boolean;
    changes: CodeChange[];
    warnings: string[];
    errors: string[];
}
```

### Command API

```typescript
// Execute single refactoring
invoke('execute_refactoring', operation: RefactoringOperation)

// Analyze refactoring impact
invoke('analyze_refactoring_impact', context: RefactoringContext)

// Get available refactorings
invoke('get_available_refactorings', context: RefactoringContext)

// Batch refactoring
invoke('batch_refactoring', operations: RefactoringOperation[])
```

## Migration Guide

### From Manual Refactoring

1. Enable refactoring system in settings
2. Start with simple operations (rename, extract variable)
3. Gradually adopt advanced operations
4. Integrate into existing workflow

### From Other Tools

1. Export existing refactoring rules
2. Map operations to new system types
3. Configure AI provider preferences
4. Test with small codebase sections first

## Module Structure Improvements (Section 5.2)

As part of our core refactoring foundation, we have implemented structural improvements to maintainability through strategic module splitting. Large modules exceeding 1000 lines have been refactored into focused sub-modules following Rust best practices.

### Recent Structural Refactoring

#### Commands Module Refactoring
**File**: `src-tauri/src/commands/refactoring_commands.rs`
**Original Size**: 1720 lines
**Refactored Into**: 3 focused modules

**New Module Structure**:
```
refactoring_commands/
├── types.rs         (487 lines - Type mappings, DTOs, data structures)
├── analysis.rs       (366 lines - Code analysis utilities)
├── commands.rs       (516 lines - Main refactoring command implementations)
└── mod.rs           (21 lines - Module declarations and re-exports)
```

**Benefits**:
- ✅ **Improved Maintainability**: Each module focuses on a single responsibility
- ✅ **Better Navigation**: Easier to locate and understand code sections
- ✅ **Enhanced Testability**: Smaller, focused modules are easier to unit test
- ✅ **Reduced Complexity**: Complex functions are logically organized
- ✅ **Preserved APIs**: All public interfaces remain unchanged through re-exports
- ✅ **Develop Productivity**: Faster code discovery and comprehension

**Refactoring Metrics**:
| Aspect | Before | After | Improvement |
|--------|--------|-------|-------------|
| Largest module | 1720 lines | 516 lines | 70% size reduction |
| Module count | 1 monolithic | 4 focused | Better separation |
| Navigation | File scanning | Direct module access | Improved workflow |
| API compatibility | N/A | 100% maintained | Zero breaking changes |

### Implementation Guidelines

1. **Module Size Targets**
   - Primary modules: 200-500 lines
   - Utility modules: < 300 lines
   - Data structure modules: < 400 lines

2. **Separation Criteria**
   - Group related types and DTOs together
   - Isolate utility functions by purpose
   - Separate command handlers from core logic

3. **Maintaining Compatibility**
   - Use `pub use` to re-export public items
   - Preserve all existing import paths
   - Document any potential migration paths

### Future Module Improvements

The same refactoring pattern will be applied to other large modules identified during analysis:
- `crates/rust-ai-ide-ai/src/analysis/security.rs` (1172 lines)
- `crates/rust-ai-ide-common/src/logging.rs` (1376 lines)
- `crates/rust-ai-ide-ai-refactoring/src/operations.rs` (1550 lines)

## Future Enhancements

- **ML-Enhanced Pattern Recognition**: More sophisticated pattern detection
- **Cross-Language Refactoring**: Support for multi-language projects
- **Team Collaboration**: Shared refactoring workflows
- **Advanced Impact Analysis**: Deeper dependency and performance impact assessment
- **Custom Operation Creation**: User-defined refactoring operations
- **Integration with CI/CD**: Automated refactoring in build pipelines

---

This documentation covers the complete Advanced Refactoring System. For community support, visit our forums or GitHub issues.
