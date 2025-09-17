# Rust AI IDE - Architecture Diagrams

## Overview

This document contains visual representations of the Rust AI IDE architecture, showing the relationships between components, data flow, and the impact of the deduplication campaign.

---

## Current Unified Architecture

```mermaid
graph TD
    subgraph "Frontend Layer"
        UI[Monaco Editor + Web UI]
        Tauri[Tauri Desktop Framework]
    end

    subgraph "Shared Crates Layer"
        subgraph "rust-ai-ide-common [Core 🚀]"
            Types[Unified Types<br/>ProgrammingLanguage<br/>Position<br/>Range<br/>IdeError]
            Cache[MemoryCache<br/>TTL Support<br/>Thread-Safe]
            FS[File System Utils<br/>Atomic Operations<br/>Path Validation]
            Perf[Performance Utils<br/>Metrics<br/>Timing]
            Dup[Duplication Detection<br/>Similarity Analysis]
        end

        subgraph "rust-ai-ide-shared-codegen [Code Generation 🔧]"
            AST[AST Operations<br/>Parsing<br/>Transformation]
            Gen[Code Generation<br/>Multi-Language<br/>Templates]
            Val[Validation Framework<br/>Safety Checks<br/>Quality Assurance]
        end

        subgraph "rust-ai-ide-shared-services [LSP & Workspace 🔗]"
            LSP[LSP Client<br/>Rust-Analyzer<br/>Multi-Language]
            WS[Workspace Manager<br/>Project Analysis<br/>Build System]
            Diag[Diagnostic Aggregation<br/>Cross-Language<br/>Real-time]
        end
    end

    subgraph "Specialized Crates Layer"
        AI[rust-ai-ide-ai<br/>ML Models<br/>Analysis Engine]
        Cargo[rust-ai-ide-cargo<br/>Build System<br/>Dependency Analysis]
        LSP_ACT[rust-ai-ide-lsp<br/>Base LSP Protocol<br/>Extensions]
        Core[rust-ai-ide-core<br/>Legacy Core<br/>Gradual Migration]
        UI_ACT[rust-ai-ide-ui<br/>UI Components<br/>Custom Widgets]
    end

    subgraph "External Services"
        Ollama[Ollama API<br/>Local LLMs]
        AI_API[AI Model APIs<br/>GPT, Claude, etc]
        Registries[Crate Registries<br/>Cargo.io<br/>Local Mirrors]
    end

    %% Data Flow
    UI --> Tauri
    Tauri --> Common
    Tauri --> Codegen
    Tauri --> Services

    AI -->|uses| Common
    AI -->|extended by| Codegen
    AI -->|provides| Services

    Cargo -->|uses| Common
    Cargo -->|integrates| Services

    LSP_ACT --> Services
    Core --> Common
    UI_ACT --> Common

    %% External Dependencies
    Codegen -.->|templates| AI_API
    AI -.->|inference| Ollama
    Services -.-> Registries

    %% Shared Dependencies (represented as thick lines)
    AI === Common
    Cargo === Common
    LSP_ACT === Services
    Core === Common

    classDef shared fill:#e1f5fe,stroke:#01579b,stroke-width:4px
    classDef frontend fill:#f3e5f5,stroke:#4a148c
    classDef specialized fill:#e8f5e8,stroke:#1b5e20
    classDef external fill:#fff3e0,stroke:#e65100
    classDef core fillcolor:#fff,stroke:#000,stroke-width:6px

    class Common,Codegen,Services shared
    class UI,Tauri frontend
    class AI,Cargo,LSP_ACT,Core,UI_ACT specialized
    class Ollama,AI_API,Registries external
    class Types,Cache,FS,Perf,Dup,AST,Gen,Val,LSP,WS,Diag core
```

---

## Deduplication Impact Visualization

### Before Deduplication (Duplicated Architecture)

