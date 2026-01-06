# Hybrid LLM Platform - UI Integration Guide

This document explains how the React UI integrates with the Rust backend through Tauri.

## Architecture Overview

The UI follows a modern React architecture with TypeScript, connecting to the Rust backend through two channels:

1. **Tauri IPC (Request/Response)**: For standard CRUD operations and commands
2. **WebSocket (Real-time Updates)**: For live status updates and streaming data

```
┌─────────────────────────────────────────────────────┐
│                   React UI (TypeScript)              │
│  ┌──────────────┐  ┌───────────────┐               │
│  │ useTauriAPI  │  │ useWebSocket  │               │
│  │   Hook       │  │     Hook      │               │
│  └──────┬───────┘  └───────┬───────┘               │
│         │                   │                        │
│         │ invoke()          │ ws://127.0.0.1:3030   │
└─────────┼───────────────────┼────────────────────────┘
          │                   │
┌─────────┼───────────────────┼────────────────────────┐
│         ▼                   ▼                        │
│  ┌─────────────┐  ┌─────────────────┐              │
│  │ Commands    │  │ WebSocket       │              │
│  │ Handler     │  │ Server          │              │
│  └─────┬───────┘  └────────┬────────┘              │
│        │                    │                        │
│        ▼                    ▼                        │
│  ┌──────────────────────────────────┐              │
│  │       AppState (Shared)           │              │
│  │  - LLM Pool                       │              │
│  │  - Security Engine                │              │
│  │  - Permissions                    │              │
│  │  - Documents                      │              │
│  │  - Audit Log                      │              │
│  └──────────────────────────────────┘              │
│                Tauri Backend (Rust)                  │
└─────────────────────────────────────────────────────┘
```

## File Structure

### React Hooks

**`ui/src/hooks/useTauriAPI.ts`**
- Wraps all Tauri IPC commands
- Provides type-safe functions for calling backend
- Handles file uploads with Base64 encoding
- Returns: API object with all available commands

**`ui/src/hooks/useWebSocket.ts`**
- Manages WebSocket connection to backend
- Auto-reconnection with exponential backoff
- Routes messages to appropriate callbacks
- Provides convenience hooks for specific message types

### Type Definitions

**`ui/src/types/api.ts`**
- All request/response types for Tauri commands
- WebSocket message types
- Ensures type safety across IPC boundary

**`ui/src/types/index.ts`**
- Core domain types (LLMInstance, Document, etc.)
- Shared between UI and backend

### Components

**`ui/src/App.tsx`**
- Root component
- Initializes Tauri API and WebSocket
- Loads all initial data in parallel
- Manages global state (system state, LLMs, documents, audit log)
- Handles real-time updates via WebSocket callbacks

**`ui/src/pages/Dashboard.tsx`**
- Main UI container
- Receives API and state from App
- Implements handlers for all user actions
- Displays WebSocket connection status
- Routes to different views (Overview, Canvas, Permissions, Audit)

**`ui/src/components/CodingCanvas.tsx`**
- Integrated code editor with sandbox execution
- Creates dedicated sandbox on mount
- Executes code and displays output
- Saves/downloads code files

**`ui/src/components/DocumentUpload.tsx`**
- Drag-and-drop file upload
- Calls `api.uploadDocument()` for each file
- Triggers refresh to show new documents

**`ui/src/components/LLMManager.tsx`**
- Controls for loading/unloading LLMs
- Calls `api.loadLLM()` and `api.unloadLLM()`
- Displays LLM status

**`ui/src/components/PermissionControl.tsx`**
- Edits permission scopes
- Calls `api.updatePermissions()` on save
- Locked when system is in lockdown mode

## Data Flow Examples

### 1. Loading LLMs on Startup

```typescript
// App.tsx
const loadLLMs = async () => {
  const llmList = await api.getLLMs();
  setLlms(llmList);
  setSystemState(prev => ({
    ...prev,
    active_llms: llmList.filter(llm => llm.loaded).map(llm => llm.id),
  }));
};
```

