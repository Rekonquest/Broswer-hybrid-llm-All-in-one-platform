# Architecture Documentation

## System Overview

The Hybrid LLM Platform is built as a **modular microkernel architecture** where the orchestrator acts as a central message bus, coordinating between independent, specialized components.

## Design Principles

1. **Compartmentalization**: Each component is isolated and communicates via well-defined interfaces
2. **Security-First**: Multi-layered security with fail-safe defaults
3. **Extensibility**: New LLM providers and features can be added without modifying core
4. **Async-First**: Built on Tokio for efficient concurrent operations
5. **Type-Safety**: Rust's type system prevents entire classes of bugs

## Core Components

### 1. Orchestrator (Kernel)

**Location**: `orchestrator/src/orchestrator.rs`

**Responsibilities**:
- Central message routing
- System state management
- Component lifecycle coordination
- Lockdown enforcement

**Key Data Structures**:
```rust
pub struct Orchestrator {
    message_bus: Arc<MessageBus>,
    router: Arc<RwLock<Router>>,
    lockdown_state: Arc<RwLock<LockdownState>>,
}
```

**Message Flow**:
1. Component publishes message to bus
2. Orchestrator receives and validates
3. Routes to appropriate handler
4. Handler processes and may publish new messages

### 2. Message Bus

**Location**: `orchestrator/src/message_bus.rs`

**Technology**: Tokio `broadcast` channels

**Pattern**: Publish-Subscribe

**Benefits**:
- Decouples components
- Multiple subscribers per message type
- Non-blocking async communication
- Built-in backpressure handling

**Message Types**:
- `UserRequest`: Initial user input
- `LLMDelegation`: Inter-LLM communication
- `PermissionRequest`: Permission checks
- `SecurityAlert`: Security violations
- `StateChange`: System state updates

### 3. Router

**Location**: `orchestrator/src/router.rs`

**Purpose**: Intelligently route tasks to appropriate LLMs

**Routing Strategy**:
```rust
1. Parse task requirements (capabilities needed)
2. Query capability index for matching LLMs
3. Filter by availability (is_loaded = true)
4. Prefer local models over cloud
5. Load balance if multiple candidates
6. Return selected LLM ID
```

**Capability Index**:
```
{
  Code: ["qwen-coder-8b", "claude-sonnet"],
  Security: ["redreamer-3b", "claude-sonnet"],
  General: ["qwen-4b", "gpt-4"],
}
```

### 4. LLM Pool Manager

**Location**: `crates/llm-pool/`

**Components**:
- `LLMPool`: Registry and lifecycle management
- `LoadBalancer`: Distribution algorithms

**Features**:
- Dynamic loading/unloading of models
- Capability-based indexing
- Health monitoring
- Resource tracking

**Provider Interface**:
```rust
#[async_trait]
pub trait LLMProvider {
    fn capabilities(&self) -> Vec<Capability>;
    fn instance(&self) -> &LLMInstance;
    async fn complete(&self, prompt: &str, context: Context) -> Result<String>;
    async fn health_check(&self) -> Result<bool>;
    async fn load(&mut self) -> Result<()>;
    async fn unload(&mut self) -> Result<()>;
}
```

### 5. Security Engine

**Location**: `crates/security-engine/`

**Three-Layer Security Model**:

#### Layer 1: Permission Manager
- Global default permissions
- Per-LLM overrides
- Runtime permission requests
- Failed request tracking

#### Layer 2: Guardrails
- Pattern matching (regex-based)
- Semantic analysis (future: small security LLM)
- Resource monitoring
- Risk level classification

**Guardrail Rules**:
```rust
GuardrailRule {
    name: "dangerous_rm",
    pattern: r"rm\s+(-rf?|--recursive).*(/|\*)",
    risk_level: RiskLevel::Critical,
}
```

#### Layer 3: Lockdown Controller
- Automatic triggers
- Read-only mode
- Audit trail
- Authentication-gated release

**Lockdown Triggers**:
- Policy violations
- Suspicious patterns
- Resource exhaustion
- Multiple failed permissions (>5)
- User panic button

### 6. API Gateway

**Location**: `crates/api-gateway/`

**Unified LLM Interface**: All cloud providers implement the same `LLMProvider` trait

**Adapters**:
- `ClaudeAdapter`: Anthropic API
- `OpenAIAdapter`: OpenAI ChatGPT API
- `GeminiAdapter`: Google Gemini API

**Benefits**:
- Orchestrator doesn't care if LLM is local or cloud
- Easy to add new providers
- Credential management centralized
- Consistent error handling

**Future**: Streaming support with `async fn complete_stream()`

### 7. Context Manager

**Location**: `crates/context-manager/`

**Two-Tier Context Model**:

#### Global Context (Shared)
- System state
- Cross-LLM alerts
- Conversation history (if persistent)

#### Per-LLM Context (Private)
- Current task
- Working memory
- Specialized knowledge

**Storage**:
- Current: In-memory (DashMap)
- Future: PostgreSQL for persistence
- RAG: pgvector for semantic search

**RAG Search Flow**:
```
1. User uploads document
2. Split into chunks
3. Generate embeddings (sentence-transformers)
4. Store in pgvector
5. Query with semantic similarity
6. Return top-k relevant chunks
```

### 8. Sandbox Manager

**Location**: `crates/sandbox-manager/`

**Technology**: Firecracker microVMs