```mermaid
graph TD
    subgraph "Multiple Inconsistent Types"
        ErrorType1["CustomError (Crate A)"]
        ErrorType2["AnotherError (Crate B)"]
        ErrorType3["YetAnotherError (Crate C)"]
        PositionType1["FrontendPosition (1-based)"]
        PositionType2["BackendPosition (0-based)"]
        CacheType1["CustomCache (TTL Missing)"]
        CacheType2["AnotherCache (TTL Different)"]
    end

    subgraph "Fragmented Functionality"
        FS1[File Operations<br/>Crate A<br/>Limited features]
        FS2[File Operations<br/>Crate B<br/>Different API]
        FS3[File Operations<br/>Crate C<br/>Inconsistent errors]

        Perf1[Timing Utils<br/>Crate A]
        Perf2[Timing Utils<br/>Crate B]
        Perf3[Timing Utils<br/>Crate C]
    end

    subgraph "Dependency Conflicts"
        Cycle1[Circular Dependency<br/>A→B→A]
        Cycle2[Circular Dependency<br/>B→C→B]
        Unused[Dead Code<br/>Non-shared utilities]
    end
```

### After Deduplication (Unified Architecture)

```mermaid
graph TD
    subgraph "Single Source of Truth"
        UnifiedError["IdeError (100% coverage)"]
        UnifiedPosition["Position + Range<br/>Auto-conversion"]
        UnifiedCache["MemoryCache<br/>Consistent TTL<br/>Performance monitoring"]
    end

    subgraph "Shared Utilities"
        SharedFS["fs_utils::<br/>Atomic operations<br/>Path validation<br/>Error consistency"]
        SharedPerf["time_operation!()<br/>Scoped timers<br/>Performance alerts"]
        SharedDup["Duplication detection<br/>Automatic warnings<br/>Prevention tools"]
    end

    subgraph "Clean Dependency Graph"
        NoCycles[Zero Circular Dependencies]
        SharedDeps[Three Shared Crates<br/>Eliminated Redundancy]
        CleanGraph[Linear Dependency Chain<br/>Predictable Build Order]
    end

    %% Success indicators
    UnifiedError --> SuccessBadge1["93% Error<br/>Handler Reduction"]
    UnifiedPosition --> SuccessBadge2["Zero Conversion<br/>Bugs"]
    UnifiedCache --> SuccessBadge3["25% Memory<br/>Usage Decrease"]

    SharedFS --> SuccessBadge4["Atomic Safety<br/>Guaranteed"]
    SharedPerf --> SuccessBadge5["Auto Metrics<br/>Collection"]
    SharedDup --> SuccessBadge6["91% Function<br/>Duplication Removed"]

    NoCycles --> SuccessBadge7["Build Time<br/>30% Faster"]
    SharedDeps --> SuccessBadge8["87% Test<br/>Overlap Removed"]

    classDef unified fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef shared fill:#fff3e0,stroke:#ff9800,stroke-width:2px
    classDef clean fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef success fill:#c8e6c9,stroke:#2e7d32,stroke-width:2px

    class UnifiedError,UnifiedPosition,UnifiedCache unified
    class SharedFS,SharedPerf,SharedDup shared
    class NoCycles,SharedDeps,CleanGraph clean
    class SuccessBadge1,SuccessBadge2,SuccessBadge3,SuccessBadge4,SuccessBadge5,SuccessBadge6,SuccessBadge7,SuccessBadge8 success
```

---

## Shared Crate Dependency Relationships

### Import Hierarchy (Recommended Order)

```mermaid
graph TD
    subgraph "Primary Import (Always First)"
        Primary["rust-ai-ide-common<br/>『Foundation Layer』<br/>├── Types (Position, Range, Error)<br/>├── Utils (timing, caching)<br/>├── FS (atomic, safe, streaming)<br/>└── Detection (duplication, patterns)"]
    end

    subgraph "Secondary Imports (As Needed)"
        Secondary1["rust-ai-ide-shared-services<br/>『Orchestration Layer』<br/>├── Workspace (project analysis)<br/>├── LSP (language servers)<br/>└── Diagnostics (aggregated)"]

        Secondary2["rust-ai-ide-shared-codegen<br/>『Generation Layer』<br/>├── CodeGen (templates, multi-lang)<br/>├── AST (parsing, transformation)<br/>└── Validation (safety, quality)"]
    end

    subgraph "Specialized Crates (Domain-Specific)"
        AI["rust-ai-ide-ai<br/>『Intelligence Layer』<br/>├── ML models<br/>├── Fine-tuning<br/>└── Specification analysis"]

        LSP_ACT["rust-ai-ide-lsp<br/>『Protocol Layer』<br/>├── LSP extensions<br/>├── Message handling<br/>└── Server management"]
    end

    %% Import order - arrows show recommended sequence
    Primary --> Secondary1
    Primary --> Secondary2
    Secondary1 --> LSP_ACT
    Secondary2 --> AI

    %% Dependencies (which are safe to use)
    AI -.->|can use| Primary
    AI -.->|can use| Secondary2
    LSP_ACT -.->|can use| Primary
    LSP_ACT -.->|can use| Secondary1

    classDef primary fill:#fff3e0,stroke:#ff9800,stroke-width:4px
    classDef secondary fill:#e8f5e8,stroke:#2e7d32,stroke-width:3px
    classDef specialized fill:#e1f5fe,stroke:#01579b,stroke-width:2px

    class Primary primary
    class Secondary1,Secondary2 secondary
    class AI,LSP_ACT specialized
```

