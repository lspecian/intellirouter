# IntelliRouter Production Roadmap

> **Last Updated:** 2025-02-21
> **Current Phase:** 0 - Foundation
> **Overall Progress:** 5%

---

## Quick Status

```
Phase 0: Foundation          [##--------] 20%   <- CURRENT
Phase 1: Core MVP            [----------]  0%
Phase 2: Integration         [----------]  0%
Phase 3: Production Ready    [----------]  0%
Phase 4: Advanced Features   [----------]  0%
```

---

## Current State Assessment

### What Works
- [x] Project compiles (lib only, not tests)
- [x] Basic CLI structure with role selection
- [x] Configuration loading (TOML)
- [x] Logging/tracing setup
- [x] Proto/gRPC code generation
- [x] Graceful shutdown coordinator

### What's Broken
- [ ] Tests don't compile (68+ errors)
- [ ] `test_utils` module not properly exported
- [ ] Many types not publicly exported
- [ ] hyper/mockito API incompatibilities
- [ ] JWT authentication is placeholder
- [ ] LLM proxy request sending not implemented

### Code Metrics
| Module | Lines | TODOs | Status |
|--------|-------|-------|--------|
| llm_proxy | 3954 | 1 | Partial - routing works, request sending TODO |
| model_registry | 2420 | 0 | Mostly complete - needs connector testing |
| router_core | 3272 | 0 | Structure complete - needs integration |
| chain_engine | 1270 | 0 | Basic structure - executors incomplete |
| persona_layer | 786 | 0 | Basic injection works |
| rag_manager | 842 | 0 | Scaffold only |
| memory | 725 | 0 | In-memory works, Redis untested |
| ipc | 3994 | 1 | gRPC clients exist, servers incomplete |
| authz | 825 | 1 | Routes disabled, API key basic |
| telemetry | 567 | 0 | Logging works, metrics partial |
| plugin_sdk | 1077 | 3 | Traits defined, loading incomplete |
| test_harness | 2000+ | 7 | Mostly stubs |

---

## Phase 0: Foundation (Current)

> **Goal:** Get tests compiling and establish baseline functionality

### 0.1 Fix Test Compilation
- [ ] **0.1.1** Export missing types from lib.rs
  - [ ] `MessageRole`, `ServiceType`, `ChatMessage`
  - [ ] `ConnectorConfig`, `ChatCompletionRequest`
  - [ ] `OllamaConnector`, `OpenAIConnector`
  - [ ] `ConnectorError`, `AuditError`
- [ ] **0.1.2** Fix module visibility
  - [ ] Export `test_harness` module (feature-gated)
  - [ ] Export `ipc::events` module
  - [ ] Fix `test_utils` exports
- [ ] **0.1.3** Fix API incompatibilities
  - [ ] Update `hyper::body::to_bytes` calls (hyper 1.0 API)
  - [ ] Update `mockito::server_url` calls (mockito 1.x API)
  - [ ] Fix `AppState` struct initializers
  - [ ] Fix `RouterConfig` struct initializers
- [ ] **0.1.4** Fix borrow checker errors
  - [ ] `response.usage.unwrap()` clone issues
  - [ ] `rx1`/`rx2` mutability in error_handling.rs
- [ ] **0.1.5** Run `cargo test` successfully

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
Blockers: _______________
```

### 0.2 Verify Basic Server Startup
- [ ] **0.2.1** Server starts without panic
- [ ] **0.2.2** Health endpoint responds
- [ ] **0.2.3** `/v1/chat/completions` returns proper error (no backend)
- [ ] **0.2.4** Graceful shutdown works

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
Blockers: _______________
```

### 0.3 Clean Up Warnings
- [ ] **0.3.1** Fix unused imports (~50)
- [ ] **0.3.2** Fix deprecated clap attributes
- [ ] **0.3.3** Fix unused variables
- [ ] **0.3.4** Target: <20 warnings

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
Warnings before: 94
Warnings after: ___
```

---

## Phase 1: Core MVP

> **Goal:** Single working LLM proxy with one backend (Ollama)

### 1.1 Model Registry - Ollama Connector
- [ ] **1.1.1** Implement `OllamaConnector::send_request`
- [ ] **1.1.2** Health check for Ollama endpoint
- [ ] **1.1.3** Model listing from Ollama
- [ ] **1.1.4** Unit tests passing
- [ ] **1.1.5** Integration test with real Ollama

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
Tested with Ollama version: ___
```

### 1.2 LLM Proxy - Request Flow
- [ ] **1.2.1** Implement request sending in `llm_proxy/mod.rs:52`
- [ ] **1.2.2** Non-streaming response handling
- [ ] **1.2.3** Streaming (SSE) response handling
- [ ] **1.2.4** Error response formatting (OpenAI-compatible)
- [ ] **1.2.5** Request validation
- [ ] **1.2.6** End-to-end test: curl -> proxy -> Ollama -> response

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
Tested models: _______________
```

### 1.3 Router Core - Basic Routing
- [ ] **1.3.1** Single model routing (passthrough)
- [ ] **1.3.2** Round-robin strategy working
- [ ] **1.3.3** Fallback on failure
- [ ] **1.3.4** Retry logic

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
Strategies tested: _______________
```

