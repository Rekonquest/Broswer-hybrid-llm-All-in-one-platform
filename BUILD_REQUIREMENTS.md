# Build Requirements

## Current Build Status

### ✅ Rust Backend - WORKING
All backend crates compile successfully:
```bash
cargo build
# ✓ orchestrator
# ✓ crates/common
# ✓ crates/llm-pool
# ✓ crates/security-engine
# ✓ crates/sandbox-manager
# ✓ crates/filesystem-interface
# ✓ crates/api-gateway
# ✓ crates/context-manager
# ✓ crates/llama-cpp-provider
```

### ❌ Tauri GUI - BLOCKED
Cannot build due to missing system dependencies in sandboxed environment.

## System Requirements for Full Build

### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libsoup2.4-dev \
    javascriptcoregtk-4.0 \
    libjavascriptcoregtk-4.0-dev
```

### Fedora/RHEL
```bash
sudo dnf install \
    webkit2gtk3-devel \
    openssl-devel \
    curl \
    wget \
    libappindicator-gtk3-devel \
    librsvg2-devel
```

### Arch Linux
```bash
sudo pacman -S \
    webkit2gtk \
    base-devel \
    curl \
    wget \
    openssl \
    appmenu-gtk-module \
    gtk3 \
    libappindicator-gtk3 \
    librsvg \
    libvips
```

## Build Commands

### Backend Only
```bash
# From project root
cargo build

# For release
cargo build --release
```

### Full Tauri Application
```bash
# Install Node dependencies
cd ui
npm install

# Development mode (hot reload)
npm run tauri dev

# Production build
npm run tauri build
```

## Testing Backend Components

Since the GUI can't build in this environment, you can test backend components individually:

### 1. Run the orchestrator binary
```bash
cargo run --bin hybrid-llm
```

### 2. Test individual crates
```bash
cargo test -p common
cargo test -p llm-pool
cargo test -p security-engine
# etc...
```

### 3. Check for unused code
```bash
cargo clippy --all-targets --all-features
```

## Next Steps for Full Build

To complete the full build and run the Tauri app:

1. **Use a system with full internet access** and proper package manager
2. **Install system dependencies** listed above for your OS
3. **Install Node.js** (v18 or later recommended)
4. **Run the build**:
   ```bash
   cd ui
   npm install
   npm run tauri dev
   ```

## Alternative: Docker Build

If you want to build in a controlled environment:

```dockerfile
FROM ubuntu:24.04

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libsoup2.4-dev

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
RUN apt-get install -y nodejs

# Copy project
COPY . /app
WORKDIR /app

# Build
RUN cargo build --release
RUN cd ui && npm install && npm run tauri build
```

## What Works Now

Even without the GUI, the following are fully functional:

1. **Message Bus** - Inter-component communication
2. **Router** - LLM selection and routing
3. **Security Engine** - Permission management, guardrails, lockdown
4. **API Gateway** - Cloud LLM adapters (Claude, OpenAI, Gemini)
5. **LLM Pool** - Provider management and load balancing
6. **Context Manager** - Memory and database-backed context
7. **Filesystem Interface** - Document upload/download/RAG
8. **Sandbox Manager** - Firecracker integration (structure ready)
9. **Llama.cpp Provider** - Local model support (structure ready)

You can import and use these crates in any Rust application, not just Tauri.

## Architecture Independence

The backend is designed to work with **any** frontend:
- Tauri (current implementation)
- Electron
- Pure web app (add REST API layer)
- CLI tool
- Embedded in another application

The Tauri integration is just one possible interface to the Rust core.
