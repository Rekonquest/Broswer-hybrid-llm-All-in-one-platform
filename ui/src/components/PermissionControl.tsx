import { useState } from 'react';
import { Lock, Shield, AlertTriangle, CheckCircle } from 'lucide-react';
import { PermissionScope } from '../types';

interface Props {
  permissions: PermissionScope;
  onUpdate: (permissions: PermissionScope) => void;
  lockdownState: 'normal' | 'readonly' | 'locked';
}

export default function PermissionControl({ permissions, onUpdate, lockdownState }: Props) {
  const [activeTab, setActiveTab] = useState<'filesystem' | 'network' | 'commands' | 'resources'>('filesystem');

  const getLockdownBadge = () => {
    switch (lockdownState) {
      case 'locked':
        return (
          <span className="flex items-center gap-1 text-danger-500 bg-danger-500/10 px-3 py-1 rounded-full text-sm font-medium">
            <AlertTriangle size={14} />
            LOCKED
          </span>
        );
      case 'readonly':
        return (
          <span className="flex items-center gap-1 text-yellow-500 bg-yellow-500/10 px-3 py-1 rounded-full text-sm font-medium">
            <Lock size={14} />
            READ-ONLY
          </span>
        );
      default:
        return (
          <span className="flex items-center gap-1 text-success-500 bg-success-500/10 px-3 py-1 rounded-full text-sm font-medium">
            <CheckCircle size={14} />
            NORMAL
          </span>
        );
    }
  };

  return (
    <div className="card">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold flex items-center gap-2">
          <Shield size={20} />
          Security Permissions
        </h2>
        {getLockdownBadge()}
      </div>

      <div className="flex gap-2 mb-4 border-b border-gray-800">
        {['filesystem', 'network', 'commands', 'resources'].map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab as any)}
            className={`px-4 py-2 font-medium transition-colors ${
              activeTab === tab
                ? 'text-primary-400 border-b-2 border-primary-500'
                : 'text-gray-500 hover:text-gray-300'
            }`}
          >
            {tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
      </div>

      <div className="space-y-4">
        {activeTab === 'filesystem' && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Read Paths
              </label>
              <div className="space-y-2">
                {permissions.file_system.read_paths.map((path, idx) => (
                  <div key={idx} className="flex items-center gap-2">
                    <input
                      type="text"
                      value={path}
                      onChange={(e) => {
                        const newPaths = [...permissions.file_system.read_paths];
                        newPaths[idx] = e.target.value;
                        onUpdate({
                          ...permissions,
                          file_system: { ...permissions.file_system, read_paths: newPaths },
                        });
                      }}
                      className="input flex-1 text-sm"
                      disabled={lockdownState === 'locked'}
                    />
                  </div>
                ))}
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Write Paths
              </label>
              <div className="space-y-2">
                {permissions.file_system.write_paths.map((path, idx) => (
                  <div key={idx} className="flex items-center gap-2">
                    <input
                      type="text"
                      value={path}
                      onChange={(e) => {
                        const newPaths = [...permissions.file_system.write_paths];
                        newPaths[idx] = e.target.value;
                        onUpdate({
                          ...permissions,
                          file_system: { ...permissions.file_system, write_paths: newPaths },
                        });
                      }}
                      className="input flex-1 text-sm"
                      disabled={lockdownState === 'locked'}
                    />
                  </div>
                ))}
              </div>
            </div>
          </>
        )}

        {activeTab === 'network' && (
          <div className="space-y-4">
            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={permissions.network.outbound}
                onChange={(e) =>
                  onUpdate({
                    ...permissions,
                    network: { ...permissions.network, outbound: e.target.checked },
                  })
                }
                className="w-4 h-4"
                disabled={lockdownState === 'locked'}
              />
              <span className="text-sm font-medium">Allow Outbound Connections</span>
            </label>

            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={permissions.network.inbound}
                onChange={(e) =>
                  onUpdate({
                    ...permissions,
                    network: { ...permissions.network, inbound: e.target.checked },
                  })
                }
                className="w-4 h-4"
                disabled={lockdownState === 'locked'}
              />
              <span className="text-sm font-medium">Allow Inbound Connections</span>
            </label>

            <label className="flex items-center gap-3">
              <input
                type="checkbox"
                checked={permissions.network.require_approval}
                onChange={(e) =>
                  onUpdate({
                    ...permissions,
                    network: { ...permissions.network, require_approval: e.target.checked },
                  })
                }
                className="w-4 h-4"
                disabled={lockdownState === 'locked'}
              />
              <span className="text-sm font-medium">Require Approval for Network Access</span>
            </label>
          </div>
        )}

        {activeTab === 'commands' && (
          <>
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Allowed Commands (Whitelist)
              </label>
              <textarea
                value={permissions.commands.whitelist.join('\n')}
                onChange={(e) =>
                  onUpdate({
                    ...permissions,
                    commands: {
                      ...permissions.commands,
                      whitelist: e.target.value.split('\n').filter((s) => s.trim()),
                    },
                  })
                }
                className="input w-full h-32 font-mono text-sm"
                disabled={lockdownState === 'locked'}
                placeholder="git&#10;npm&#10;python"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Blocked Commands (Blacklist)
              </label>
              <textarea
                value={permissions.commands.blacklist.join('\n')}
                onChange={(e) =>
                  onUpdate({
                    ...permissions,
                    commands: {
                      ...permissions.commands,
                      blacklist: e.target.value.split('\n').filter((s) => s.trim()),
                    },
                  })
                }
                className="input w-full h-32 font-mono text-sm"
                disabled={lockdownState === 'locked'}
                placeholder="rm -rf /&#10;sudo&#10;dd"
              />
            </div>
          </>
        )}

        {activeTab === 'resources' && (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Max CPU Usage: {permissions.resources.max_cpu_percent}%
              </label>
              <input
                type="range"
                min="0"
                max="100"
                value={permissions.resources.max_cpu_percent}
                onChange={(e) =>
                  onUpdate({
                    ...permissions,
                    resources: {
                      ...permissions.resources,
                      max_cpu_percent: parseInt(e.target.value),
                    },
                  })
                }
                className="w-full"
                disabled={lockdownState === 'locked'}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Max Memory: {permissions.resources.max_memory_gb} GB
              </label>
              <input
                type="range"
                min="1"
                max="32"
                value={permissions.resources.max_memory_gb}
                onChange={(e) =>
                  onUpdate({
                    ...permissions,
                    resources: {
                      ...permissions.resources,
                      max_memory_gb: parseInt(e.target.value),
                    },
                  })
                }
                className="w-full"
                disabled={lockdownState === 'locked'}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Max Disk Usage: {permissions.resources.max_disk_gb} GB
              </label>
              <input
                type="range"
                min="1"
                max="100"
                value={permissions.resources.max_disk_gb}
                onChange={(e) =>
                  onUpdate({
                    ...permissions,
                    resources: {
                      ...permissions.resources,
                      max_disk_gb: parseInt(e.target.value),
                    },
                  })
                }
                className="w-full"
                disabled={lockdownState === 'locked'}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
