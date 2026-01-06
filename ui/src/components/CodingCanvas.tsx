import { useState, useEffect } from 'react';
import { Code, Save, Download, Play, Terminal as TerminalIcon } from 'lucide-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { useTauriAPI } from '../hooks/useTauriAPI';

interface Props {
  api: ReturnType<typeof useTauriAPI>;
}

export default function CodingCanvas({ api }: Props) {
  const [code, setCode] = useState('// Write your code here...\n\n');
  const [language, setLanguage] = useState('javascript');
  const [filename, setFilename] = useState('untitled.js');
  const [sandboxId, setSandboxId] = useState<string | null>(null);
  const [output, setOutput] = useState<string>('');
  const [isRunning, setIsRunning] = useState(false);

  // Create sandbox on mount
  useEffect(() => {
    const createSandbox = async () => {
      try {
        const result = await api.createSandbox('coding-canvas', {
          cpu_limit: 2,
          memory_limit_mb: 512,
          disk_limit_mb: 1024,
          timeout_seconds: 30,
        });
        setSandboxId(result.sandbox_id);
        setOutput(`Sandbox created: ${result.sandbox_id}\n`);
      } catch (err) {
        console.error('Failed to create sandbox:', err);
        setOutput(`Error creating sandbox: ${err}\n`);
      }
    };
    createSandbox();
  }, [api]);

  const handleSave = () => {
    console.log('Saving file:', filename);
    // Save to local storage for now
    localStorage.setItem(`canvas_${filename}`, code);
    setOutput((prev) => prev + `Saved to local storage: ${filename}\n`);
  };

  const handleDownload = () => {
    const blob = new Blob([code], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleRun = async () => {
    if (!sandboxId) {
      setOutput((prev) => prev + 'Error: Sandbox not initialized\n');
      return;
    }

    setIsRunning(true);
    setOutput((prev) => prev + `\n--- Running ${filename} ---\n`);

    try {
      const result = await api.executeInSandbox(sandboxId, code, language);
      setOutput(
        (prev) =>
          prev +
          result.output +
          `\n--- Completed in ${result.execution_time_ms}ms (exit code: ${result.exit_code}) ---\n`
      );
    } catch (err) {
      setOutput((prev) => prev + `Error: ${err}\n`);
    } finally {
      setIsRunning(false);
    }
  };

  return (
    <div className="card h-[calc(100vh-12rem)]">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <Code size={20} />
          <input
            type="text"
            value={filename}
            onChange={(e) => setFilename(e.target.value)}
            className="bg-gray-800 border border-gray-700 rounded px-3 py-1 text-sm font-mono"
          />
          <select
            value={language}
            onChange={(e) => setLanguage(e.target.value)}
            className="bg-gray-800 border border-gray-700 rounded px-3 py-1 text-sm"
          >
            <option value="javascript">JavaScript</option>
            <option value="typescript">TypeScript</option>
            <option value="python">Python</option>
            <option value="rust">Rust</option>
            <option value="go">Go</option>
            <option value="java">Java</option>
          </select>
        </div>

        <div className="flex gap-2">
          <button
            onClick={handleRun}
            className="btn btn-primary flex items-center gap-2"
            disabled={!sandboxId || isRunning}
          >
            <Play size={16} />
            {isRunning ? 'Running...' : 'Run in Sandbox'}
          </button>
          <button onClick={handleSave} className="btn btn-secondary flex items-center gap-2">
            <Save size={16} />
            Save
          </button>
          <button onClick={handleDownload} className="btn btn-secondary flex items-center gap-2">
            <Download size={16} />
            Download
          </button>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4 h-[calc(100%-4rem)]">
        {/* Editor */}
        <div className="flex flex-col">
          <div className="text-xs text-gray-500 mb-2 px-1">Editor</div>
          <textarea
            value={code}
            onChange={(e) => setCode(e.target.value)}
            className="flex-1 bg-gray-800 border border-gray-700 rounded-lg p-4 font-mono text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary-500"
            spellCheck={false}
          />
        </div>

        {/* Output */}
        <div className="flex flex-col">
          <div className="text-xs text-gray-500 mb-2 px-1 flex items-center gap-2">
            <TerminalIcon size={12} />
            Output
          </div>
          <div className="flex-1 bg-gray-900 border border-gray-700 rounded-lg p-4 overflow-auto">
            <pre className="font-mono text-sm text-gray-300 whitespace-pre-wrap">{output || 'No output yet. Run your code to see results here.'}</pre>
          </div>
        </div>
      </div>
    </div>
  );
}
