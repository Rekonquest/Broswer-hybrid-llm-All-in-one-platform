import { LLMInstance, LLMStatus } from '../types';
import { Cpu, Circle, Play, Square } from 'lucide-react';

interface Props {
  llms: LLMInstance[];
  statuses: Map<string, LLMStatus>;
  onLoadModel: (id: string) => void;
  onUnloadModel: (id: string) => void;
}

export default function LLMManager({ llms, statuses, onLoadModel, onUnloadModel }: Props) {
  const getProviderColor = (provider: string) => {
    switch (provider) {
      case 'local':
        return 'text-green-500';
      case 'claude':
        return 'text-purple-500';
      case 'openai':
        return 'text-blue-500';
      case 'gemini':
        return 'text-yellow-500';
      default:
        return 'text-gray-500';
    }
  };

  const getStatusColor = (status?: string) => {
    switch (status) {
      case 'processing':
        return 'text-primary-500 animate-pulse';
      case 'error':
        return 'text-danger-500';
      case 'idle':
      default:
        return 'text-success-500';
    }
  };

  return (
    <div className="card">
      <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
        <Cpu size={20} />
        LLM Management
      </h2>

      <div className="space-y-3">
        {llms.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            <p>No LLMs configured</p>
            <p className="text-sm mt-1">Add models in config.toml</p>
          </div>
        ) : (
          llms.map((llm) => {
            const status = statuses.get(llm.id);
            return (
              <div
                key={llm.id}
                className="p-4 bg-gray-800 rounded-lg border border-gray-700 hover:border-gray-600 transition-colors"
              >
                <div className="flex items-start justify-between mb-2">
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <Circle
                        size={8}
                        className={getStatusColor(status?.status)}
                        fill="currentColor"
                      />
                      <h3 className="font-semibold">{llm.id}</h3>
                      <span
                        className={`text-xs px-2 py-0.5 rounded-full ${getProviderColor(
                          llm.provider
                        )} bg-opacity-10 border border-current`}
                      >
                        {llm.provider}
                      </span>
                    </div>
                    <p className="text-sm text-gray-400">{llm.model_name}</p>
                  </div>

                  <button
                    onClick={() =>
                      llm.is_loaded ? onUnloadModel(llm.id) : onLoadModel(llm.id)
                    }
                    className={`btn btn-sm ${
                      llm.is_loaded ? 'btn-secondary' : 'btn-primary'
                    }`}
                    disabled={status?.status === 'processing'}
                  >
                    {llm.is_loaded ? (
                      <>
                        <Square size={14} className="mr-1" />
                        Unload
                      </>
                    ) : (
                      <>
                        <Play size={14} className="mr-1" />
                        Load
                      </>
                    )}
                  </button>
                </div>

                <div className="flex flex-wrap gap-2 mb-2">
                  {llm.capabilities.map((cap) => (
                    <span
                      key={cap}
                      className="text-xs px-2 py-1 bg-gray-700 rounded-full text-gray-300"
                    >
                      {cap}
                    </span>
                  ))}
                </div>

                <div className="flex items-center justify-between text-xs text-gray-500">
                  <span>Context: {(llm.max_context / 1024).toFixed(0)}K tokens</span>
                  {status?.current_task && (
                    <span className="text-primary-400">{status.current_task}</span>
                  )}
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
