import { useCallback } from 'react';
import { useDropzone } from 'react-dropzone';
import { Upload, File, CheckCircle, Loader } from 'lucide-react';
import { Document } from '../types';

interface Props {
  documents: Document[];
  onUpload: (files: File[]) => Promise<void>;
}

export default function DocumentUpload({ documents, onUpload }: Props) {
  const onDrop = useCallback(async (acceptedFiles: File[]) => {
    await onUpload(acceptedFiles);
  }, [onUpload]);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'text/*': ['.txt', '.md', '.json', '.csv'],
      'application/pdf': ['.pdf'],
      'application/vnd.openxmlformats-officedocument.wordprocessingml.document': ['.docx'],
    },
  });

  return (
    <div className="card">
      <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
        <Upload size={20} />
        RAG Document Upload
      </h2>

      <div
        {...getRootProps()}
        className={`dropzone ${isDragActive ? 'dropzone-active' : ''}`}
      >
        <input {...getInputProps()} />
        <div className="text-center">
          <Upload size={48} className="mx-auto mb-4 text-gray-500" />
          {isDragActive ? (
            <p className="text-primary-400 font-medium">Drop files here...</p>
          ) : (
            <>
              <p className="text-gray-300 font-medium mb-2">
                Drag & drop documents here
              </p>
              <p className="text-gray-500 text-sm">
                or click to browse
              </p>
              <p className="text-gray-600 text-xs mt-2">
                Supports: TXT, MD, PDF, DOCX, JSON, CSV
              </p>
            </>
          )}
        </div>
      </div>

      {documents.length > 0 && (
        <div className="mt-6">
          <h3 className="text-sm font-semibold text-gray-400 mb-3">
            Uploaded Documents ({documents.length})
          </h3>
          <div className="space-y-2">
            {documents.map((doc) => (
              <div
                key={doc.id}
                className="flex items-center gap-3 p-3 bg-gray-800 rounded-lg"
              >
                <File size={16} className="text-gray-400" />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">{doc.filename}</p>
                  <p className="text-xs text-gray-500">
                    {(doc.size / 1024).toFixed(1)} KB
                    {doc.chunk_count && ` • ${doc.chunk_count} chunks`}
                  </p>
                </div>
                {doc.indexed ? (
                  <CheckCircle size={16} className="text-success-500 flex-shrink-0" />
                ) : (
                  <Loader size={16} className="text-primary-500 animate-spin flex-shrink-0" />
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
