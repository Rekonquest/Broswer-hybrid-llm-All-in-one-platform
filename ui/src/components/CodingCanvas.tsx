import { useState } from 'react';
import { Code, Save, Download, Play } from 'lucide-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';

export default function CodingCanvas() {
  const [code, setCode] = useState('// Write your code here...\n\n');
  const [language, setLanguage] = useState('javascript');
  const [filename, setFilename] = useState('untitled.js');

  const handleSave = () => {
    console.log('Saving file:', filename);
    // TODO: Implement save to sandbox
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

  const handleRun = () => {
    console.log('Running code in sandbox');
    // TODO: Implement sandbox execution
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
          <button onClick={handleRun} className="btn btn-primary flex items-center gap-2">
            <Play size={16} />
            Run in Sandbox
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

        {/* Preview */}
        <div className="flex flex-col">
          <div className="text-xs text-gray-500 mb-2 px-1">Preview</div>
          <div className="flex-1 border border-gray-700 rounded-lg overflow-auto">
            <SyntaxHighlighter
              language={language}
              style={vscDarkPlus}
              customStyle={{
                margin: 0,
                padding: '1rem',
                background: '#1f2937',
                fontSize: '0.875rem',
              }}
              showLineNumbers
            >
              {code}
            </SyntaxHighlighter>
          </div>
        </div>
      </div>
    </div>
  );
}