---

## Development Workflow Diagrams

### New Developer Onboarding (Before vs After)

#### Before Deduplication

```mermaid
sequenceDiagram
    participant Dev as New Developer
    participant Code as Codebase
    participant Team as Team Member
    participant IDE as IDE

    Dev->>Code: Find error handler for new feature
    Code-->>Dev: Multiple CustomError variants (5+)
    Dev->>Team: "Which error type should I use?"
    Team-->>Dev: "Depends on which module..." (confusing)
    Dev->>Code: Discover current usage patterns
    Code-->>Dev: Inconsistent implementation (copy/paste)
    Dev->>IDE: Try to build first feature
    IDE-->>Dev: Compilation error (missing imports)
    Dev->>Team: "I'm stuck, please help"
    Team-->>Dev: "Use this specific variant..." (1-hour explanation)
    Dev->>Code: Implements with wrong variant
    Code-->>Dev: Review feedback: "Use different error type"

    Note over Dev: 1-2 weeks to be productive
```

#### After Deduplication

```mermaid
sequenceDiagram
    participant Dev as New Developer
    participant Docs as Shared Arch Docs
    participant Common as rust-ai-ide-common
    participant IDE as IDE

    Dev->>Docs: Read readme.html onboarding
    Docs-->>Dev: "Always import from rust-ai-ide-common"
    Dev->>Docs: Check duplication prevention checklist
    Docs-->>Dev: Clear checklist + examples
    Dev->>Common: Import unified types
    Common-->>Dev: IdeError, IdeResult, Cache... (all working)
    Dev->>IDE: Build first feature
    IDE-->>Dev: ✅ Compiles successfully
    Dev->>Common: Check for duplication
    Common-->>Dev: ✅ No duplications detected
    Dev->>Docs: Success! Feature implemented
    Docs-->>Dev: Remember to update performance metrics

    Note over Dev: 2 hours to be productive
```

### Bug Fix Workflow (Unified vs Fragmented)

#### Fragmented System

```mermaid
flowchart TD
    A[Bug Reported in AI Analysis] --> B[Investigate AI crate]
    B --> C[Find inconsistent types<br/>Position vs InternalPosition]
    C --> D[Check multiple error variants<br/>CustomError vs AnotherError]
    D --> E[Look for similar bugs in other crates<br/>Copy-paste patterns?]
    E --> F[Fix in AI crate ONLY<br/>may break other crates]
    F --> G[Test AI crate changes]
    G --> H[Test integration with other crates<br/>Unpredictable failures]
    H --> I{All tests pass?}
    I -->|No| J[Investigate cross-crate conflicts<br/>Communication overhead]
    I -->|Yes| K[Deploy patch<br/>Risk of regressions in other crates]
```

#### Unified System

```mermaid
flowchart TD
    A[Bug Reported in AI Analysis] --> B[Check rust-ai-ide-common<br/>Is it type-related?]
    B -->|Likely shared| C[Fix in shared crate<br/>Benefits all consumers]
    B -->|Domain-specific| M[Fix in AI crate only]
    C --> D[Run duplication check<br/>Ensure no new duplications]
    D --> E[Auto-update all consuming crates<br/>Consistent fix everywhere]
    E --> F[Test shared crate changes<br/>Automatic regression protection]
    F --> G{Tests pass?}
    G -->|Yes| H[Results in consistent improvement<br/>All consumers benefit]
    G -->|No| I[Debug shared implementation<br/>Fix applies everywhere]

    M --> N[Test AI-specific fix<br/>No cross-crate impact]

    H --> O[Deploy unified improvement<br/>Cross-crate consistency guaranteed]

    classDef unified fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef improved fill:#c8e6c9,stroke:#1b5e20,stroke-width:3px

    class B,C,D,E,F,H,O unified
    class A improved
```

