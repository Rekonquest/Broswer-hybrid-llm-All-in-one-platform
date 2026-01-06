// LLM Types
export interface LLMInstance {
  id: string;
  provider: 'local' | 'claude' | 'openai' | 'gemini';
  model_name: string;
  capabilities: Capability[];
  is_loaded: boolean;
  max_context: number;
}

export type Capability = 'code' | 'security' | 'general' | 'analysis' | 'creative';

export interface LLMStatus {
  id: string;
  status: 'idle' | 'processing' | 'error';
  current_task?: string;
}

// Message Types
export interface Message {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  llm_id?: string;
}

// Document Types
export interface Document {
  id: string;
  filename: string;
  size: number;
  uploaded_at: string;
  indexed: boolean;
  chunk_count?: number;
}

// Permission Types
export interface PermissionScope {
  file_system: {
    read_paths: string[];
    write_paths: string[];
    execute_paths: string[];
  };
  network: {
    inbound: boolean;
    outbound: boolean;
    require_approval: boolean;
  };
  commands: {
    whitelist: string[];
    blacklist: string[];
    require_explanation: boolean;
  };
  resources: {
    max_cpu_percent: number;
    max_memory_gb: number;
    max_disk_gb: number;
  };
}

// Audit Log Types
export interface AuditLogEntry {
  id: string;
  timestamp: string;
  llm_id?: string;
  action: string;
  approved: boolean;
  reason?: string;
}

// System State
export interface SystemState {
  lockdown: 'normal' | 'readonly' | 'locked';
  active_llms: string[];
  pending_approvals: number;
}

// Sandbox Types
export interface Sandbox {
  id: string;
  llm_id: string;
  status: 'running' | 'stopped' | 'pending';
  files: SandboxFile[];
}

export interface SandboxFile {
  path: string;
  size: number;
  modified: string;
}

// File Transfer
export interface FileTransfer {
  id: string;
  sandbox_id: string;
  source: string;
  destination: string;
  explanation: string;
  approved?: boolean;
}
