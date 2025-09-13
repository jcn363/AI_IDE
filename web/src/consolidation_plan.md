# Frontend Component Consolidation Plan

## Executive Summary

Analysis of the frontend React/TypeScript codebase reveals significant duplication across panels, editors, and UI components. The consolidation plan targets ~35% reduction in code duplication through shared components, utilities, and patterns.

## Current State Analysis

### 1. Shared Component Structures

**Duplication Found:**

- **TabPanel**: Implemented inline in 5+ files (CargoPanel.tsx, PerformanceDashboard.tsx, etc.) and as separate component
- **BasePanel Structure**: `Paper sx={{ p: 2, height: '100%', display: 'flex', flexDirection: 'column' }}` appears in 10+ components
- **a11yProps Utility**: Duplicated in 4 files for tab accessibility

**Impact:** ~500 LOC of duplicated structure code

### 2. UI Pattern Duplication

**Common patterns across components:**

- Button groups with loading/indicators: `{!isLoading ? 'Save' : 'Saving...'}` pattern
- Form sections: `{errors.length > 0 && <Alert severity="error">` pattern
- List containers: Shared list/filter/search templates
- Collapse/expand sections with identical styling
- Checkbox lists with selection handling

**Impact:** ~800 LOC of repeated UI logic

### 3. AI Component Duplication

**Refactoring Wizards:**

- 6 wizards with identical structure: AsyncAwait, Batch, Extract, Pattern, Unified
- Common patterns: `useState([])`, `useEffect(async)`, toggle handlers, validation logic
- Shared UI: Form controls, checklists, configuration panels

**Configuration Panels:**

- Similar initialization/load patterns
- Error handling duplication
- Loading state management

**Impact:** Most significant - ~2000 LOC across wizards alone

### 4. Shared Utilities Discovery

**Error Handling**: `try/catch(console.error('/Failed to../', error))` appears 50+ times
**State Management**: `setState(prev => ({ ...prev, loading: true }))` pattern in 20+ places
**Async Patterns**: Similar data loading, caching, and retry logic

### 5. Hook Consolidation Opportunities

**Common Patterns:**

- API request hooks with loading/error states
- Form submission with validation
- Data synchronization with debouncing
- Undoing redo operations

## Proposed Architecture

### Consolidated Components

```typescript
// 1. BasePanel - Foundation for all panels
export const BasePanel = ({ children, title, actions, ...props }) => (
  <Paper sx={{ p: 2, height: '100%', display: 'flex', flexDirection: 'column' }}>
    <Header title={title} actions={actions} />
    <Content>{children}</Content>
  </Paper>
);

// 2. SharedTabPanel - Single implementation
export const SharedTabPanel = ({ value, index, children, ...props }) => {
  // Consolidated implementation
};

// 3. WizardBase - Base for refactoring wizards
export const WizardBase = ({
  steps,
  onAnalyze,
  renderStep,
  validationRules,
  onComplete
}) => {
  // Common wizard logic
};
```

### Shared Hooks

```typescript
// 1. useAsyncOperation
export const useAsyncOperation = <T>(
  operation: () => Promise<T>,
  options: { onSuccess?; onError?; retry? } = {}
) => {
  // Consolidated async handling
};

// 2. useDataLoader
export const useDataLoader = (fetchFn, options) => {
  // Consistent loading/error patterns
};

// 3. useFormHandler
export const useFormHandler = (initialValues, validationSchema) => {
  // Unified form management
};
```

### Utility Functions

```typescript
// 1. Error handling utilities
export const createErrorHandler = (context: string) => (error: unknown, customMessage?: string) => {
  console.error(`Error in ${context}:`, error);
  return customMessage || error instanceof Error ? error.message : 'An error occurred';
};

// 2. Common formatting
export const formatErrorMessage = createErrorHandler;
export const formatLoadingState = (isLoading, text = 'Processing...') => (isLoading ? text : null);
```

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)

1. Create shared utilities (`utils/consolidated.ts`)
2. Implement BasePanel component
3. Create SharedTabPanel component
4. Establish design system tokens

### Phase 2: Core Consolidation (Week 2-3)

1. Refactor refactoring wizards to use WizardBase
2. Update all panels to use BasePanel
3. Replace inline TabPanel implementations
4. Consolidate a11yProps usage