### 1.4 Basic Authentication
- [ ] **1.4.1** API key validation middleware
- [ ] **1.4.2** Reject requests without valid key
- [ ] **1.4.3** Config-based API keys

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
```

---

## Phase 2: Integration

> **Goal:** Multiple backends, persona injection, basic memory

### 2.1 Additional Connectors
- [ ] **2.1.1** OpenAI connector working
- [ ] **2.1.2** Anthropic connector working
- [ ] **2.1.3** Connector selection based on model name

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
```

### 2.2 Persona Layer
- [ ] **2.2.1** System prompt injection
- [ ] **2.2.2** Persona profiles from config
- [ ] **2.2.3** Per-request persona override
- [ ] **2.2.4** Guardrails (basic content filtering)

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
```

### 2.3 Memory - Conversation History
- [ ] **2.3.1** In-memory conversation tracking
- [ ] **2.3.2** Session ID support
- [ ] **2.3.3** Context window management (truncation)
- [ ] **2.3.4** Redis backend working

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
```

### 2.4 Telemetry
- [ ] **2.4.1** Request/response logging
- [ ] **2.4.2** Token usage tracking
- [ ] **2.4.3** Latency metrics
- [ ] **2.4.4** Prometheus endpoint working

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
```

---

## Phase 3: Production Ready

> **Goal:** Deployable, observable, secure

### 3.1 Docker Deployment
- [ ] **3.1.1** Dockerfile builds successfully
- [ ] **3.1.2** docker-compose.yml works
- [ ] **3.1.3** Environment variable configuration
- [ ] **3.1.4** Health checks in compose

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
Image size: ___ MB
```

### 3.2 Security Hardening
- [ ] **3.2.1** JWT authentication (fix ipc/security.rs)
- [ ] **3.2.2** RBAC working
- [ ] **3.2.3** Rate limiting
- [ ] **3.2.4** Input sanitization audit

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
Security review: ____-__-__
```

### 3.3 IPC/Multi-Service
- [ ] **3.3.1** gRPC server implementations
- [ ] **3.3.2** Service discovery
- [ ] **3.3.3** Circuit breaker working
- [ ] **3.3.4** Multi-role deployment tested

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
```

### 3.4 Documentation & Examples
- [ ] **3.4.1** Updated README with actual usage
- [ ] **3.4.2** Working curl examples
- [ ] **3.4.3** Python SDK tested
- [ ] **3.4.4** TypeScript SDK tested

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
```

---

## Phase 4: Advanced Features

> **Goal:** Full PRD capabilities

### 4.1 Chain Engine
- [ ] **4.1.1** Sequential chain execution
- [ ] **4.1.2** Parallel execution
- [ ] **4.1.3** Conditional branching
- [ ] **4.1.4** Loop support
- [ ] **4.1.5** Chain definition from YAML

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
```

### 4.2 RAG Manager
- [ ] **4.2.1** ChromaDB integration
- [ ] **4.2.2** Document ingestion
- [ ] **4.2.3** Context retrieval
- [ ] **4.2.4** Context injection into prompts

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
```

### 4.3 Plugin SDK
- [ ] **4.3.1** Plugin loading working
- [ ] **4.3.2** Custom routing strategy example
- [ ] **4.3.3** Custom connector example
- [ ] **4.3.4** Documentation

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
```

### 4.4 Kubernetes Deployment
- [ ] **4.4.1** Helm chart working
- [ ] **4.4.2** Horizontal scaling
- [ ] **4.4.3** Service mesh integration
- [ ] **4.4.4** Production load testing

**Tracking:**
```
Started: ____-__-__
Completed: ____-__-__
Target RPS: ___
Achieved RPS: ___
```

---

## Deleted/Deferred Items

Items removed from scope to reduce complexity:

| Item | Reason | Original Location |
|------|--------|-------------------|
| test_harness module | Over-engineered, use standard testing | src/modules/test_harness |
| 13 GitHub workflows | Too many, consolidate to 3-4 | .github/workflows |
| Dashboard module | Not MVP | src/modules/monitoring/dashboard.rs |
| PDF export | Not MVP | wkhtmltopdf feature |

---

## Session Log

Track progress across sessions:

### Session 1 - 2025-02-21
- [x] Initial assessment
- [x] Fixed rust-toolchain.toml (1.82 -> 1.87)
- [x] Fixed build.rs (protoc flag)
- [x] Pinned time crate to 0.3.36
- [x] Created ROADMAP.md
- [x] Created claude.md
- [ ] Next: Start Phase 0.1 (fix test compilation)

### Session 2 - ____-__-__
- [ ] ...

### Session 3 - ____-__-__
- [ ] ...

---

## Commands Reference

```bash
# Check compilation
cargo check --all-targets

# Run tests
cargo test

# Count warnings
cargo check 2>&1 | grep -c "warning:"

# Run with specific role
cargo run -- run --role router

# Build release
cargo build --release

# Docker build
docker build -t intellirouter .
```

---

## Priority Order

If time is limited, do these in order:

1. **Phase 0.1** - Tests must compile
2. **Phase 1.2** - LLM proxy request flow (the core feature)
3. **Phase 1.1** - Ollama connector (easiest to test locally)
4. **Phase 0.2** - Verify server startup
5. **Phase 1.4** - Basic auth (security baseline)

Everything else can come after you have a working proxy.