Backend handler in `src-tauri/src/commands.rs:get_llms`:
```rust
#[tauri::command]
pub async fn get_llms(state: State<'_, AppState>) -> Result<Vec<LLMInstance>, String> {
    let pool = state.llm_pool.read().await;
    let llms: Vec<LLMInstance> = pool.get_all_ids()
        .iter()
        .filter_map(|id| pool.get(id))
        .map(|provider| provider.instance().clone())
        .collect();
    Ok(llms)
}
```

### 2. Uploading Documents

```typescript
// Dashboard.tsx
const handleDocumentUpload = async (files: File[]) => {
  for (const file of files) {
    await api.uploadDocument(file);  // Converts to Base64 internally
  }
  onRefresh();  // Reload documents list
};
```

Backend handler in `src-tauri/src/commands.rs:upload_document`:
```rust
#[tauri::command]
pub async fn upload_document(
    state: State<'_, AppState>,
    request: UploadDocumentRequest,
) -> Result<UploadDocumentResponse, String> {
    let document = Document {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        mime_type: request.mime_type,
        uploaded_at: chrono::Utc::now().to_rfc3339(),
    };

    state.documents.write().await.push(document.clone());

    // Broadcast via WebSocket
    // ... (WebSocket notification code)

    Ok(UploadDocumentResponse {
        id: document.id,
        name: document.name,
        uploaded_at: document.uploaded_at,
    })
}
```

### 3. Real-time Updates via WebSocket

```typescript
// App.tsx
const { isConnected } = useWebSocket({
  onLLMStatus: (message) => {
    console.log('LLM status update:', message);
    loadLLMs();  // Reload LLM list when status changes
  },
  onDocumentUploaded: (message) => {
    loadDocuments();  // Reload documents when new one is uploaded
  },
  onLockdownChanged: (message) => {
    loadSystemState();  // Update lockdown state
  },
});
```

Backend WebSocket in `src-tauri/src/websocket.rs`:
```rust
async fn handle_connection(ws_stream: WebSocketStream<TcpStream>, app: AppHandle) {
    let (mut write, mut read) = ws_stream.split();

    // Send periodic updates
    while let Some(msg) = read.next().await {
        // Handle incoming messages
    }

    // Broadcast events
    let message = WebSocketMessage {
        type_: "llm_status".to_string(),
        payload: serde_json::to_value(&llm_status).unwrap(),
    };
    write.send(Message::Text(serde_json::to_string(&message)?)).await?;
}
```

### 4. Sandbox Code Execution

```typescript
// CodingCanvas.tsx
const handleRun = async () => {
  const result = await api.executeInSandbox(sandboxId, code, language);
  setOutput(result.output + `\nCompleted in ${result.execution_time_ms}ms`);
};
```

Backend handler in `src-tauri/src/commands.rs:execute_in_sandbox`:
```rust
#[tauri::command]
pub async fn execute_in_sandbox(
    state: State<'_, AppState>,
    request: ExecuteInSandboxRequest,
) -> Result<ExecuteInSandboxResponse, String> {
    // Execute code in Firecracker VM
    // Return output and execution time
    Ok(ExecuteInSandboxResponse {
        sandbox_id: request.sandbox_id,
        output: "Code output here".to_string(),
        exit_code: 0,
        execution_time_ms: 125,
    })
}
```

## API Reference

### System Commands

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `getSystemState()` | - | `SystemState` | Current system lockdown state, active LLMs count |
| `triggerLockdown(reason)` | `reason: string` | `LockdownResponse` | Enters lockdown mode |
| `releaseLockdown(password)` | `password: string` | `LockdownResponse` | Exits lockdown mode |

