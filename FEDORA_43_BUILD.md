# Fedora 43 (2026) Build Guide

This document provides specific instructions for building the Hybrid LLM Platform on **Fedora 43** with updated system libraries.

## System Requirements - Fedora 43

Fedora 43 has updated to newer library versions that are incompatible with older Tauri builds:

- **WebKitGTK**: Uses `webkit2gtk-4.1` (4.0 is deprecated)
- **JavaScriptCore**: Uses `javascriptcoregtk-4.1`
- **libsoup**: Uses `libsoup-3.0` (2.4 is deprecated)

## What Changed for Fedora 43 Compatibility

### 1. Tauri Version Update

**Old (Tauri v1.x)**:
- Used webkit2gtk-4.0
- Used libsoup-2.4
- Had allowlist-based security model

**New (Tauri v2.x)**:
- ✅ Uses webkit2gtk-4.1 (Fedora 43 compatible)
- ✅ Uses libsoup-3.0 (Fedora 43 compatible)
- ✅ Uses new capability-based security model
- ✅ Better IPC serialization

### 2. Serialization Fixes

All Tauri IPC communication requires proper serialization:

**Error Types**: `HybridLLMError` now implements custom `Serialize`:
```rust
impl Serialize for HybridLLMError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
```

**Response Types**: All command response structs have `#[derive(Serialize)]`:
- `SystemState`
- `Document`
- `AuditLogEntry`
- `SendMessageResponse`
- `CreateSandboxResponse`
- `ExecuteInSandboxResponse`
- `SandboxFile`

### 3. Configuration Updates

**tauri.conf.json** migrated from v1 to v2 format:
- Removed deprecated `allowlist` section
- New `app` section with `security` configuration
- Updated `assetProtocol` for file access
- Simplified structure

## Installation on Fedora 43

### Step 1: Install System Dependencies

```bash
# Install required libraries for Tauri v2
sudo dnf install \
    webkit2gtk4.1-devel \
    gtk3-devel \
    libsoup3-devel \
    openssl-devel \
    curl \
    wget \
    file-devel \
    librsvg2-devel \
    libappindicator-gtk3-devel

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Node.js 18+ (if not already installed)
sudo dnf install nodejs npm
```

### Step 2: Verify Library Versions

```bash
# Check WebKitGTK version (should be 4.1.x)
pkg-config --modversion webkit2gtk-4.1

# Check libsoup version (should be 3.x)
pkg-config --modversion libsoup-3.0

# Check GTK version
pkg-config --modversion gtk+-3.0
```

Expected output:
```
webkit2gtk-4.1: 2.42.x or higher
libsoup-3.0: 3.4.x or higher
gtk+-3.0: 3.24.x or higher
```

### Step 3: Clone and Build

```bash
# Clone the repository
git clone https://github.com/Rekonquest/browser-privacy.git
cd browser-privacy

# Build Rust backend (verify it compiles)
cargo build --release

# Install UI dependencies
cd ui
npm install

# Run in development mode
npm run tauri dev

# Or build for production
npm run tauri build
```

## Troubleshooting Common Issues

### Issue 1: "libsoup-2.4 not found"

**Error:**
```
Package libsoup-2.4 was not found in the pkg-config search path
```

**Solution:**
This means you're still using Tauri v1.x. Update to v2.x:
```bash
cd src-tauri
# Verify Cargo.toml has:
# tauri = { version = "2.0", features = ["protocol-asset"] }
```

### Issue 2: "webkit2gtk-4.0-dev not found"

**Error:**
```
Package webkit2gtk-4.0 was not found
```

**Solution:**
Fedora 43 only has webkit2gtk-4.1. Install the correct version:
```bash
sudo dnf install webkit2gtk4.1-devel
```

### Issue 3: Serialization Errors

**Error:**
```
the trait `Serialize` is not implemented for `HybridLLMError`
```

**Solution:**
The custom `Serialize` implementation is already in `crates/common/src/errors.rs`. Make sure you've pulled the latest changes:
```bash
git pull origin main
cargo clean
cargo build
```

### Issue 4: Tauri Config Schema Error

**Error:**
```
Error parsing tauri.conf.json: unknown field 'allowlist'
```

**Solution:**
You're using a v1 config with v2 Tauri. The config has been updated to v2 format. Make sure `src-tauri/tauri.conf.json` has:
```json
{
  "$schema": "https://schema.tauri.app/config/2.0",
  ...
}
```

## Performance Notes

On Fedora 43, the application should have:
- ✅ Faster cold start (WebKit 4.1 optimizations)
- ✅ Better memory efficiency (libsoup 3.0 improvements)
- ✅ Improved security (Tauri v2 capabilities)

## Testing the Build

```bash
# Run all Rust tests
cargo test --all

# Check for unused dependencies
cargo +nightly udeps

# Lint the codebase
cargo clippy --all-targets --all-features

# Run the app
cd ui && npm run tauri dev
```

## Key Files Modified for Fedora 43

1. **src-tauri/Cargo.toml**
   - Updated `tauri` to v2.0
   - Updated `tauri-build` to v2.0
   - Added `tauri-plugin-shell`

2. **crates/common/src/errors.rs**
   - Added custom `Serialize` implementation for `HybridLLMError`

3. **src-tauri/tauri.conf.json**
   - Migrated from v1 to v2 config format
   - Updated security model
   - Removed deprecated `allowlist`

4. **src-tauri/src/state.rs**
   - All response structs have `Serialize` derives

5. **src-tauri/src/commands.rs**
   - All command response types properly serialized

## Comparison: Tauri v1 vs v2

| Feature | Tauri v1 (Old) | Tauri v2 (New - Fedora 43) |
|---------|----------------|---------------------------|
| WebKitGTK | 4.0 (deprecated) | 4.1 ✅ |
| libsoup | 2.4 (deprecated) | 3.0 ✅ |
| Security | Allowlist | Capabilities ✅ |
| IPC | Basic | Enhanced ✅ |
| Build Time | Slower | Faster ✅ |
| Bundle Size | Larger | Smaller ✅ |

## Next Steps

After successful build:

1. **Set up PostgreSQL** (see `docs/SETUP.md`)
2. **Configure LLM providers** (see `config.example.toml`)
3. **Test the UI** - Load the app and verify:
   - WebSocket connection indicator (top-right)
   - Document drag-and-drop works
   - Permission controls function
   - Coding canvas appears

4. **Optional: Set up Firecracker** for production sandboxing

## Support

If you encounter issues:

1. Check library versions with `pkg-config`
2. Verify Rust is updated: `rustc --version` (should be 1.75+)
3. Check Node.js: `node --version` (should be 18+)
4. Review build logs in `target/` directory
5. Open an issue on GitHub with full error output

## References

- [Tauri v2 Migration Guide](https://v2.tauri.app/start/migrate/from-tauri-1/)
- [Fedora Developer Portal](https://developer.fedoraproject.org/)
- [WebKitGTK 4.1 Documentation](https://webkitgtk.org/)
- [libsoup 3.0 Documentation](https://libsoup.org/)
