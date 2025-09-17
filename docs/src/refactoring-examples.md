# Refactoring Examples & Use Cases

This document provides practical examples and real-world use cases for the Advanced Refactoring System, demonstrating how to leverage its capabilities for code improvement, modernization, and maintenance.

## Table of Contents

- [Basic Refactoring Examples](#basic-refactoring-examples)
- [Advanced Refactoring Scenarios](#advanced-refactoring-scenarios)
- [Real-World Use Cases](#real-world-use-cases)
- [Pattern Recognition Examples](#pattern-recognition-examples)
- [Batch Refactoring Workflows](#batch-refactoring-workflows)
- [Integration Examples](#integration-examples)

## Basic Refactoring Examples

### 1. Extract Method Example

**Before Refactoring:**

```rust
fn process_user_data(mut users: Vec<User>, config: &Config) {
    // Validate input
    if users.is_empty() {
        return;
    }

    // Filter active users
    users.retain(|user| user.active);

    // Apply business rules
    for user in &mut users {
        if user.score > config.threshold {
            user.status = UserStatus::Premium;
        } else if user.score < config.min_score {
            user.status = UserStatus::Inactive;
        } else {
            user.status = UserStatus::Standard;
        }
    }

    // Save to database
    for user in &users {
        save_user(user).await?;
    }

    // Send notifications
    for user in users.iter().filter(|u| u.status == UserStatus::Premium) {
        send_notification(user.email, "Welcome to Premium!").await?;
    }
}
```

**AI Suggestion**: "This function is doing too much. Consider extracting methods for validation, filtering, and notification logic."

**After Refactoring:**

```typescript
// Use the refactoring wizard
const context = {
    filePath: './src/user_service.rs',
    selectedText: 'fn process_user_data(mut users: Vec<User>, config: &Config) { ... }',
    cursorPosition: { line: 42, column: 5 }
};

// Extract validation logic
const extractValidation = await refactor({
    operation: 'extract-method',
    context: context,
    options: {
        methodName: 'validate_users',
        parameters: ['users'],
        returnType: 'Vec<User>',
        description: 'Validates and filters user input'
    }
});

// Extract business logic application
const extractBusinessLogic = await refactor({
    operation: 'extract-method',
    context: context,
    options: {
        methodName: 'apply_business_rules',
        parameters: ['users', 'config'],
        returnType: 'Vec<User>',
        description: 'Applies business rules to determine user status'
    }
});

// Extract notification logic
const extractNotifications = await refactor({
    operation: 'extract-method',
    context: context,
    options: {
        methodName: 'send_premium_notifications',
        parameters: ['users'],
        returnType: 'Result<(), Error>',
        description: 'Sends notifications to new premium users'
    }
});
```

**Result:**

```rust
fn process_user_data(users: Vec<User>, config: &Config) -> Result<(), Error> {
    let validated_users = validate_users(users)?;
    let processed_users = apply_business_rules(validated_users, config);
    save_users_to_database(&processed_users).await?;
    send_premium_notifications(&processed_users).await
}

fn validate_users(users: Vec<User>) -> Result<Vec<User>, Error> {
    if users.is_empty() {
        return Ok(Vec::new());
    }
    Ok(users.into_iter().filter(|user| user.active).collect())
}

fn apply_business_rules(users: Vec<User>, config: &Config) -> Vec<User> {
    users.into_iter().map(|mut user| {
        if user.score > config.threshold {
            user.status = UserStatus::Premium;
        } else if user.score < config.min_score {
            user.status = UserStatus::Inactive;
        } else {
            user.status = UserStatus::Standard;
        }
        user
    }).collect()
}

async fn save_users_to_database(users: &[User]) -> Result<(), Error> {
    for user in users {
        save_user(user).await?;
    }
    Ok(())
}

async fn send_premium_notifications(users: &[User]) -> Result<(), Error> {
    for user in users.iter().filter(|u| u.status == UserStatus::Premium) {
        send_notification(user.email, "Welcome to Premium!").await?;
    }
    Ok(())
}
```

### 2. Rename Refactoring Example

**Before:**

```rust
// Old naming convention
fn get_usr_dt(id: i32) -> Result<Usr, Error> {
    let usr_repo = UsrRepo::new();
    let usr_dt = usr_repo.find_by_id(id)?;
    Ok(usr_dt)
}
```

**Refactoring Command:**

```typescript
const renameOps = [
    // Rename function
    {
        operation: 'rename-function',
        oldName: 'get_usr_dt',
        newName: 'get_user_details',
        updateReferences: true
    },
    // Rename parameter
    {
        operation: 'rename-variable',
        oldName: 'id',
        newName: 'user_id',
        scope: 'function'
    },
    // Rename types
    {
        operation: 'rename-class',
        oldName: 'Usr',
        newName: 'User',
        updateReferences: true
    },
    {
        operation: 'rename-class',
        oldName: 'UsrRepo',
        newName: 'UserRepository',
        updateReferences: true
    },
    // Rename variable
    {
        operation: 'rename-variable',
        oldName: 'usr_dt',
        newName: 'user_details',
        scope: 'file'
    }
];

const batchResult = await executeBatch({
    name: 'Rename User DTO Types',
    operations: renameOps,
    autoResolveConflicts: true
});
```

## Advanced Refactoring Scenarios

### 1. Async/Await Conversion Example

**Before (Synchronous API):**

```rust
fn process_orders(orders: Vec<Order>) -> Result<Vec<ProcessedOrder>, Error> {
    let client = HttpClient::new();
    let mut processed = Vec::new();

    for order in orders {
        // Validate order
        validate_order(&order)?;

        // Calculate pricing
        let price = calculate_pricing(&order, &client)?;

        // Apply discounts
        let discount = get_discount(&order.customer_id, &client)?;

        // Process payment
        let payment_result = process_payment(&order, price - discount, &client)?;

        processed.push(ProcessedOrder {
            order_id: order.id,
            final_price: price - discount,
            payment_status: payment_result.status,
        });
    }

    Ok(processed)
}
```

**AI Detection**: "Synchronous I/O operations detected - candidates for async/await conversion to improve scalability"

**Async/Await Conversion:**

```typescript
// Configure async conversion
const conversion = await convertToAsync({
    function: 'process_orders',
    options: {
        errorHandling: 'propagate',
        generateCancellationTokens: true,
        preserveOrdering: true,
        addTimeoutSupport: true
    }
});

// Review and apply changes
const preview = await conversion.generatePreview();
console.log(`Will convert ${preview.functions_to_modify} functions`);
console.log(`Will add ${preview.dependencies_to_add} async dependencies`);

// Apply the conversion
await conversion.apply();
```

**After refactoring:**

```rust
async fn process_orders(orders: Vec<Order>) -> Result<Vec<ProcessedOrder>, Error> {
    let client = HttpClient::new();
    let mut processed = Vec::new();

    for order in orders {
        // Validate order
        validate_order(&order)?;

        // Calculate pricing concurrently
        let (price_result, discount_result) = tokio::join!(
            calculate_pricing_async(&order, &client),
            get_discount_async(&order.customer_id, &client)
        );

        let price = price_result?;
        let discount = discount_result?;

        // Process payment
        let payment_result = process_payment_async(&order, price - discount, &client).await?;

        processed.push(ProcessedOrder {
            order_id: order.id,
            final_price: price - discount,
            payment_status: payment_result.status,
        });
    }

    Ok(processed)
}
```

### 2. Design Pattern Conversion

**Strategy Pattern Example:**

**Before (Conditional Logic):**

```typescript
class PaymentProcessor {
    processPayment(amount: number, method: string) {
        switch (method) {
            case 'credit':
                return processCreditCard(amount);
            case 'paypal':
                return processPayPal(amount);
            case 'bitcoin':
                return processBitcoin(amount);
            default:
                throw new Error('Unknown payment method');
        }
    }
}
```

**AI Recommendation**: "Conditional logic detected - consider Strategy pattern for better extensibility"

**Pattern Conversion:**

```typescript
// Use pattern conversion wizard
const conversion = await convertPattern({
    from: 'conditional-dispatch',
    to: 'strategy',
    source: 'PaymentProcessor.processPayment',
    options: {
        generateInterface: true,
        extractMethods: true,
        addValidation: true
    }
});

await conversion.apply();
```

**After Pattern Conversion:**

```typescript
interface PaymentStrategy {
    process(amount: number): Promise<PaymentResult>;
}

class CreditCardStrategy implements PaymentStrategy {
    async process(amount: number): Promise<PaymentResult> {
        return processCreditCard(amount);
    }
}

class PayPalStrategy implements PaymentStrategy {
    async process(amount: number): Promise<PaymentResult> {
        return processPayPal(amount);
    }
}

class BitcoinStrategy implements PaymentStrategy {
    async process(amount: number): Promise<PaymentResult> {
        return processBitcoin(amount);
    }
}

class PaymentProcessor {
    constructor(private strategy: PaymentStrategy) {}

    async processPayment(amount: number): Promise<PaymentResult> {
        return this.strategy.process(amount);
    }

    setStrategy(strategy: PaymentStrategy) {
        this.strategy = strategy;
    }
}
```

## Real-World Use Cases

### 1. Microservices Migration

**Scenario**: Splitting a monolithic application into microservices

```typescript
// Analyze monolithic API
const analysis = await analyzeCodebase({
    entryPoints: ['./src/api/*.rs'],
    analysis: {
        couplingAnalysis: true,
        cohesionAnalysis: true,
        domainAnalysis: true
    }
});

// AI suggestions for service boundaries
const suggestions = await getMicroserviceSuggestions(analysis);
console.log(`Suggested ${suggestions.services.length} microservices`);

// Batch extract services
const batchOps = suggestions.services.map(service => ({
    operation: 'extract-microservice',
    context: service.context,
    options: {
        serviceName: service.name,
        dependencies: service.dependencies,
        generateAPI: true,
        generateClient: true
    }
}));

await executeBatch({
    name: 'Microservices Migration',
    operations: batchOps,
    validateAfterEach: true,
    generateTests: true
});
```

### 2. Legacy Code Modernization

**Scenario**: Updating legacy synchronous code to modern async patterns

```typescript
// Identify legacy patterns
const legacyPatterns = [
    'synchronous_http_calls',
    'blocking_file_operations',
    'sequential_processing',
    'thread_blocking_operations'
];

const modernizationOps = [];

// Find and convert legacy patterns
for (const pattern of legacyPatterns) {
    const occurrences = await findPattern(pattern);

    for (const occurrence of occurrences) {
        modernizationOps.push({
            operation: 'modernize-legacy-pattern',
            context: occurrence.context,
            pattern: pattern
        });
    }
}

// Execute modernization
await executeBatch({
    name: 'Legacy Code Modernization',
    operations: modernizationOps,
    conflictResolution: 'intelligent',
    progressTracking: true
});
```

### 3. Code Quality Improvement

**Scenario**: Systematically improving code quality across the codebase

```typescript
// Comprehensive code quality assessment
const qualityAssessment = await assessCodeQuality({
    scope: 'project',
    metrics: [
        'cyclomatic_complexity',
        'cognitive_complexity',
        'maintainability_index',
        'code_duplication',
        'test_coverage'
    ]
});

// Prioritize refactoring opportunities
const refactoringPriorities = [
    { type: 'complexity_reduction', weight: 0.3 },
    { type: 'duplication_elimination', weight: 0.25 },
    { type: 'test_coverage_improvement', weight: 0.2 },
    { type: 'naming_improvement', weight: 0.15 },
    { type: 'structure_optimization', weight: 0.1 }
];

// Generate prioritized task list
const refactoringTasks = await generateRefactoringTasks(
    qualityAssessment,
    refactoringPriorities
);

// Execute prioritized improvements
await executeBatch({
    name: 'Code Quality Campaign',
    operations: refactoringTasks,
    maxConcurrency: 3,
    requireApproval: true // Manual approval for major changes
});
```

## Pattern Recognition Examples

### 1. Code Smell Detection

```typescript
// Long method detection
const longMethodOps = await detectCodeSmells({
    type: 'long-method',
    threshold: 50, // lines
    autoFix: true
});

// Large class detection
const largeClassOps = await detectCodeSmells({
    type: 'large-class',
    threshold: 1000, // LOC
    autoFix: false // Requires manual review
});

// Duplicate code detection
const duplicateOps = await detectCodeSmells({
    type: 'duplicate-code',
    minSimilarity: 0.8,
    minSize: 10, // tokens
});

// Unused code detection
const unusedOps = await detectCodeSmells({
    type: 'unused-code',
    ignoreTests: true
});
```

### 2. Anti-pattern Recognition

```typescript
// Singleton over-abuse
const singletonOps = await detectAntiPatterns('singleton-abuse');

// Tight coupling
const couplingOps = await detectAntiPatterns('tight-coupling');

// Circular dependencies
const circularOps = await detectCircularDependencies({
    scope: 'project',
    includeExternal: false
});
```

## Batch Refactoring Workflows

### 1. Architecture Migration

```typescript
const migrationWorkflow = {
    name: 'Architecture Migration',
    phases: [
        {
            name: 'Preparation',
            operations: [
                { type: 'backup-source', scope: 'project' },
                { type: 'analyze-dependencies', scope: 'project' },
                { type: 'generate-migration-plan', scope: 'project' }
            ]
        },
        {
            name: 'Structural Changes',
            operations: [
                { type: 'extract-modules', scope: 'project' },
                { type: 'move-classes', scope: 'multi-file' },
                { type: 'update-imports', scope: 'project' }
            ]
        },
        {
            name: 'Update References',
            operations: [
                { type: 'update-references', scope: 'project' },
                { type: 'fix-import-paths', scope: 'project' },
                { type: 'update-documentation', scope: 'project' }
            ]
        },
        {
            name: 'Verification',
            operations: [
                { type: 'run-tests', scope: 'project' },
                { type: 'validate-compilation', scope: 'project' },
                { type: 'generate-reports', scope: 'project' }
            ]
        }
    ],
    errorHandling: 'rollback-on-failure'
};

await executeWorkflow(migrationWorkflow);
```

### 2. API Modernization

```typescript
// Modernize REST API to GraphQL
const apiModernization = {
    name: 'API Modernization',
    operations: [
        // Extract data models
        {
            type: 'batch-extract-models',
            scope: 'api',
            target: 'graphql-types'
        },
        // Convert endpoints to resolvers
        {
            type: 'convert-endpoints-to-resolvers',
            scope: 'api',
            mapping: 'rest-to-graphql'
        },
        // Generate GraphQL schema
        {
            type: 'generate-graphql-schema',
            scope: 'project',
            includeRelations: true
        },
        // Update API clients
        {
            type: 'update-api-clients',
            scope: 'clients',
            fromFormat: 'rest',
            toFormat: 'graphql'
        }
    ]
};

await executeBatch(apiModernization);
```

## Integration Examples

### 1. CI/CD Integration

```yaml
# .github/workflows/refactoring.yml
name: Automated Refactoring
on:
  schedule:
    - cron: '0 2 * * 1'  # Weekly on Monday
  workflow_dispatch:

jobs:
  refactor:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run AI-Powered Refactoring
        run: |
          # Analyze codebase
          rust-ai-ide refactor analyze --output analysis.json

          # Execute scheduled refactorings
          rust-ai-ide refactor batch --config refactoring-rules.json

          # Generate pull request
          rust-ai-ide refactor pr-create --branch refactor/auto-improvements

      - name: Run Tests
        run: cargo test --all-features
```

### 2. Pre-commit Hooks

```bash
# .git/hooks/pre-commit
#!/bin/bash

# Run refactoring analysis
if command -v rust-ai-ide &> /dev/null; then
    echo "Running refactoring analysis..."
    rust-ai-ide refactor lint --scope staged
fi

# Exit with error if refactoring issues found
if [ $? -ne 0 ]; then
    echo "❌ Refactoring issues detected. Please fix before committing."
    exit 1
fi
```

### 3. IDE Integration

```typescript
// VS Code extension integration
import { RefactoringService } from 'rust-ai-ide';

class RefactoringExtension {
    private service: RefactoringService;

    async activate() {
        this.service = new RefactoringService();

        // Register commands
        vscode.commands.registerCommand('rust-ai-ide.refactor', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor) return;

            const selection = editor.selection;
            const context = this.getContextFromEditor(editor, selection);

            const analysis = await this.service.analyzeContext(context);
            this.showRefactoringOptions(analysis.operations);
        });
    }

    private async showRefactoringOptions(operations: RefactoringType[]) {
        const selected = await vscode.window.showQuickPick(
            operations.map(op => ({
                label: this.getOperationLabel(op),
                description: this.getOperationDescription(op),
                operation: op
            }))
        );

        if (selected) {
            await this.executeRefactoring(selected.operation);
        }
    }
}
```

These examples demonstrate the practical application of the Advanced Refactoring System across various scenarios, from basic code improvements to complex architectural transformations. The system provides both automated refactoring capabilities and intelligent guidance for manual improvements.
