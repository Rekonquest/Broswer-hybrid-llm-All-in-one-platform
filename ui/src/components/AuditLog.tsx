import { ScrollText, CheckCircle, XCircle, Clock } from 'lucide-react';
import { AuditLogEntry } from '../types';

interface Props {
  entries: AuditLogEntry[];
}

export default function AuditLog({ entries }: Props) {
  return (
    <div className="card">
      <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
        <ScrollText size={20} />
        Audit Log ({entries.length})
      </h2>

      {entries.length === 0 ? (
        <div className="text-center py-12 text-gray-500">
          <ScrollText size={48} className="mx-auto mb-3 opacity-50" />
          <p>No audit entries yet</p>
          <p className="text-sm mt-1">All LLM actions will be logged here</p>
        </div>
      ) : (
        <div className="space-y-2 max-h-[600px] overflow-y-auto">
          {entries.map((entry) => (
            <div
              key={entry.id}
              className={`p-3 rounded-lg border ${
                entry.approved
                  ? 'bg-gray-800 border-gray-700'
                  : 'bg-danger-500/5 border-danger-500/20'
              }`}
            >
              <div className="flex items-start justify-between gap-3">
                <div className="flex items-start gap-2 flex-1">
                  {entry.approved ? (
                    <CheckCircle size={16} className="text-success-500 mt-0.5 flex-shrink-0" />
                  ) : (
                    <XCircle size={16} className="text-danger-500 mt-0.5 flex-shrink-0" />
                  )}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="font-medium text-sm">{entry.action}</span>
                      {entry.llm_id && (
                        <span className="text-xs px-2 py-0.5 bg-gray-700 rounded-full text-gray-400">
                          {entry.llm_id}
                        </span>
                      )}
                    </div>
                    {entry.reason && (
                      <p className="text-xs text-gray-400 mb-1">{entry.reason}</p>
                    )}
                    <div className="flex items-center gap-1 text-xs text-gray-500">
                      <Clock size={12} />
                      {new Date(entry.timestamp).toLocaleString()}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
