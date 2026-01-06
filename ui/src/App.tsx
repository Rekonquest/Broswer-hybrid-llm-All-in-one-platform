import { useState, useEffect } from 'react';
import { LLMInstance, SystemState, Document, AuditLogEntry } from './types';
import Dashboard from './pages/Dashboard';
import { AlertCircle } from 'lucide-react';
import { useTauriAPI } from './hooks/useTauriAPI';
import { useWebSocket } from './hooks/useWebSocket';

function App() {
  const api = useTauriAPI();
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

  // WebSocket connection for real-time updates
  const { isConnected } = useWebSocket({
    onLLMStatus: (message) => {
      console.log('LLM status update:', message);
      loadLLMs();
    },
    onDocumentUploaded: (message) => {
      console.log('Document uploaded:', message);
      loadDocuments();
    },
    onLockdownChanged: (message) => {
      console.log('Lockdown changed:', message);
      loadSystemState();
    },
    onAuditLog: (message) => {
      console.log('Audit log entry:', message);
      loadAuditLog();
    },
  });

  useEffect(() => {
    loadInitialData();
  }, []);

  const loadSystemState = async () => {
    try {
      const state = await api.getSystemState();
      setSystemState({
        lockdown: state.lockdown_state.toLowerCase() as 'normal' | 'readonly' | 'locked',
        active_llms: [], // Will be populated from LLMs list
        pending_approvals: 0, // TODO: Add to backend
      });
    } catch (err) {
      console.error('Failed to load system state:', err);
    }
  };

  const loadLLMs = async () => {
    try {
      const llmList = await api.getLLMs();
      setLlms(llmList);
      setSystemState(prev => ({
        ...prev,
        active_llms: llmList.filter(llm => llm.loaded).map(llm => llm.id),
      }));
    } catch (err) {
      console.error('Failed to load LLMs:', err);
    }
  };

  const loadDocuments = async () => {
    try {
      const docs = await api.getDocuments();
      setDocuments(docs);
    } catch (err) {
      console.error('Failed to load documents:', err);
    }
  };

  const loadAuditLog = async () => {
    try {
      const log = await api.getAuditLog();
      setAuditLog(log);
    } catch (err) {
      console.error('Failed to load audit log:', err);
    }
  };

  const loadInitialData = async () => {
    try {
      setLoading(true);
      await Promise.all([
        loadSystemState(),
        loadLLMs(),
        loadDocuments(),
        loadAuditLog(),
      ]);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  const handlePanicButton = async () => {
    try {
      const result = await api.triggerLockdown('User pressed panic button');
      setSystemState(prev => ({
        ...prev,
        lockdown: result.new_state.toLowerCase() as 'normal' | 'readonly' | 'locked',
      }));
    } catch (err) {
      setError('Failed to trigger lockdown');
      console.error(err);
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
        isConnected={isConnected}
        api={api}
      />
    </div>
  );
}

export default App;
