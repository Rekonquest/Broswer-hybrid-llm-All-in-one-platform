# Complete Installation Guide for Fedora 43

This guide walks you through installing and running the Hybrid LLM Platform on Fedora 43.

## Prerequisites Check

Before starting, verify you're on Fedora 43:
```bash
cat /etc/fedora-release
# Should show: Fedora release 43 (Forty Three)
```

---

## Step 1: Pull Latest Updates from Git

Navigate to your project directory and pull the latest changes:

```bash
# Navigate to the project (adjust path if needed)
cd ~/browser-privacy  # or wherever you cloned it

# Make sure you're on the correct branch
git checkout claude/fedora43-tauriv2-6G25L

# Pull the latest changes
git pull origin claude/fedora43-tauriv2-6G25L

# Verify you have the latest commit
git log -1 --oneline
# Should show: bfe0943 refactor: Complete Fedora 43 + Tauri v2 architecture rebuild
```

**What this does**: Downloads all the Tauri v2 compatibility fixes, error handling improvements, and frontend dependency updates.

---

## Step 2: Install System Dependencies

This is the most important step. Fedora 43 requires the complete GTK3 development stack for Tauri.

### 2.1 Install GTK3 and WebKit Development Packages

```bash
sudo dnf install -y \
    webkit2gtk4.1-devel \
    gtk3-devel \
    atk-devel \
    pango-devel \
    gdk-pixbuf2-devel \
    cairo-devel \
    libsoup3-devel \
    openssl-devel \
    librsvg2-devel \
    libappindicator-gtk3-devel \
    file-devel
```

**What this does**: Installs all the system libraries that Tauri needs to build on Fedora 43.

### 2.2 Verify System Libraries

Check that the key libraries are installed correctly:

```bash
pkg-config --modversion webkit2gtk-4.1
# Expected: 2.42.x or higher

pkg-config --modversion libsoup-3.0
# Expected: 3.4.x or higher

pkg-config --modversion gtk+-3.0
# Expected: 3.24.x or higher
```

If any of these commands fail, the dnf install didn't complete correctly. Retry step 2.1.

---

## Step 3: Install Rust (if not already installed)

### 3.1 Check if Rust is Installed

```bash
rustc --version
cargo --version
```

If both commands work and show version 1.75 or higher, **skip to Step 4**.

### 3.2 Install Rust

If Rust is not installed or is too old:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts (usually just press Enter to accept defaults)

# Load Rust into your current shell
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

---

## Step 4: Install Node.js and npm (if not already installed)

### 4.1 Check if Node.js is Installed

```bash
node --version
npm --version
```

If both commands work and show Node.js 18.x or higher, **skip to Step 5**.

### 4.2 Install Node.js

```bash
sudo dnf install -y nodejs npm

# Verify installation
node --version  # Should be 18.x or higher
npm --version   # Should be 9.x or higher
```

---

## Step 5: Install Frontend Dependencies

Now install the Tauri v2 frontend packages:

```bash
# Make sure you're in the project root
cd ~/browser-privacy

# Install all npm dependencies
npm install

# This will install:
# - @tauri-apps/api ^2.0.0
# - @tauri-apps/plugin-shell ^2.0.0
# - @tauri-apps/plugin-dialog ^2.0.0
# - @tauri-apps/plugin-fs ^2.0.0
# - @tauri-apps/cli ^2.0.0
# - All React and UI dependencies
```

**Expected output**: Should complete without errors. You might see some warnings (these are normal).

---

## Step 6: Build the Rust Backend

Time to build the Tauri backend!

### 6.1 Clean Previous Builds (recommended)

```bash
cargo clean
```

### 6.2 Build the Backend

```bash
cd ~/browser-privacy
cargo build --release
```

**What to expect**:
- First build will take 5-10 minutes (downloads and compiles all dependencies)
- You'll see progress as it compiles all the crates
- Should complete with "Finished release [optimized] target(s) in X.XXm"

**Common issues**:
- If you see `atk.pc not found` or `pango.pc not found`: Go back to Step 2 and reinstall system dependencies
- If you see compiler errors: Make sure you're on the correct branch (Step 1)

---

## Step 7: Configure the Application (Optional)

### 7.1 Database Setup (PostgreSQL)

If you want to use the full features, you'll need PostgreSQL:

```bash
# Install PostgreSQL
sudo dnf install -y postgresql postgresql-server

# Initialize the database
sudo postgresql-setup --initdb

# Start PostgreSQL service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Create database and user
sudo -u postgres createuser -s $USER
createdb hybrid_llm
```

### 7.2 Configuration File

```bash
# Copy example config
cp config.example.toml config.toml

# Edit configuration (optional)
nano config.toml
```

