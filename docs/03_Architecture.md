# Architecture

## High-Level Architecture

RuntimeShield follows a layered architecture with clear separation of concerns.

```mermaid
graph TB
    subgraph "Application Layer"
        APP[Application Code]
    end
    
    subgraph "RuntimeShield API"
        API[RuntimeShield Public API]
        BLD[Builder]
        EVT[Event Callbacks]
    end
    
    subgraph "Core Services"
        MON[Runtime Monitor]
        POL[Policy Engine]
        CFG[Configuration]
    end
    
    subgraph "Integrity Verification"
        BIN[Binary Integrity]
        LIB[Library Integrity]
        MEM[Memory Integrity]
        AD[Anti-Debug]
    end
    
    subgraph "Cryptography"
        HASH[Hash Functions - SHA256]
        MERKLE[Merkle Tree]
    end
    
    subgraph "Platform Abstraction"
        PLAT[Platform Traits]
        LNX[Linux Implementation]
        MAC[macOS Implementation]
        WIN[Windows Stub]
    end
    
    APP --> API
    API --> BLD
    API --> EVT
    API --> MON
    API --> POL
    API --> CFG
    
    MON --> BIN
    MON --> LIB
    MON --> MEM
    MON --> AD
    MON --> EVT
    MON --> POL
    
    BIN --> HASH
    BIN --> MERKLE
    
    BIN --> PLAT
    LIB --> PLAT
    MEM --> PLAT
    AD --> PLAT
    
    PLAT --> LNX
    PLAT --> MAC
    PLAT --> WIN
```

## Module Dependencies

```mermaid
graph LR
    subgraph "External Crates"
        SERDE[serde]
        SHA2[sha2]
        TOML[toml]
        LOG[log]
    end
    
    subgraph "RuntimeShield"
        CORE[core]
        CRYPTO[crypto]
        CONFIG[config]
        INTEG[integrity]
        PLAT[platform]
        MON[monitor]
        POL[policy]
        EVTS[events]
        API[api]
        UTILS[utils]
    end
    
    API --> CORE
    API --> CONFIG
    API --> EVTS
    API --> POL
    API --> INTEG
    API --> MON
    
    CORE --> CRYPTO
    CORE --> CONFIG
    
    INTEG --> CRYPTO
    INTEG --> PLAT
    
    MON --> INTEG
    MON --> EVTS
    MON --> POL
    
    POL --> CONFIG
    POL --> EVTS
    
    CRYPTO --> SHA2
    CONFIG --> SERDE
    CONFIG --> TOML
    EVTS --> SERDE
    POL --> LOG
```

## Builder Pattern

```mermaid
sequenceDiagram
    participant App as Application
    participant Builder as RuntimeShieldBuilder
    participant Shield as RuntimeShield
    participant Monitor as RuntimeMonitor
    
    App->>Builder: new()
    App->>Builder: enable_startup_verification()
    App->>Builder: enable_runtime_monitor()
    App->>Builder: enable_binary_integrity()
    App->>Builder: on_event(callback)
    App->>Builder: policy("policy.toml")
    App->>Builder: build()
    
    Builder->>Shield: new(builder, policy)
    Builder-->>App: RuntimeShield
    
    App->>Shield: start()
    
    Shield->>Shield: startup_verification()
    Shield->>Shield: initialize_integrity_modules()
    
    alt runtime_monitor enabled
        Shield->>Monitor: start()
        Monitor-->>Shield: background thread running
    end
    
    Shield-->>App: Ok(())
```

## Threading Model

```mermaid
graph LR
    subgraph "Main Application Thread"
        APP[Application]
        API[RuntimeShield API]
    end
    
    subgraph "Background Verification Thread"
        MON[RuntimeMonitor]
        VERIFY{Verification Loop}
        EVTDISP[Event Dispatch]
    end
    
    APP -->|start()| API
    API -->|spawns| MON
    MON -->|every interval| VERIFY
    VERIFY -->|events| EVTDISP
    EVTDISP -->|callbacks| APP
    
    style MON fill:#f9f,stroke:#333,stroke-width:2px
```

The threading model is intentionally simple:

- **Main thread**: Application code and RuntimeShield API calls
- **Background thread**: Runtime verification loop (if enabled)
- **Synchronization**: Events are dispatched via `Arc<dyn Fn(Event) + Send + Sync>` callbacks. The policy engine is stateless for thread safety.

## Data Flow

```mermaid
flowchart LR
    A[Builder Configuration] --> B[RuntimeShield Instance]
    B --> C{start() called?}
    C -->|Yes| D[Startup Verification]
    C -->|No| E[Idle]
    D --> F[Initialize Modules]
    F --> G{Background Monitor?}
    G -->|Yes| H[Spawn Verification Thread]
    G -->|No| I[Ready for On-Demand]
    H --> J[Verification Loop]
    J --> K{Integrity OK?}
    K -->|Yes| J
    K -->|No| L[Policy Engine]
    L --> M{Action}
    M -->|Terminate| N[exit process]
    M -->|Callback| O[dispatch event]
    M -->|Log| P[log warning]
    M -->|Ignore| Q[do nothing]
```

## Key Design Decisions

1. **No async runtime** — Uses standard threads instead of tokio to minimize dependencies and complexity.

2. **Stateless policy engine** — The policy engine has no internal state, making it safe to share across threads.

3. **Event-based communication** — Verification results are communicated through events and callbacks, not through return values or shared state.

4. **Builder pattern** — All configuration flows through the builder, making the API discoverable and preventing invalid states.

5. **Trait-based platform abstraction** — Platform-specific code is behind traits, allowing clean separation and testability.

6. **No global state** — RuntimeShield instances are independent. Multiple instances can exist in the same process.
