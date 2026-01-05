# Hybrid LLM Platform

> A BrowserOS-inspired platform for running multiple LLMs with fine-grained security controls, inter-LLM collaboration, and sandboxed execution environments.

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

## 🚀 Overview

The Hybrid LLM Platform is a comprehensive orchestration system for running multiple Language Learning Models (both local and cloud-based) with enterprise-grade security, permission management, and collaborative capabilities. Think of it as an operating system for LLMs.

### Key Features

- **🤝 Multi-LLM Collaboration**: LLMs can call each other for specialized tasks
- **🔒 Fine-Grained Security**: Extensive permission system with algorithmic guardrails
- **🏖️ Sandboxed Execution**: Firecracker-based microVMs for isolated code execution
- **🌐 Hybrid Architecture**: Support for both local (llama.cpp) and cloud LLMs (Claude, GPT, Gemini)
- **📚 Built-in RAG**: PostgreSQL + pgvector for semantic document search
- **🎯 Developer-First**: Full root access by default with configurable safety controls
- **📊 Audit Trail**: Complete logging of all LLM actions and permission requests

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│         Rust Orchestrator (Kernel)      │
│  ┌────────────────────────────────────┐ │
│  │       Message Bus (tokio)          │ │
│  └────────────────────────────────────┘ │
│                                          │
│  ┌──────────┐  ┌──────────┐  ┌────────┐│
│  │ LLM Pool │  │ Security │  │Context ││
│  │ Manager  │  │  Engine  │  │Manager ││
│  └──────────┘  └──────────┘  └────────┘│
│                                          │
│  ┌──────────┐  ┌──────────┐  ┌────────┐│
│  │Filesystem│  │ Sandbox  │  │   API  ││
│  │Interface │  │ Manager  │  │Gateway ││
│  └──────────┘  └──────────┘  └────────┘│
└─────────────────────────────────────────┘
         │                    │
         ▼                    ▼
  ┌───────────┐        ┌─────────────┐
  │ Local LLMs│        │  Cloud APIs │
  │(llama.cpp)│        │Claude/GPT/  │
  │           │        │   Gemini    │
  └───────────┘        └─────────────┘
```

### Component Overview

| Component | Purpose | Technology |
|-----------|---------|------------|
| **Orchestrator** | Central message routing and coordination | Rust + Tokio |
| **LLM Pool** | Manages multiple LLM instances and load balancing | Rust |
| **Security Engine** | Permission management and guardrails | Rust + Regex |
| **Sandbox Manager** | Isolated code execution | Firecracker microVMs |
| **Context Manager** | Global and per-LLM context/memory | In-memory + PostgreSQL |
| **API Gateway** | Unified interface for cloud LLMs | Rust + reqwest |
| **Filesystem Interface** | Upload/download/RAG document management | Rust + notify |

## 🎯 Use Cases

- **Multi-Model Development**: Use specialized models for different tasks (coding, security, analysis)
- **Secure AI Assistants**: LLMs with controlled system access and audit trails
- **Research & Experimentation**: Test different models and collaboration patterns
- **Privacy-First AI**: Run local models while optionally leveraging cloud APIs
- **Educational**: Learn about LLM orchestration and security

## 🛠️ Installation

### Prerequisites

- **Rust**: 1.75 or later
- **PostgreSQL**: 14+ with pgvector extension
- **Firecracker** (optional for MVP, required for full sandbox support)
- **llama.cpp** (for local models)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/Rekonquest/browser-privacy.git
cd browser-privacy

# Build the project
cargo build --release

# Set up PostgreSQL with pgvector (see docs/setup.md)

# Run the orchestrator
./target/release/hybrid-llm
```

## ⚙️ Configuration

### Permission System

The platform uses a layered permission system:

1. **Global Defaults**: Apply to all LLMs by default
2. **Per-LLM Overrides**: Custom permissions for specific models
3. **Runtime Requests**: LLMs can request additional permissions with justification

Example permission configuration:

```yaml
file_system:
  read: ["/home/user/downloads/*", "/rag/*"]
  write: ["/home/user/downloads/*"]
  execute: ["/usr/bin/python", "/usr/bin/node"]

network:
  inbound: true
  outbound: true
  require_approval: ["*"]  # All network access requires user approval

commands:
  whitelist: ["git", "npm", "python", "cargo", "ls", "cat"]
  blacklist: ["rm -rf /", "sudo", "dd", "mkfs"]
  require_explanation: true

resources:
  max_cpu_percent: 80
  max_memory_gb: 8
  max_disk_gb: 50
```

### Security Guardrails