### LLM Commands

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `getLLMs()` | - | `LLMInstance[]` | All registered LLMs |
| `loadLLM(llmId)` | `llmId: string` | `LoadLLMResponse` | Load LLM into memory |
| `unloadLLM(llmId)` | `llmId: string` | `UnloadLLMResponse` | Unload LLM from memory |
| `sendMessage(llmId, content, context?)` | `llmId, content, context` | `SendMessageResponse` | Send prompt to LLM |

### Document Commands

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `uploadDocument(file)` | `file: File` | `UploadDocumentResponse` | Upload document for RAG |
| `getDocuments()` | - | `Document[]` | All uploaded documents |
| `deleteDocument(documentId)` | `documentId: string` | `DeleteDocumentResponse` | Delete document |

### Permission Commands

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `getPermissions()` | - | `Permissions` | Current permission scope |
| `updatePermissions(permissions)` | `permissions: Permissions` | `UpdatePermissionsResponse` | Update permissions |

### Sandbox Commands

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `createSandbox(name, config?)` | `name, config` | `CreateSandboxResponse` | Create new sandbox |
| `executeInSandbox(sandboxId, code, language)` | `sandboxId, code, language` | `ExecuteInSandboxResponse` | Execute code |
| `getSandboxFiles(sandboxId, path?)` | `sandboxId, path` | `GetSandboxFilesResponse` | List sandbox files |
| `approveTransfer(sandboxId, filePath, destination)` | `sandboxId, filePath, destination` | `ApproveTransferResponse` | Approve file transfer |

### Audit Commands

| Function | Parameters | Returns | Description |
|----------|-----------|---------|-------------|
| `getAuditLog()` | - | `AuditLogEntry[]` | All audit log entries |

## WebSocket Message Types

| Type | Payload | Description |
|------|---------|-------------|
| `llm_status` | `LLMStatusMessage` | LLM loaded/unloaded/processing |
| `document_uploaded` | `DocumentUploadedMessage` | New document added |
| `lockdown_changed` | `LockdownChangedMessage` | Lockdown state changed |
| `sandbox_output` | `SandboxOutputMessage` | Real-time sandbox output |
| `audit_log` | `AuditLogEntry` | New audit log entry |

## Error Handling

All API calls return `Promise<T>` and throw errors on failure:

```typescript
try {
  await api.loadLLM('qwen-coder-8b');
} catch (err) {
  console.error('Failed to load LLM:', err);
  // Show error to user
}
```

Backend errors are serialized as strings:
```rust
#[tauri::command]
pub async fn some_command() -> Result<Response, String> {
    some_operation().map_err(|e| e.to_string())?;
    Ok(response)
}
```

## Building and Running

### Development
```bash
# Install dependencies
npm install

# Run in dev mode
npm run tauri dev
```

### Production
```bash
# Build for production
npm run tauri build
```

The Tauri build will:
1. Bundle the React app with Vite
2. Compile all Rust crates including `src-tauri`
3. Link Tauri backend with frontend
4. Create platform-specific executable

## Testing Integration

To test the full integration:

1. Start the app: `npm run tauri dev`
2. Check WebSocket connection indicator in header
3. Upload a document (drag & drop)
4. Load an LLM model
5. Try the coding canvas sandbox
6. Test the panic button (lockdown)
7. Check audit log for all actions

## Security Considerations

- **IPC**: All commands go through Tauri's security allowlist
- **WebSocket**: Runs on localhost only (127.0.0.1:3030)
- **File Upload**: Base64 encoding prevents path traversal
- **Sandbox**: Firecracker provides VM-level isolation
- **Lockdown**: All operations respect lockdown state

## Future Enhancements

1. **Streaming LLM Responses**: Use WebSocket for token streaming
2. **File Browser**: Direct sandbox filesystem access
3. **Multi-Sandbox**: Create multiple sandboxes simultaneously
4. **RAG Search**: Semantic search UI for documents
5. **LLM Chat UI**: Dedicated chat interface with history
6. **Permission Presets**: Save/load permission configurations
7. **Audit Export**: Download audit log as JSON/CSV
