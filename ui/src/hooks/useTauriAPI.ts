import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { readBinaryFile } from '@tauri-apps/api/fs';
import {
  SystemState,
  LockdownRequest,
  LockdownResponse,
  SendMessageRequest,
  SendMessageResponse,
  LoadLLMRequest,
  LoadLLMResponse,
  UnloadLLMRequest,
  UnloadLLMResponse,
  UploadDocumentRequest,
  UploadDocumentResponse,
  DeleteDocumentRequest,
  DeleteDocumentResponse,
  UpdatePermissionsRequest,
  UpdatePermissionsResponse,
  CreateSandboxRequest,
  CreateSandboxResponse,
  ExecuteInSandboxRequest,
  ExecuteInSandboxResponse,
  GetSandboxFilesRequest,
  GetSandboxFilesResponse,
  ApproveTransferRequest,
  ApproveTransferResponse,
} from '../types/api';
import { LLMInstance, Document, Permissions, AuditLogEntry } from '../types';

export function useTauriAPI() {
  // System Commands
  const getSystemState = async (): Promise<SystemState> => {
    return await invoke<SystemState>('get_system_state');
  };

  const triggerLockdown = async (reason: string): Promise<LockdownResponse> => {
    const request: LockdownRequest = { reason };
    return await invoke<LockdownResponse>('trigger_lockdown', { request });
  };

  const releaseLockdown = async (password: string): Promise<LockdownResponse> => {
    return await invoke<LockdownResponse>('release_lockdown', { password });
  };

  // LLM Commands
  const getLLMs = async (): Promise<LLMInstance[]> => {
    return await invoke<LLMInstance[]>('get_llms');
  };

  const loadLLM = async (llmId: string): Promise<LoadLLMResponse> => {
    const request: LoadLLMRequest = { llm_id: llmId };
    return await invoke<LoadLLMResponse>('load_llm', { request });
  };

  const unloadLLM = async (llmId: string): Promise<UnloadLLMResponse> => {
    const request: UnloadLLMRequest = { llm_id: llmId };
    return await invoke<UnloadLLMResponse>('unload_llm', { request });
  };

  const sendMessage = async (
    llmId: string,
    content: string,
    context?: Record<string, any>
  ): Promise<SendMessageResponse> => {
    const request: SendMessageRequest = { llm_id: llmId, content, context };
    return await invoke<SendMessageResponse>('send_message', { request });
  };

  // Document Commands
  const uploadDocument = async (file: File): Promise<UploadDocumentResponse> => {
    const arrayBuffer = await file.arrayBuffer();
    const base64 = btoa(
      new Uint8Array(arrayBuffer).reduce((data, byte) => data + String.fromCharCode(byte), '')
    );

    const request: UploadDocumentRequest = {
      name: file.name,
      content: base64,
      mime_type: file.type || 'application/octet-stream',
    };

    return await invoke<UploadDocumentResponse>('upload_document', { request });
  };

  const uploadDocumentFromDialog = async (): Promise<UploadDocumentResponse | null> => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Documents',
          extensions: ['txt', 'md', 'json', 'csv', 'pdf', 'doc', 'docx'],
        },
      ],
    });

    if (!selected || typeof selected !== 'string') {
      return null;
    }

    const fileName = selected.split('/').pop() || 'unnamed';
    const content = await readBinaryFile(selected);
    const base64 = btoa(
      new Uint8Array(content).reduce((data, byte) => data + String.fromCharCode(byte), '')
    );

    const request: UploadDocumentRequest = {
      name: fileName,
      content: base64,
      mime_type: 'application/octet-stream',
    };

    return await invoke<UploadDocumentResponse>('upload_document', { request });
  };

  const getDocuments = async (): Promise<Document[]> => {
    return await invoke<Document[]>('get_documents');
  };

  const deleteDocument = async (documentId: string): Promise<DeleteDocumentResponse> => {
    const request: DeleteDocumentRequest = { document_id: documentId };
    return await invoke<DeleteDocumentResponse>('delete_document', { request });
  };

  // Permission Commands
  const getPermissions = async (): Promise<Permissions> => {
    return await invoke<Permissions>('get_permissions');
  };

  const updatePermissions = async (permissions: Permissions): Promise<UpdatePermissionsResponse> => {
    const request: UpdatePermissionsRequest = { permissions };
    return await invoke<UpdatePermissionsResponse>('update_permissions', { request });
  };

  // Audit Commands
  const getAuditLog = async (): Promise<AuditLogEntry[]> => {
    return await invoke<AuditLogEntry[]>('get_audit_log');
  };

  // Sandbox Commands
  const createSandbox = async (
    name: string,
    config?: CreateSandboxRequest['config']
  ): Promise<CreateSandboxResponse> => {
    const request: CreateSandboxRequest = { name, config };
    return await invoke<CreateSandboxResponse>('create_sandbox', { request });
  };

  const executeInSandbox = async (
    sandboxId: string,
    code: string,
    language: string
  ): Promise<ExecuteInSandboxResponse> => {
    const request: ExecuteInSandboxRequest = {
      sandbox_id: sandboxId,
      code,
      language,
    };
    return await invoke<ExecuteInSandboxResponse>('execute_in_sandbox', { request });
  };

  const getSandboxFiles = async (
    sandboxId: string,
    path?: string
  ): Promise<GetSandboxFilesResponse> => {
    const request: GetSandboxFilesRequest = { sandbox_id: sandboxId, path };
    return await invoke<GetSandboxFilesResponse>('get_sandbox_files', { request });
  };

  const approveTransfer = async (
    sandboxId: string,
    filePath: string,
    destination: string
  ): Promise<ApproveTransferResponse> => {
    const request: ApproveTransferRequest = {
      sandbox_id: sandboxId,
      file_path: filePath,
      destination,
    };
    return await invoke<ApproveTransferResponse>('approve_transfer', { request });
  };

  return {
    // System
    getSystemState,
    triggerLockdown,
    releaseLockdown,
    // LLMs
    getLLMs,
    loadLLM,
    unloadLLM,
    sendMessage,
    // Documents
    uploadDocument,
    uploadDocumentFromDialog,
    getDocuments,
    deleteDocument,
    // Permissions
    getPermissions,
    updatePermissions,
    // Audit
    getAuditLog,
    // Sandbox
    createSandbox,
    executeInSandbox,
    getSandboxFiles,
    approveTransfer,
  };
}