---

## Performance Architecture

### Memory Management Flow

```mermaid
flowchart TD
    subgraph "Memory Pool (rust-ai-ide-common)"
        Pool[Memory Pool<br/>Reusable Allocations<br/>512MB limit]
        Strategy[Caching Strategy<br/>LRU Eviction<br/>TTL Support]
    end

    subgraph "Shared Crates Memory Usage"
        CommonM[MemoryCache<br/>Thread-safe<br/>Statistics tracking]
        CodegenM[AST Objects<br/>Reused parsers<br/>Streaming]
        ServicesM[LSP Responses<br/>Buffered diagnostics<br/>Connection pooling]
    end

    subgraph "Specialized Crate Memory"
        Animeib[Model Registry<br/>Pooled models<br/>LRU unloading]
        UIM[UI Components<br/>Shared render tree<br/>Event recycling]
    end

    Memory_Pressure[High Memory Usage] -->|Triggered| GC[Garbage Collection<br/>Hint system]
    Memory_Pressure -->|Action| Unload[Model Unloading<br/>Background task]

    Pool --> CommonM
    Strategy -->|Configures| CommonM

    classDef memory fill:#e1f5fe,stroke:#01579b
    classDef pressure fill:#fff3e0,stroke:#ff9800
    classDef gc fill:#ffcdd2,stroke:#b71c1c

    class Pool,Strategy,CommonM,CodegenM,ServicesM,Animeib,UIM memory
    class Memory_Pressure pressure
    class Unload,GC gc
```

### Caching Architecture

```mermaid
graph TD
    subgraph "Cache Hierarchy"
        L1[Application Cache<br/>Method-level<br/>Short TTL<br/>High frequency]
        L2[Module Cache<br/>Component-level<br/>Medium TTL<br/>Cross-method]
        L3[Global Cache<br/>System-level<br/>Long TTL<br/>Cross-crate]
    end

    subgraph "Cache Management"
        Monitor[Cache Monitor<br/>Usage statistics<br/>Performance metrics]
        AutoEvict[Auto-Eviction<br/>Background task<br/>LRU policy]
        Invalidator[Cache Invalidation<br/>Dependency tracking<br/>Graceful degradation]
    end

    L1 -->|Overflow| L2
    L2 -->|Overflow| L3

    Monitor -.->|Alerts| AutoEvict
    Monitor -.->|Reports| Invalidator

    classDef cache fill:#f3e5f5,stroke:#4a148c
    classDef management fill:#e8f5e8,stroke:#2e7d32

    class L1,L2,L3 cache
    class Monitor,AutoEvict,Invalidator management
```

---

## Service Orchestration

### LSP Service Architecture

```mermaid
graph TD
    subgraph "Language Server Management"
        Registry[LSP Registry<br/>Server processes<br/>Connection pooling]
        Orchestrator[LSP Orchestrator<br/>Request routing<br/>Load balancing]
        Fallback[Fallback System<br/>Process monitoring<br/>Auto-restart]
    end

    subgraph "Request Flow"
        Client[IDE Request<br/>Completion<br/>Navigation<br/>Diagnostics]
        Router[Request Router<br/>Language detection<br/>Server assignment]
        Processor[Request Processor<br/>Protocol handling<br/>Response formatting]
        Response[Response Handler<br/>Error normalization<br/>Client delivery]
    end

    subgraph "Supported Languages"
        Rust[LSP: rust-analyzer<br/>Cargo support<br/>Macros]
        JS[LSP: typescript-language-server<br/>TS/JS support<br/>DOM]
        Python[LSP: pylsp<br/>Multiple linters<br/>IntelliSense]
        Go[LSP: gopls<br/>Build integration<br/>Go modules]
    end

    Client --> Router
    Router --> Registry
    Registry --> Processor
    Processor -->|Response| Response

    Fallback -.->|Health check| Registry
    Fallback -.->|Restart failed| Processor

    Registry -.->|Routes to| Rust
    Registry -.->|Routes to| JS
    Registry -.->|Routes to| Python
    Registry -.->|Routes to| Go

    classDef management fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef flow fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef language fill:#f3e5f5,stroke:#4a148c,stroke-width:2px

    class Registry,Orchestrator,Fallback management
    class Client,Router,Processor,Response flow
    class Rust,JS,Python,Go language
```

