use tauri::State;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error, debug};

use common::{
    types::{LLMInstance, PermissionScope, LockdownState, LockdownReason},
    errors::Result,
};
use crate::state::{AppState, SystemState, Document, AuditLogEntry};

// ============================================================================
// System Commands
// ============================================================================

#[tauri::command]
pub async fn get_system_state(state: State<'_, AppState>) -> Result<SystemState, String> {
    info!("📊 Getting system state");
    Ok(state.get_system_state().await)
}

#[tauri::command]
pub async fn trigger_lockdown(
    state: State<'_, AppState>,
    reason: String,
) -> Result<(), String> {
    info!("🚨 Triggering lockdown: {}", reason);

    state.security_engine
        .trigger_lockdown(LockdownReason::UserPanicButton)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn release_lockdown(
    state: State<'_, AppState>,
    auth_token: String,
) -> Result<(), String> {
    info!("🔓 Releasing lockdown");

    state.security_engine
        .release_lockdown(&auth_token)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// LLM Commands
// ============================================================================

#[tauri::command]
pub async fn get_llms(state: State<'_, AppState>) -> Result<Vec<LLMInstance>, String> {
    debug!("📋 Getting LLM list");

    let pool = state.llm_pool.read().await;
    let llms: Vec<LLMInstance> = pool.get_all_ids()
        .iter()
        .filter_map(|id| pool.get(id))
        .map(|provider| provider.instance().clone())
        .collect();

    Ok(llms)
}

#[tauri::command]
pub async fn load_llm(
    state: State<'_, AppState>,
    llm_id: String,
) -> Result<(), String> {
    info!("⬆️  Loading LLM: {}", llm_id);

    let pool = state.llm_pool.read().await;
    pool.load(&llm_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn unload_llm(
    state: State<'_, AppState>,
    llm_id: String,
) -> Result<(), String> {
    info!("⬇️  Unloading LLM: {}", llm_id);

    let pool = state.llm_pool.read().await;
    pool.unload(&llm_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub llm_id: String,
    pub content: String,
    pub conversation_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub content: String,
    pub llm_id: String,
}

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    request: SendMessageRequest,
) -> Result<SendMessageResponse, String> {
    info!("💬 Sending message to LLM: {}", request.llm_id);

    let pool = state.llm_pool.read().await;
    let provider = pool.get(&request.llm_id)
        .ok_or_else(|| format!("LLM not found: {}", request.llm_id))?;

    let response = provider.complete(&request.content, std::collections::HashMap::new())
        .await
        .map_err(|e| e.to_string())?;

    Ok(SendMessageResponse {
        content: response,
        llm_id: request.llm_id,
    })
}

// ============================================================================
// Document Commands
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct UploadDocumentRequest {
    pub filename: String,
    pub content: Vec<u8>,
}

#[tauri::command]
pub async fn upload_document(
    state: State<'_, AppState>,
    request: UploadDocumentRequest,
) -> Result<Document, String> {
    info!("📤 Uploading document: {}", request.filename);

    let doc = Document {
        id: Uuid::new_v4(),
        filename: request.filename.clone(),
        size: request.content.len(),
        uploaded_at: chrono::Utc::now(),
        indexed: false,
        chunk_count: None,
    };

    // TODO: Actually save and index the document
    // For now, just add to in-memory list
    let mut documents = state.documents.write().await;
    documents.push(doc.clone());

    info!("✅ Document uploaded: {}", doc.id);

    Ok(doc)
}

#[tauri::command]
pub async fn get_documents(state: State<'_, AppState>) -> Result<Vec<Document>, String> {
    debug!("📋 Getting document list");

    let documents = state.documents.read().await;
    Ok(documents.clone())
}

#[tauri::command]
pub async fn delete_document(
    state: State<'_, AppState>,
    document_id: Uuid,
) -> Result<(), String> {
    info!("🗑️  Deleting document: {}", document_id);

    let mut documents = state.documents.write().await;
    documents.retain(|doc| doc.id != document_id);

    Ok(())
}

// ============================================================================
// Permission Commands
// ============================================================================

#[tauri::command]
pub async fn get_permissions(state: State<'_, AppState>) -> Result<PermissionScope, String> {
    debug!("📋 Getting permissions");

    let permissions = state.permissions.read().await;
    Ok(permissions.clone())
}

#[tauri::command]
pub async fn update_permissions(
    state: State<'_, AppState>,
    permissions: PermissionScope,
) -> Result<(), String> {
    info!("💾 Updating permissions");

    let mut perms = state.permissions.write().await;
    *perms = permissions;

    Ok(())
}

// ============================================================================
// Audit Log Commands
// ============================================================================

#[tauri::command]
pub async fn get_audit_log(state: State<'_, AppState>) -> Result<Vec<AuditLogEntry>, String> {
    debug!("📋 Getting audit log");

    let log = state.audit_log.read().await;
    Ok(log.clone())
}

// ============================================================================
// Sandbox Commands
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateSandboxRequest {
    pub llm_id: String,
    pub purpose: String,
}

#[derive(Debug, Serialize)]
pub struct CreateSandboxResponse {
    pub sandbox_id: Uuid,
}

#[tauri::command]
pub async fn create_sandbox(
    _state: State<'_, AppState>,
    request: CreateSandboxRequest,
) -> Result<CreateSandboxResponse, String> {
    info!("📦 Creating sandbox for LLM: {}", request.llm_id);

    // TODO: Actually create sandbox
    let sandbox_id = Uuid::new_v4();

    Ok(CreateSandboxResponse { sandbox_id })
}

#[derive(Debug, Deserialize)]
pub struct ExecuteInSandboxRequest {
    pub sandbox_id: Uuid,
    pub code: String,
    pub language: String,
}

#[derive(Debug, Serialize)]
pub struct ExecuteInSandboxResponse {
    pub output: String,
    pub exit_code: i32,
}

#[tauri::command]
pub async fn execute_in_sandbox(
    _state: State<'_, AppState>,
    request: ExecuteInSandboxRequest,
) -> Result<ExecuteInSandboxResponse, String> {
    info!("🚀 Executing code in sandbox: {}", request.sandbox_id);

    // TODO: Actually execute in sandbox
    Ok(ExecuteInSandboxResponse {
        output: format!("Executed {} code (placeholder)", request.language),
        exit_code: 0,
    })
}

#[derive(Debug, Serialize)]
pub struct SandboxFile {
    pub path: String,
    pub size: usize,
    pub modified: chrono::DateTime<chrono::Utc>,
}

#[tauri::command]
pub async fn get_sandbox_files(
    _state: State<'_, AppState>,
    sandbox_id: Uuid,
) -> Result<Vec<SandboxFile>, String> {
    debug!("📋 Getting files for sandbox: {}", sandbox_id);

    // TODO: Actually list sandbox files
    Ok(Vec::new())
}

#[derive(Debug, Deserialize)]
pub struct ApproveTransferRequest {
    pub transfer_id: Uuid,
    pub approved: bool,
}

#[tauri::command]
pub async fn approve_transfer(
    _state: State<'_, AppState>,
    request: ApproveTransferRequest,
) -> Result<(), String> {
    info!("✅ Transfer approval: {} - {}", request.transfer_id, request.approved);

    // TODO: Actually handle transfer
    Ok(())
}
