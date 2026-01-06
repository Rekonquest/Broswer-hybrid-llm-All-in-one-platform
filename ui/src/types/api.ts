// Tauri API Request/Response Types

// System Commands
export interface SystemState {
  lockdown_state: 'Normal' | 'ReadOnly' | 'Locked';
  active_llms: number;
  total_documents: number;
  sandbox_count: number;
  uptime_seconds: number;
}

export interface LockdownRequest {
  reason: string;
}

export interface LockdownResponse {
  success: boolean;
  new_state: 'Normal' | 'ReadOnly' | 'Locked';
}

// LLM Commands
export interface SendMessageRequest {
  llm_id: string;
  content: string;
  context?: Record<string, any>;
}

export interface SendMessageResponse {
  content: string;
  llm_id: string;
}

export interface LoadLLMRequest {
  llm_id: string;
}

export interface LoadLLMResponse {
  success: boolean;
  llm_id: string;
}

export interface UnloadLLMRequest {
  llm_id: string;
}

export interface UnloadLLMResponse {
  success: boolean;
  llm_id: string;
}

// Document Commands
export interface UploadDocumentRequest {
  name: string;
  content: string; // Base64 encoded for binary files
  mime_type: string;
}

export interface UploadDocumentResponse {
  id: string;
  name: string;
  uploaded_at: string;
}

export interface DeleteDocumentRequest {
  document_id: string;
}

export interface DeleteDocumentResponse {
  success: boolean;
}

// Permission Commands
export interface UpdatePermissionsRequest {
  permissions: Permissions;
}

export interface UpdatePermissionsResponse {
  success: boolean;
}

// Sandbox Commands
export interface CreateSandboxRequest {
  name: string;
  config?: SandboxConfig;
}

export interface SandboxConfig {
  cpu_limit?: number;
  memory_limit_mb?: number;
  disk_limit_mb?: number;
  timeout_seconds?: number;
}

export interface CreateSandboxResponse {
  sandbox_id: string;
  name: string;
}

export interface ExecuteInSandboxRequest {
  sandbox_id: string;
  code: string;
  language: string;
}

export interface ExecuteInSandboxResponse {
  sandbox_id: string;
  output: string;
  exit_code: number;
  execution_time_ms: number;
}

export interface GetSandboxFilesRequest {
  sandbox_id: string;
  path?: string;
}

export interface SandboxFile {
  name: string;
  path: string;
  size: number;
  is_directory: boolean;
}

export interface GetSandboxFilesResponse {
  sandbox_id: string;
  files: SandboxFile[];
}

export interface ApproveTransferRequest {
  sandbox_id: string;
  file_path: string;
  destination: string;
}

export interface ApproveTransferResponse {
  success: boolean;
  transferred_path: string;
}

// WebSocket Message Types
export interface WebSocketMessage {
  type: 'llm_status' | 'document_uploaded' | 'lockdown_changed' | 'audit_log' | 'sandbox_output';
  payload: any;
}

export interface LLMStatusMessage {
  llm_id: string;
  status: 'loaded' | 'unloaded' | 'processing';
}

export interface DocumentUploadedMessage {
  document_id: string;
  name: string;
}

export interface LockdownChangedMessage {
  old_state: 'Normal' | 'ReadOnly' | 'Locked';
  new_state: 'Normal' | 'ReadOnly' | 'Locked';
  reason: string;
}

export interface SandboxOutputMessage {
  sandbox_id: string;
  output: string;
  stream_type: 'stdout' | 'stderr';
}
