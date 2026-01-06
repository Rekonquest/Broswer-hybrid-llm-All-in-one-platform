use common::errors::{Result, HybridLLMError};
use std::path::{Path, PathBuf};
use tracing::{info, debug};

/// File system interface for managing uploads/downloads and RAG
pub struct FileSystemInterface {
    base_path: PathBuf,
    downloads_path: PathBuf,
    uploads_path: PathBuf,
    rag_path: PathBuf,
}

impl FileSystemInterface {
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        let downloads_path = base_path.join("downloads");
        let uploads_path = base_path.join("uploads");
        let rag_path = base_path.join("rag");

        // Create directories if they don't exist
        std::fs::create_dir_all(&downloads_path)
            .map_err(|e| HybridLLMError::FileSystemError(e.to_string()))?;
        std::fs::create_dir_all(&uploads_path)
            .map_err(|e| HybridLLMError::FileSystemError(e.to_string()))?;
        std::fs::create_dir_all(&rag_path)
            .map_err(|e| HybridLLMError::FileSystemError(e.to_string()))?;

        info!("📁 File system interface initialized at {:?}", base_path);

        Ok(Self {
            base_path,
            downloads_path,
            uploads_path,
            rag_path,
        })
    }

    pub fn downloads_path(&self) -> &Path {
        &self.downloads_path
    }

    pub fn uploads_path(&self) -> &Path {
        &self.uploads_path
    }

    pub fn rag_path(&self) -> &Path {
        &self.rag_path
    }

    /// Write a file to the downloads folder
    pub async fn write_download(&self, filename: &str, content: &[u8]) -> Result<PathBuf> {
        let path = self.downloads_path.join(filename);
        tokio::fs::write(&path, content)
            .await
            .map_err(|e| HybridLLMError::FileSystemError(e.to_string()))?;

        info!("⬇️  Downloaded file: {:?}", path);
        Ok(path)
    }

    /// Read a file from the uploads folder
    pub async fn read_upload(&self, filename: &str) -> Result<Vec<u8>> {
        let path = self.uploads_path.join(filename);
        let content = tokio::fs::read(&path)
            .await
            .map_err(|e| HybridLLMError::FileSystemError(e.to_string()))?;

        info!("⬆️  Read uploaded file: {:?}", path);
        Ok(content)
    }

    /// List files in uploads folder
    pub fn list_uploads(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in walkdir::WalkDir::new(&self.uploads_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }

        Ok(files)
    }

    /// Watch uploads folder for changes (for RAG indexing)
    pub async fn watch_uploads<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(PathBuf) + Send + 'static,
    {
        // TODO: Implement file watcher using notify crate
        debug!("👀 Watching uploads folder for changes");
        Ok(())
    }
}
