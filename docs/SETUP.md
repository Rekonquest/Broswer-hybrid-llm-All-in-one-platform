# Setup Guide - Hybrid LLM Platform

This guide will help you set up the Hybrid LLM Platform on your system.

## Prerequisites

### Required
- **Rust** 1.75 or later
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **PostgreSQL** 14+ with pgvector extension
  ```bash
  # Ubuntu/Debian
  sudo apt install postgresql postgresql-contrib

  # macOS
  brew install postgresql

  # Arch Linux
  sudo pacman -S postgresql
  ```

###Optional (for specific features)
- **llama.cpp** models in GGUF format (for local LLMs)
- **Firecracker** (for full sandbox support - Linux only)
- API keys for cloud LLMs (Claude, OpenAI, Gemini)

## Installation Steps

### 1. Clone the Repository

```bash
git clone https://github.com/Rekonquest/browser-privacy.git
cd browser-privacy
```

### 2. Install pgvector Extension

```bash
# Clone and build pgvector
git clone --branch v0.5.0 https://github.com/pgvector/pgvector.git
cd pgvector
make
sudo make install

# Or use package manager (if available)
# Ubuntu 22.04+
sudo apt install postgresql-15-pgvector
```

### 3. Set Up PostgreSQL Database

```bash
# Start PostgreSQL
sudo systemctl start postgresql  # systemd
# OR
brew services start postgresql   # macOS

# Run the setup script
./scripts/setup_db.sh
```

The script will:
- Create database `hybrid_llm`
- Create user `hybrid_llm_user`
- Install pgvector extension
- Run initial schema migrations
- Generate `.env` file with connection details

**⚠️ Important**: Change the default password in `.env` before production use!

### 4. Configure Environment Variables

Copy the example environment file and edit:

```bash
cp .env.example .env
```

Edit `.env` and add your API keys:

```bash
# Cloud LLM API Keys (optional)
ANTHROPIC_API_KEY=sk-ant-your-key-here
OPENAI_API_KEY=sk-your-key-here
GOOGLE_API_KEY=your-key-here

# Database (already set by setup script)
DATABASE_URL=postgresql://hybrid_llm_user:your-password@localhost:5432/hybrid_llm
```

### 5. Configure the Platform

Copy and customize the configuration file:

```bash
cp config.example.toml config.toml
```

Edit `config.toml` to:
- Set up local model paths
- Configure permissions
- Adjust resource limits
- Enable/disable features

### 6. Download Local Models (Optional)

If you want to use local LLMs:

```bash
# Create models directory
mkdir -p models

# Download models from Hugging Face
# Example: Qwen 2.5 Coder 8B
wget https://huggingface.co/Qwen/Qwen2.5-Coder-8B-Instruct-GGUF/resolve/main/qwen2.5-coder-8b-instruct-q4_k_m.gguf \
  -O models/qwen2.5-coder-8b-q4_k_m.gguf

# Example: Qwen 2.5 4B Generalist
wget https://huggingface.co/Qwen/Qwen2.5-4B-Instruct-GGUF/resolve/main/qwen2.5-4b-instruct-q4_k_m.gguf \
  -O models/qwen2.5-4b-q4_k_m.gguf
```

Update `config.toml` with the correct paths:

```toml
[[local_models.models]]
id = "qwen-coder-8b"
path = "./models/qwen2.5-coder-8b-q4_k_m.gguf"
capabilities = ["code"]
context_size = 8192
```

### 7. Build the Platform

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release
```

### 8. Run the Orchestrator

```bash
# Development
cargo run

# Or use the release binary
./target/release/hybrid-llm
```

## Verification

### Test Database Connection

```bash
# Connect to database
psql postgresql://hybrid_llm_user:your-password@localhost:5432/hybrid_llm

# Verify tables
\dt

# Should show:
# - conversations
# - messages
# - documents
# - document_chunks
# - llm_contexts
# - global_context
# - audit_log
# - lockdown_events

# Exit
\q
```

### Test Model Loading (if using local models)

```rust
// TODO: Add example test code
```

## Troubleshooting

### PostgreSQL Connection Issues

**Error**: `connection refused`

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Check connection settings
sudo -u postgres psql -c "SELECT version();"
```

**Error**: `role does not exist`

```bash
# Re-run setup script
./scripts/setup_db.sh
```

### pgvector Extension Issues

**Error**: `extension "vector" is not available`

```bash
# Reinstall pgvector
cd pgvector
sudo make install

# Enable in database
sudo -u postgres psql hybrid_llm -c "CREATE EXTENSION vector;"
```

### Build Errors

**Error**: `llama-cpp-2` compilation fails

llama.cpp bindings require a C++ compiler:

```bash
# Ubuntu/Debian
sudo apt install build-essential

# macOS
xcode-select --install
```

### Permission Denied Errors

```bash
# Ensure PostgreSQL user has proper permissions
sudo -u postgres psql
```

```sql
ALTER USER hybrid_llm_user WITH SUPERUSER;
GRANT ALL PRIVILEGES ON DATABASE hybrid_llm TO hybrid_llm_user;
```

## Directory Structure

After setup, your directory should look like:

```
browser-privacy/
├── config.toml          # Your configuration
├── .env                 # Environment variables (don't commit!)
├── models/              # Local GGUF models
│   ├── qwen2.5-coder-8b-q4_k_m.gguf
│   └── qwen2.5-4b-q4_k_m.gguf
├── data/                # Runtime data (created automatically)
│   ├── downloads/
│   ├── uploads/
│   ├── rag/
│   └── sandboxes/
├── target/              # Rust build artifacts
└── [source code...]
```

## Next Steps

- **Read** [ARCHITECTURE.md](ARCHITECTURE.md) to understand the system design
- **Configure** permissions in `config.toml`
- **Test** with cloud LLMs first (easier to set up)
- **Download** local models for offline use
- **Explore** the audit log to see all actions

## Production Deployment

For production use:

1. **Change all default passwords**
2. **Use environment variables** for sensitive data (not config files)
3. **Enable SSL/TLS** for PostgreSQL connections
4. **Set up firewall** rules
5. **Configure backups** for the database
6. **Use systemd** or similar for process management
7. **Monitor** resource usage
8. **Review** security settings regularly

## Getting Help

- Check [GitHub Issues](https://github.com/Rekonquest/browser-privacy/issues)
- Read the [Architecture Documentation](ARCHITECTURE.md)
- Review example configurations in `config.example.toml`

## Quick Start (TL;DR)

```bash
# Install dependencies
sudo apt install postgresql postgresql-15-pgvector build-essential

# Clone and setup
git clone https://github.com/Rekonquest/browser-privacy.git
cd browser-privacy
./scripts/setup_db.sh
cp .env.example .env
cp config.example.toml config.toml

# Add your API keys to .env
nano .env

# Build and run
cargo build --release
./target/release/hybrid-llm
```

That's it! 🚀
