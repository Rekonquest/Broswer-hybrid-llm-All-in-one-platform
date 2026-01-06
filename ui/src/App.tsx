import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { LLMInstance, SystemState, Document, AuditLogEntry } from './types';
import Dashboard from './pages/Dashboard';
import { AlertCircle } from 'lucide-react';

function App() {
  const [systemState, setSystemState] = useState<SystemState>({
    lockdown: 'normal',
    active_llms: [],
    pending_approvals: 0,
  });
  const [llms, setLlms] = useState<LLMInstance[]>([]);
  const [documents, setDocuments] = useState<Document[]>([]);
  const [auditLog, setAuditLog] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadInitialData();
  }, []);

  const loadInitialData = async () => {
    try {
      setLoading(true);
      // TODO: Connect to Tauri backend
      // const state = await invoke<SystemState>('get_system_state');
      // const llmList = await invoke<LLMInstance[]>('get_llms');

      // Mock data for now
      setSystemState({
        lockdown: 'normal',
        active_llms: [],
        pending_approvals: 0,
      });

      setLlms([]);
      setDocuments([]);
      setAuditLog([]);

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  const handlePanicButton = async () => {
    try {
      // await invoke('trigger_lockdown', { reason: 'user_panic_button' });
      setSystemState(prev => ({ ...prev, lockdown: 'locked' }));
    } catch (err) {
      setError('Failed to trigger lockdown');
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-t-2 border-b-2 border-primary-500 mx-auto mb-4"></div>
          <p className="text-gray-400">Loading Hybrid LLM Platform...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center p-4">
        <div className="card max-w-md w-full">
          <div className="flex items-center gap-3 text-danger-500 mb-4">
            <AlertCircle size={24} />
            <h2 className="text-xl font-bold">Error</h2>
          </div>
          <p className="text-gray-300">{error}</p>
          <button
            onClick={loadInitialData}
            className="btn btn-primary mt-4 w-full"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-950">
      <Dashboard
        systemState={systemState}
        llms={llms}
        documents={documents}
        auditLog={auditLog}
        onPanicButton={handlePanicButton}
        onRefresh={loadInitialData}
      />
    </div>
  );
}

export default App;