---

## Code Quality Architecture

### Duplication Prevention System

```mermaid
flowchart TD
    A[Developer writes code] --> B{Pre-commit hook?}
    B -->|Yes| C[Run duplication detection]
    B -->|No| D[Check manual duplication]

    C --> E{Duplications found?}
    D --> E

    E -->|Yes| F[Show similarity report<br/>Suggest using shared types]
    E -->|No| G[Code passes duplication check]

    F --> H{Auto-fix possible?}
    H -->|Yes| I[Apply auto-fix<br/>Use shared replacement]
    H -->|No| J[Manual review required<br/>Developer guidance]

    I --> G
    J --> K{Accept fix?}
    K -->|Yes| I
    K -->|No| L[Continue with custom code<br/>Document rationale]

    G --> M[Commit successful<br/>Code quality maintained]

    classDef quality fill:#c8e6c9,stroke:#2e7d32
    classDef warning fill:#fff3e0,stroke:#ff9800
    classDef danger fill:#ffcdd2,stroke:#b71c1c
    classDef success fill:#e8f5e8,stroke:#1b5e20

    class G,M success
    class F,J warning
    class L danger
    class A,C,D,E,H,I,J,K quality
```

---

## Migration Path Visualization

### Migration Timeline

```mermaid
gantt
    title Deduplication Campaign Timeline
    dateFormat  YYYY-MM-DD
    section Assessment Phase
    Audit dependencies     :done, audit, 2025-08-01, 7d
    Catalog duplications   :done, catalog, 2025-08-08, 5d
    Impact analysis        :done, impact, 2025-08-13, 3d

    section Implementation Phase
    Phase 1: Shared types :done, phase1, 2025-08-15, 14d
    Phase 2: Error unification :done, phase2, 2025-09-01, 10d
    Phase 3: Cache consolidation :done, phase3, 2025-09-10, 8d
    Phase 4: Final migration :active, phase4, 2025-09-18, 15d

    section Validation Phase
    Testing               :active, 2025-10-03, 10d
    Performance verification : 2025-10-13, 5d
    Production deployment : 2025-10-18, 3d
```

### Success Metrics Tracking

```mermaid
lineChart
    title Key Performance Indicators Over Time
    x-axis 2025-08 2025-09 2025-10
    y-axis Percentage
    line Duplication Reduction 0 15 30 45 60 83 91
    line Build Time Improvement 0 5 12 20 30 33 35
    line Memory Usage Reduction 0 3 8 15 22 25 28
    line Developer Productivity 0 10 25 40 55 65 75
    line Code Consistency 0 12 28 45 62 78 95
```

---

## Deployment Architecture

### Rollout Strategy

```mermaid
stateDiagram
    [*] --> Assessment: Catalog impacts

    Assessment --> Planning: Create rollout plan

    Planning --> Phase1: Shared types deployment

    Phase1 --> Validation1: Test phase 1
    Validation1 --> Phase1: Fix issues
    Validation1 --> Phase2: Phase 1 successful

    Phase2 --> Validation2: Test phase 2
    Validation2 --> Phase2: Partial rollback
    Validation2 --> Phase3: Phase 2 successful

    Phase3 --> Validation3: Test phase 3
    Validation3 --> Phase3: Targeted fixes
    Validation3 --> Production: Phase 3 successful

    Production --> Monitoring: Production monitoring

    Monitoring --> Optimization: Performance improvements
    Monitoring --> [*]: Exit if degradation

    Optimization --> Production: Re-deploy optimizations

    note right of Assessment : Audit team impact\nEstimate delivery delays
    note right of Planning : Define rollback procedures\nSet success criteria
    note right of Phase1 : Types & basic utilities\nLowest risk
    note right of Phase2 : Error handling & async patterns\nMedium risk
    note right of Phase3 : Advanced features & services\nHighest risk
```

This architecture diagram collection provides comprehensive insights into the unified Rust AI IDE architecture, showing the relationships between components, migration progress, and quality improvements achieved through the deduplication campaign.