**Isolation Levels**:
- Separate kernel
- Isolated filesystem
- Network policy enforcement
- Resource quotas

**Lifecycle**:
```
Create → Configure → Execute → Review → Approve/Reject → Destroy
```

**Future Features**:
- Snapshot/restore
- Pre-configured templates
- Network access gating
- Volume mounting

### 9. Filesystem Interface

**Location**: `crates/filesystem-interface/`

**Directory Structure**:
```
/platform-root/
├── downloads/    # LLM outputs to user
├── uploads/      # User inputs to LLM
│   └── .versions/  # Git-like versioning
└── rag/          # Processed RAG data
    ├── embeddings/
    └── metadata/
```

**File Watcher** (future):
- Monitors `uploads/` for new files
- Triggers automatic indexing
- Updates RAG database

## Data Flow Examples

### Example 1: Simple User Request

```
User: "What's the weather?"
  ↓
Orchestrator receives UserRequest message
  ↓
Router classifies: TaskType::General
  ↓
Router selects: qwen-generalist-4b
  ↓
LLM Pool forwards to provider
  ↓
Provider calls LLM
  ↓
Response sent back through message bus
  ↓
User receives answer
```

### Example 2: Inter-LLM Collaboration

```
User: "Write a secure login API"
  ↓
Router → Generalist LLM
  ↓
Generalist analyzes task
  ↓
Generalist publishes LLMDelegation message
  Target: Coder LLM
  ↓
Orchestrator routes to Coder LLM
  ↓
Coder writes implementation
  ↓
Coder publishes LLMDelegation message
  Target: Security LLM
  ↓
Security LLM reviews code
  ↓
Security LLM finds issue
  ↓
Security LLM responds to Coder
  ↓
Coder revises code
  ↓
Coder responds to Generalist
  ↓
Generalist formats for user
  ↓
User receives secure code
```

### Example 3: Permission Request Flow

```
LLM wants to write file
  ↓
LLM publishes PermissionRequest
  Type: FileWrite("/downloads/output.txt")
  Explanation: "Saving analysis results"
  ↓
Security Engine receives request
  ↓
Permission Manager checks:
  - Is path in whitelist? ✓
  - Is lockdown active? ✗
  ↓
Permission granted
  ↓
Audit Logger records decision
  ↓
LLM proceeds with file write
```

### Example 4: Lockdown Trigger

```
LLM attempts: "rm -rf /"
  ↓
Command goes to Security Engine
  ↓
Guardrails analyze command
  ↓
Pattern match: "dangerous_rm" ✓
  Risk Level: Critical
  ↓
Security Engine publishes SecurityAlert
  SuggestedAction: Lockdown
  ↓
Orchestrator receives alert
  ↓
Lockdown state → Locked
  ↓
All subsequent LLM requests rejected
  ↓
UI notifies user
  ↓
User reviews audit log
  ↓
User authenticates
  ↓
Lockdown released
```

## Concurrency Model

### Async Runtime: Tokio

**Benefits**:
- Lightweight tasks (green threads)
- Efficient I/O multiplexing
- Work-stealing scheduler
- Cancellation support

### Synchronization Primitives

- `Arc`: Shared ownership of immutable data
- `RwLock`: Reader-writer locks for shared mutable state
- `DashMap`: Concurrent HashMap
- `broadcast::channel`: Multi-producer, multi-consumer
- `mpsc::channel`: Single-consumer queues

### Thread Safety

All shared state is either:
1. Immutable (via `Arc`)
2. Protected by locks (`RwLock`)
3. Lock-free concurrent structures (`DashMap`)

## Error Handling

### Error Types

```rust
pub enum HybridLLMError {
    LLMError(String),
    PermissionDenied(String),
    SecurityViolation(String),
    LockdownActive(String),
    // ... more specific errors
}
```

### Error Propagation

- Use `Result<T>` for all fallible operations
- `?` operator for early returns
- Contextual error messages
- Never panic in library code

## Future Optimizations

### Performance
- Connection pooling for cloud APIs
- Request batching
- Response caching
- Lazy loading of models

### Scalability
- Distributed orchestrator (multiple instances)
- Load balancing across machines
- Kubernetes deployment
- Horizontal scaling

### Features
- WebAssembly sandboxes (lighter than Firecracker)
- Plugin system for extensions
- Model fine-tuning integration
- A/B testing between models

## Testing Strategy

### Unit Tests
- Each component tested in isolation
- Mock dependencies
- Property-based testing for security rules

### Integration Tests
- End-to-end message flows
- Multi-LLM scenarios
- Permission edge cases

### Security Tests
- Fuzzing guardrails
- Penetration testing
- Audit log verification

## Deployment

### Single-User (Current)
```
cargo build --release
./target/release/hybrid-llm
```

### Future: Docker
```dockerfile
FROM rust:1.75 as builder
# ... build steps
FROM debian:bookworm-slim
# ... runtime
```

### Future: Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hybrid-llm
spec:
  replicas: 3
  # ... orchestrator pods
```

## Monitoring & Observability

### Logging (Tracing)
- Structured logs via `tracing` crate
- Log levels: debug, info, warn, error
- Correlation IDs for request tracking

### Metrics (Future)
- Request latency
- Model load times
- Permission grant/deny rates
- Resource utilization

### Tracing (Future)
- Distributed tracing with OpenTelemetry
- Request flow visualization
- Performance profiling

---

**Last Updated**: Initial MVP (January 2026)
