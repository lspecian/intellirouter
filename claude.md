# IntelliRouter

## Overview

IntelliRouter is a **programmable LLM gateway** written in Rust that provides an OpenAI-compatible API endpoint for chat completions. It routes and orchestrates LLM requests across multiple backends (OpenAI, Anthropic, Google Gemini, Mistral, xAI, Azure OpenAI, Ollama).

## Quick Start

```bash
# Build
cargo build --release

# Run all services
./target/release/intellirouter run

# Run specific role
./target/release/intellirouter run --role router

# Available roles: router, orchestrator, rag-injector, summarizer, all
```

## Architecture

### Core Modules (`src/modules/`)

| Module | Purpose |
|--------|---------|
| `llm_proxy` | OpenAI-compatible API endpoint (`/v1/chat/completions`) with streaming |
| `router_core` | Request routing (round-robin, content-based, priority strategies) |
| `model_registry` | LLM backend management, health checking, connectors |
| `chain_engine` | Multi-step workflow orchestration (sequential, parallel, conditional) |
| `persona_layer` | System prompt and guardrail injection |
| `rag_manager` | Retrieval Augmented Generation with vector stores |
| `memory` | Conversation memory (in-memory, Redis backends) |
| `ipc` | gRPC-based inter-service communication |
| `authz` | API key validation and RBAC |
| `telemetry` | Metrics, cost tracking, logging |
| `plugin_sdk` | Extension system for custom components |
| `test_harness` | Testing infrastructure |

### Key Files

- `src/main.rs` - Entry point with CLI
- `src/cli.rs` - Command-line parsing
- `src/config.rs` - Configuration management
- `build.rs` - Proto compilation
- `proto/*.proto` - gRPC service definitions

## Configuration

Config files in `config/`:
- `development.toml` - Dev environment
- `production.toml` - Production
- `testing.toml` - Test environment

Environment variables (`.env.example`):
```
ANTHROPIC_API_KEY=sk-ant-api03-...
OPENAI_API_KEY=sk-proj-...
GOOGLE_API_KEY=...
MISTRAL_API_KEY=...
XAI_API_KEY=...
AZURE_OPENAI_API_KEY=...
```

## Development

### Prerequisites

- Rust 1.87+ (see `rust-toolchain.toml`)
- `protoc` (Protocol Buffers compiler) - `apt install protobuf-compiler`
- Redis (optional, for redis backend)
- Docker (for containerized deployment)

### Building

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo check --all-targets      # Fast check without codegen
```

### Testing

```bash
cargo test                     # Run unit tests
cargo test --test e2e_tests    # E2E tests (requires test-utils feature)
./scripts/run_benchmarks.sh    # Performance benchmarks
```

### Code Quality

```bash
cargo clippy                   # Linting
cargo fmt                      # Formatting
./scripts/analyze_warnings.sh  # Warning analysis
```

## Docker

```bash
docker-compose up -d                    # Production
docker-compose -f docker-compose.dev.yml up -d   # Development
docker-compose -f docker-compose.test.yml up -d  # Testing
```

## Project Structure

```
src/
  modules/           # Core functionality
  generated/         # Proto-generated code
  bin/               # Additional binaries
tests/
  unit/              # Unit tests (mirrors src/)
  integration/       # Integration tests
  e2e/               # End-to-end tests
proto/               # gRPC proto definitions
sdk/                 # Python, TypeScript, Rust SDKs
deployment/          # K8s configs (MicroK8s, EKS, GKE)
scripts/             # Automation scripts
docs/                # Documentation
```

## Current Issues & TODOs

### Build Requirements

- Requires Rust 1.87+ due to transitive dependencies
- `protoc` must be installed for gRPC code generation
- Note: `time` crate pinned to 0.3.36 for compatibility

### Known Incomplete Features

1. **JWT IPC Authentication** (`src/modules/ipc/security.rs`) - Placeholder, not functional
2. **Some gRPC Servers** (`src/modules/ipc/memory/`) - Need implementation
3. **Plugin Downcasting** (`src/modules/plugin_sdk/traits.rs`) - TODO items
4. **Test Harness CI** (`src/modules/test_harness/ci/`) - Mock implementations
5. **Authorization Routes** (`src/modules/authz/routes.rs`) - Disabled due to dependency issues

### Warnings

Currently ~94 warnings, mostly:
- Unused imports (~50)
- Deprecated clap attributes (`#[clap(...)]` -> `#[arg(...)]`)
- Unused variables in test code

## API Endpoints

```
POST /v1/chat/completions    # OpenAI-compatible chat
GET  /health                 # Health check
GET  /metrics                # Prometheus metrics
```

## Feature Flags

In `Cargo.toml`:
- `memory-backend` (default) - In-memory storage
- `redis-backend` - Redis storage
- `file-backend` - File-based storage
- `test-harness` - Testing utilities
- `pdf-export` - PDF report generation

## Project Status & Roadmap

**See [ROADMAP.md](ROADMAP.md) for detailed progress tracking.**

Current state (as of 2025-02-21):
- **Phase 0** (Foundation) - In progress
- Tests don't compile (68+ errors)
- Core library compiles with ~94 warnings
- Server starts but LLM request forwarding not implemented

Priority items:
1. Fix test compilation
2. Implement LLM proxy request sending
3. Get Ollama connector working end-to-end

## Notes for AI Assistants

- This is a Rust 2021 edition project using async/await (Tokio runtime)
- Uses Axum for HTTP, Tonic for gRPC
- Heavy use of `Arc<RwLock<>>` for shared state
- Test-first development approach documented in `TESTING.md`
- Proto files define IPC contracts between services
- The project has extensive documentation in `docs/`
- **Check ROADMAP.md** for current progress and next tasks
