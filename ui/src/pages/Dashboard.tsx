import { useState, useEffect } from 'react';
import { AlertOctagon, RefreshCw, Terminal, Wifi, WifiOff } from 'lucide-react';
import { SystemState, LLMInstance, Document, AuditLogEntry, PermissionScope, LLMStatus } from '../types';
import { useTauriAPI } from '../hooks/useTauriAPI';
import DocumentUpload from '../components/DocumentUpload';
import LLMManager from '../components/LLMManager';
import PermissionControl from '../components/PermissionControl';
import CodingCanvas from '../components/CodingCanvas';
import AuditLog from '../components/AuditLog';

interface Props {
  systemState: SystemState;
  llms: LLMInstance[];
  documents: Document[];
  auditLog: AuditLogEntry[];
  onPanicButton: () => void;
  onRefresh: () => void;
  isConnected: boolean;
  api: ReturnType<typeof useTauriAPI>;
}

export default function Dashboard({
  systemState,
  llms,
  documents,
  auditLog,
  onPanicButton,
  onRefresh,
  isConnected,
  api,
}: Props) {
  const [activeView, setActiveView] = useState<'overview' | 'canvas' | 'permissions' | 'audit'>('overview');
  const [llmStatuses] = useState<Map<string, LLMStatus>>(new Map());
  const [permissions, setPermissions] = useState<PermissionScope>({
    file_system: {
      read_paths: ['/home/*/downloads/*', '/rag/*'],
      write_paths: ['/home/*/downloads/*'],
      execute_paths: [],
    },
    network: {
      inbound: true,
      outbound: true,
      require_approval: true,
    },
    commands: {
      whitelist: ['git', 'npm', 'python', 'cargo'],
      blacklist: ['rm -rf /', 'sudo', 'dd'],
      require_explanation: true,
    },
    resources: {
      max_cpu_percent: 80,
      max_memory_gb: 8,
      max_disk_gb: 50,
    },
  });

  // Load permissions from backend on mount
  useEffect(() => {
    const loadPermissions = async () => {
      try {
        const perms = await api.getPermissions();
        setPermissions(perms);
      } catch (err) {
        console.error('Failed to load permissions:', err);
      }
    };
    loadPermissions();
  }, [api]);

  const handleDocumentUpload = async (files: File[]) => {
    try {
      for (const file of files) {
        await api.uploadDocument(file);
      }
      onRefresh(); // Refresh to show new documents
    } catch (err) {
      console.error('Failed to upload documents:', err);
    }
  };

  const handleLoadModel = async (id: string) => {
    try {
      await api.loadLLM(id);
      onRefresh(); // Refresh to show updated LLM status
    } catch (err) {
      console.error('Failed to load LLM:', err);
    }
  };

  const handleUnloadModel = async (id: string) => {
    try {
      await api.unloadLLM(id);
      onRefresh(); // Refresh to show updated LLM status
    } catch (err) {
      console.error('Failed to unload LLM:', err);
    }
  };

  const handlePermissionUpdate = async (newPermissions: PermissionScope) => {
    try {
      await api.updatePermissions(newPermissions);
      setPermissions(newPermissions);
    } catch (err) {
      console.error('Failed to update permissions:', err);
    }
  };

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="bg-gray-900 border-b border-gray-800 sticky top-0 z-50">
        <div className="px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="flex items-center gap-2">
                <Terminal size={24} className="text-primary-500" />
                <h1 className="text-2xl font-bold">Hybrid LLM Platform</h1>
              </div>

              <nav className="flex gap-2 ml-8">
                {[
                  { id: 'overview', label: 'Overview' },
                  { id: 'canvas', label: 'Coding Canvas' },
                  { id: 'permissions', label: 'Permissions' },
                  { id: 'audit', label: 'Audit Log' },
                ].map((view) => (
                  <button
                    key={view.id}
                    onClick={() => setActiveView(view.id as any)}
                    className={`px-4 py-2 rounded-lg transition-colors ${
                      activeView === view.id
                        ? 'bg-primary-600 text-white'
                        : 'text-gray-400 hover:text-gray-200 hover:bg-gray-800'
                    }`}
                  >
                    {view.label}
                  </button>
                ))}
              </nav>
            </div>

            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-gray-800 text-xs">
                {isConnected ? (
                  <>
                    <Wifi size={14} className="text-success-500" />
                    <span className="text-gray-400">Connected</span>
                  </>
                ) : (
                  <>
                    <WifiOff size={14} className="text-danger-500" />
                    <span className="text-gray-400">Disconnected</span>
                  </>
                )}
              </div>

              <button
                onClick={onRefresh}
                className="btn btn-secondary"
                title="Refresh"
              >
                <RefreshCw size={16} />
              </button>

              <button
                onClick={onPanicButton}
                className="btn btn-danger flex items-center gap-2"
                disabled={systemState.lockdown === 'locked'}
              >
                <AlertOctagon size={16} />
                PANIC
              </button>
            </div>
          </div>

          {systemState.lockdown !== 'normal' && (
            <div className="mt-4 p-3 bg-danger-500/10 border border-danger-500 rounded-lg flex items-center gap-2">
              <AlertOctagon size={16} className="text-danger-500" />
              <span className="text-danger-500 font-medium">
                System in {systemState.lockdown.toUpperCase()} mode
              </span>
            </div>
          )}
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 p-6">
        {activeView === 'overview' && (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <LLMManager
              llms={llms}
              statuses={llmStatuses}
              onLoadModel={handleLoadModel}
              onUnloadModel={handleUnloadModel}
            />
            <DocumentUpload
              documents={documents}
              onUpload={handleDocumentUpload}
            />
          </div>
        )}

        {activeView === 'canvas' && (
          <CodingCanvas api={api} />
        )}

        {activeView === 'permissions' && (
          <PermissionControl
            permissions={permissions}
            onUpdate={handlePermissionUpdate}
            lockdownState={systemState.lockdown}
          />
        )}

        {activeView === 'audit' && (
          <AuditLog entries={auditLog} />
        )}
      </main>

      {/* Footer */}
      <footer className="bg-gray-900 border-t border-gray-800 px-6 py-3">
        <div className="flex items-center justify-between text-sm text-gray-500">
          <div className="flex items-center gap-6">
            <span>Active LLMs: {systemState.active_llms.length}</span>
            <span>Pending Approvals: {systemState.pending_approvals}</span>
            <span>Documents: {documents.length}</span>
          </div>
          <div>
            <span>Hybrid LLM Platform v0.1.0</span>
          </div>
        </div>
      </footer>
    </div>
  );
}