Add your LLM API keys if you have them:
- OpenAI API key
- Anthropic API key
- Ollama endpoint (if using local models)

**Note**: The app will work without this, but you won't be able to use LLM features.

---

## Step 8: Run the Application

### 8.1 Development Mode (Recommended for First Run)

```bash
cd ~/browser-privacy
npm run tauri dev
```

**What happens**:
1. Vite dev server starts (React frontend)
2. Tauri compiles and launches the app window
3. You should see a window open with the Hybrid LLM Platform UI

**Expected behavior**:
- Window opens successfully
- No console errors
- UI loads with the main interface

### 8.2 Production Build (Optional)

For a production build:

```bash
npm run tauri build
```

The compiled app will be in: `src-tauri/target/release/bundle/`

---

## Step 9: Verify Everything Works

Once the app opens:

### 9.1 Check System Status
- Click on "System" or status indicator
- Should show connection status
- WebSocket should be connected (green indicator)

### 9.2 Test Basic Features
- Try uploading a document (drag and drop a text file)
- Check the permissions panel
- View the audit log

### 9.3 Check Console for Errors

In the terminal where you ran `npm run tauri dev`, check for:
- ✅ "Tauri v2 app initialized successfully"
- ✅ "WebSocket server started"
- ❌ No error messages

---

## Troubleshooting

### Build Fails with "atk.pc not found"

**Solution**:
```bash
sudo dnf install -y atk-devel pango-devel gdk-pixbuf2-devel cairo-devel
cargo clean
cargo build --release
```

### "Cannot find webkit2gtk-4.0"

**Solution**: Fedora 43 uses webkit2gtk-4.1, not 4.0. Make sure you installed `webkit2gtk4.1-devel` (Step 2.1).

### npm install fails

**Solution**:
```bash
# Clear npm cache
npm cache clean --force

# Delete node_modules and package-lock.json
rm -rf node_modules package-lock.json

# Reinstall
npm install
```

### App Window Doesn't Open

**Solution**:
```bash
# Check if Wayland is causing issues
# Try with X11 instead
GDK_BACKEND=x11 npm run tauri dev
```

### Port Already in Use

**Solution**:
```bash
# Kill any process using port 1420
sudo lsof -ti:1420 | xargs kill -9

# Try again
npm run tauri dev
```

---

## Performance Tips

Once everything is working:

1. **Enable Hardware Acceleration**:
   - Check in app settings or config.toml

2. **Use Production Build**:
   - Production builds are faster and smaller
   - Run `npm run tauri build` once you've verified dev mode works

3. **Monitor Resources**:
   - The app should use <500MB RAM in idle state
   - CPU usage should be low when not processing

---

## Next Steps

After successful installation:

1. **Add LLM Providers**:
   - Configure API keys in config.toml
   - Or set up Ollama for local models

2. **Import Documents**:
   - Test document upload and processing
   - Try the RAG (Retrieval Augmented Generation) features

3. **Explore Security Features**:
   - Test the sandboxed execution environment
   - Review permission scopes
   - Check audit logs

4. **Customize Settings**:
   - Adjust privacy levels
   - Configure data retention policies
   - Set resource limits

---

## Getting Help

If you run into issues:

1. **Check logs**:
   ```bash
   # Tauri logs
   journalctl --user -u tauri

   # Application logs
   tail -f ~/.local/share/hybrid-llm-platform/logs/app.log
   ```

2. **Enable debug logging**:
   ```bash
   RUST_LOG=debug npm run tauri dev
   ```

3. **Open an issue**: Include:
   - Full error output
   - Output of `pkg-config --modversion webkit2gtk-4.1`
   - Your Fedora version: `cat /etc/fedora-release`
   - Rust version: `rustc --version`

---

## Summary of Commands

Quick reference for the full installation:

```bash
# 1. Pull updates
cd ~/browser-privacy
git checkout claude/fedora43-tauriv2-6G25L
git pull origin claude/fedora43-tauriv2-6G25L

# 2. Install system dependencies
sudo dnf install -y webkit2gtk4.1-devel gtk3-devel atk-devel pango-devel \
    gdk-pixbuf2-devel cairo-devel libsoup3-devel openssl-devel \
    librsvg2-devel libappindicator-gtk3-devel file-devel

# 3. Verify Rust (install if needed)
rustc --version || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 4. Verify Node.js (install if needed)
node --version || sudo dnf install -y nodejs npm

# 5. Install frontend dependencies
npm install

# 6. Build backend
cargo clean
cargo build --release

# 7. Run the app
npm run tauri dev
```

---

**Installation complete!** 🎉

Your Hybrid LLM Platform should now be running on Fedora 43 with full Tauri v2 compatibility.