### Phase 3: Advanced Patterns (Week 4)

1. Implement shared hooks
2. Standardize error handling
3. Consolidate form patterns
4. Update component templates

### Phase 4: Migration & Testing (Week 5)

1. Update import statements
2. Add tests for new components
3. Performance verification
4. Documentation updates

## Architecture Visualization

```mermaid
graph TB
    subgraph "Current Architecture"
        A1[Components Layer]
        A2[Hooks Layer]
        A3[Utilities Layer]
        subgraph "Components Layer"
            A1a[CargoPanel<br/>~500 LOC]
            A1b[PerformancePanel<br/>~300 LOC]
            A1c[DebuggerPanel<br/>~150 LOC]
            A1d[AI Wizards<br/>~800 LOC each]
        end
        subgraph "Hooks Layer"
            A2a[useAIAssistant<br/>~800 LOC]
            A2b[useModelManagement<br/>~100 LOC]
            A2c[useRefactoring<br/>~200 LOC]
            A2d[Local Hooks<br/>~50-150 LOC]
        end
        subgraph "Utilities Layer"
            A3a[Error Handlers<br/>(50+ instances)]
            A3b[State Updaters<br/>(20+ instances)]
            A3c[Formatters<br/>(10+ instances)]
        end
    end

    subgraph "Proposed Architecture"
        B1[Shared Components]
        B2[Shared Hooks]
        B3[Shared Utilities]
        subgraph "Shared Components"
            B1a[BasePanel<br/>~50 LOC]
            B1b[SharedTabPanel<br/>~30 LOC]
            B1c[WizardBase<br/>~100 LOC]
            B1d[Reusable Templates<br//>~200 LOC]
        end
        subgraph "Shared Hooks"
            B2a[useAsyncOperation<br/>~50 LOC]
            B2b[useDataLoader<br/>~40 LOC]
            B2c[useFormHandler<br/>~60 LOC]
            B2d[useLocalStorage<br/>~30 LOC]
        end
        subgraph "Shared Utilities"
            B3a[errorHandler.ts<br/>~25 LOC]
            B3b[stateUtils.ts<br/>~30 LOC]
            B3c[formatUtils.ts<br/>~20 LOC]
        end
    end

    A1 -->|imports| A2
    A1 -->|imports| A3
    A2 -->|imports| A3

    B1 -->|imports| B2
    B1 -->|imports| B3
    B2 -->|imports| B3

    A1a -.->|#refactor| B1a
    A1b -.->|#refactor| B1a
    A1d -.->|#refactor| B1c

    A2a -.->|#consolidate| B2a
    A2a -.->|#consolidate| B2b

    A3a -.->|#consolidate| B3a
    A3b -.->|#consolidate| B3b

    style A1 fill:#ffcccc
    style B1 fill:#ccffcc
```

## Metrics & Benefits

### Quantitative Targets

- **Code Reduction:** 35% (target ~1800-2400 LOC reduction from estimated ~5100 LOC duplication)
- **Component Savings:** ~25 shared components replace ~100+ individual implementations
- **Hook Consolidation:** ~10 shared hooks reduce manual state management by ~500 LOC
- **Duplication Score:** Reduce from current ~25% duplication to <5%
- **Maintainability:** Single source for UI patterns and utilities with 80% code reusability

### Qualitative Benefits

- Consistent UI/UX across panels
- Faster development of new features
- Easier maintenance and updates
- Better testability through shared components
- Improved developer experience

## Dependency Analysis

### Critical Dependencies

- None - consolidation maintains existing APIs
- New shared components are backwards compatible
- Gradual migration possible

### Breaking Changes

- Minimal - most changes internal
- Some prop interface standardization
- Import path updates required

## Migration Guidelines

### For Developers

1. Use shared components instead of custom implementations
2. Prefer consolidated hooks over inline state management
3. Follow established patterns for UI consistency
4. Update imports when consolidation components are available

### Backward Compatibility

- Existing components continue to work during transition
- Gradual adoption through feature flags
- API compatibility maintained where possible

## Risk Mitigation

- Comprehensive testing of consolidated components
- Incremental rollout to production
- Rollback plan for critical issues
- Developer training on new patterns