Built-in algorithmic guardrails automatically detect:

- ⚠️ Dangerous file deletions (`rm -rf /`)
- ⚠️ Privilege escalation (`sudo`)
- ⚠️ Low-level disk operations (`dd`, `mkfs`)
- ⚠️ Potential data exfiltration patterns
- ⚠️ Hardcoded credentials
- ⚠️ Shell injection attempts

### Lockdown Mode

The system can enter lockdown mode when:

- Policy violations are detected
- Suspicious patterns emerge
- Resource limits are exceeded
- User presses panic button
- Multiple failed permission requests (>5)

In lockdown mode:
- All LLM operations pause
- System reverts to read-only mode
- Incident is flagged for human review
- User authentication required to resume

## 🤖 Multi-LLM Collaboration

LLMs can collaborate to solve complex tasks:

```
User: "Build a secure REST API"
  ↓
Generalist LLM: Analyzes request
  ↓
Generalist → Coder LLM: "Write the API implementation"
  ↓
Coder → Security LLM: "Review this code for vulnerabilities"
  ↓
Security LLM: Provides feedback
  ↓
Coder: Revises implementation
  ↓
User: Receives final, security-reviewed code
```

## 📦 Sandboxed Execution

Code execution happens in isolated Firecracker microVMs:

1. LLM requests sandbox
2. System creates isolated microVM
3. Code executes in sandbox
4. LLM requests artifact transfer
5. User reviews and approves
6. Files move to main system
7. Sandbox destroyed

## 🔌 Supported LLM Providers

### Local Models (via llama.cpp)
- Qwen 2.5 Coder (8B, 14B, 32B)
- Qwen 2.5 (3B, 7B, 14B)
- RedReamer (3B - specialized for security)
- DeepSeek Coder
- Llama 3.1/3.2
- Any GGUF-format model

### Cloud APIs
- ✅ **Claude** (Anthropic) - Implemented
- ✅ **ChatGPT** (OpenAI) - Implemented
- ✅ **Gemini** (Google) - Implemented
- 🔜 Mistral API
- 🔜 Cohere API

## 📁 Project Structure

```
browser-privacy/
├── Cargo.toml              # Workspace configuration
├── orchestrator/           # Main binary
│   └── src/
│       ├── main.rs
│       ├── message_bus.rs
│       ├── router.rs
│       └── orchestrator.rs
├── crates/
│   ├── common/            # Shared types and traits
│   ├── llm-pool/          # LLM management
│   ├── security-engine/   # Permissions & guardrails
│   ├── context-manager/   # Memory & RAG
│   ├── api-gateway/       # Cloud LLM adapters
│   ├── filesystem-interface/
│   └── sandbox-manager/   # Firecracker integration
├── docs/                  # Documentation
└── scripts/               # Utility scripts
```

## 🚦 Current Status: MVP Foundation

### ✅ Completed
- [x] Core orchestrator with message bus
- [x] LLM pool management and routing
- [x] Security engine with guardrails
- [x] Permission system (file, network, command, resource)
- [x] API Gateway (Claude, OpenAI, Gemini)
- [x] Context manager foundation
- [x] Filesystem interface
- [x] Sandbox manager (placeholder for Firecracker)
- [x] Audit logging

### 🚧 In Progress
- [ ] PostgreSQL + pgvector integration for RAG
- [ ] llama.cpp integration
- [ ] Actual Firecracker microVM implementation
- [ ] Tauri UI development
- [ ] File watcher for auto-RAG indexing

### 🔮 Roadmap
- [ ] Streaming responses from all LLM providers
- [ ] Advanced load balancing and resource optimization
- [ ] Model performance metrics
- [ ] Plugin system for extensions
- [ ] Multi-user support
- [ ] Web-based UI dashboard
- [ ] Model marketplace/discovery

## 🤝 Contributing

This is a developer-first project. Contributions are welcome!

### Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/Rekonquest/browser-privacy.git
cd browser-privacy
cargo build

# Run tests
cargo test

# Check code
cargo clippy
```

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🙏 Acknowledgments

- **llama.cpp** for efficient local model inference
- **Firecracker** for secure microVM technology
- **Tokio** for async runtime
- **PostgreSQL & pgvector** for RAG capabilities

## 📞 Contact

- Issues: [GitHub Issues](https://github.com/Rekonquest/browser-privacy/issues)
- Discussions: [GitHub Discussions](https://github.com/Rekonquest/browser-privacy/discussions)

---

**Note**: This is an MVP foundation. Core components are functional but some features (Firecracker integration, UI, full RAG) are in development. See the roadmap above for details.
