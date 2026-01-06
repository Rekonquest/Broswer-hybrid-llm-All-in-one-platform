mod memory;
mod database;
mod embeddings;

pub use memory::ContextManagerImpl as InMemoryContextManager;
pub use database::DatabaseContextManager;
pub use embeddings::EmbeddingGenerator;

// Re-export for convenience
pub use database::DatabaseContextManager as ContextManagerImpl;
