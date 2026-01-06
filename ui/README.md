# UI Development

## Hybrid LLM Platform - Web Interface

Modern, drag-and-drop UI built with:
- **Tauri** - Rust-native desktop app framework
- **React** + **TypeScript** - Component-based UI
- **Tailwind CSS** - Utility-first styling
- **Vite** - Lightning-fast build tool

## Features

### 📤 Drag & Drop Document Upload
- Drop files anywhere to upload to RAG
- Real-time indexing status
- Supported formats: TXT, MD, PDF, DOCX, JSON, CSV

### 🤖 LLM Management Dashboard
- Visual status indicators for all LLMs
- One-click load/unload
- Real-time processing indicators
- Capability badges

### 🔒 Interactive Permission Controls
- Tabbed interface for all permission types
- Live updates (respects lockdown state)
- Visual feedback for changes

### 💻 Coding Canvas
- Syntax-highlighted editor
- Live preview
- Save to sandbox
- Download files
- Multi-language support

### 📊 Audit Log Viewer
- Real-time action logging
- Approval/denial indicators
- Filterable by LLM
- Timestamp tracking

### 🚨 Panic Button
- Prominent emergency shutdown
- Immediate lockdown trigger
- Visual lockdown status

## Setup

### Install Dependencies

```bash
npm install
```

### Development Mode

```bash
npm run dev
```

This starts the Vite dev server at `http://localhost:1420`

### Build for Production

```bash
npm run build
```

### Run Tauri App

```bash
npm run tauri dev
```

## Project Structure

```
ui/
├── src/
│   ├── components/      # Reusable UI components
│   │   ├── DocumentUpload.tsx
│   │   ├── LLMManager.tsx
│   │   ├── PermissionControl.tsx
│   │   ├── CodingCanvas.tsx
│   │   └── AuditLog.tsx
│   ├── pages/          # Page layouts
│   │   └── Dashboard.tsx
│   ├── types/          # TypeScript definitions
│   │   └── index.ts
│   ├── styles/         # Global styles
│   │   └── globals.css
│   ├── App.tsx         # Main app component
│   └── main.tsx        # Entry point
├── index.html
├── vite.config.ts
├── tailwind.config.js
└── tsconfig.json
```

## Key Components

### DocumentUpload
Drag-and-drop zone with visual feedback:
- Accepts multiple file types
- Shows upload progress
- Displays indexed document list

### LLMManager
Control panel for all LLMs:
- Status indicators (idle/processing/error)
- Load/unload buttons
- Capability tags
- Context size display

### PermissionControl
Comprehensive permission management:
- Filesystem paths (read/write/execute)
- Network settings (inbound/outbound/approval)
- Command whitelist/blacklist
- Resource limits (CPU/memory/disk)

### CodingCanvas
Split-pane code editor:
- Syntax highlighting for 6+ languages
- Live preview pane
- Save to sandbox
- Download locally
- Run in isolated environment

### AuditLog
Complete action history:
- Chronological log of all actions
- Approval/denial status
- LLM attribution
- Filtering and search

## Styling

Uses Tailwind CSS with custom theme:

```css
/* Dark mode by default */
bg-gray-950 - Background
bg-gray-900 - Cards
bg-gray-800 - Inputs

/* Primary color */
primary-500 - Main accent
primary-600 - Hover state

/* Status colors */
success-500 - Green (approved, active)
danger-500 - Red (denied, errors)
```

## TypeScript Types

All types are defined in `ui/src/types/index.ts`:

```typescript
LLMInstance
SystemState
Document
AuditLogEntry
PermissionScope
Sandbox
Message
```

## Future Enhancements

- [ ] WebSocket connection to Rust backend
- [ ] Real-time LLM response streaming
- [ ] Drag-and-drop LLM workflow builder
- [ ] Dark/light theme toggle
- [ ] Keyboard shortcuts
- [ ] Mobile responsive design
- [ ] Multi-language i18n

## Troubleshooting

### Port already in use

```bash
# Change port in vite.config.ts
server: {
  port: 3000,  // or any other port
}
```

### Tailwind styles not loading

```bash
# Rebuild Tailwind
npm run build
```

### TypeScript errors

```bash
# Check types
npx tsc --noEmit
```

## Screenshots

[Dashboard View]
- Split layout with LLM manager and document upload

[Coding Canvas]
- Side-by-side editor and preview

[Permissions]
- Tabbed interface with all security controls

[Audit Log]
- Chronological action list with status indicators
