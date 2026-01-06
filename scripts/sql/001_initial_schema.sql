-- Initial schema for Hybrid LLM Platform
-- Requires PostgreSQL 14+ with pgvector extension

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Conversations table
CREATE TABLE IF NOT EXISTS conversations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    persistent BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB
);

-- Messages table
CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB,
    CONSTRAINT messages_conversation_fk FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

-- Index for faster message retrieval
CREATE INDEX IF NOT EXISTS idx_messages_conversation
    ON messages(conversation_id, timestamp DESC);

-- Documents table for RAG
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    filename VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    uploaded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    version INTEGER NOT NULL DEFAULT 1,
    checksum VARCHAR(64) NOT NULL,  -- SHA256 hash
    metadata JSONB,
    llm_visibility TEXT[] DEFAULT '{}'  -- Array of LLM IDs that can see this document
);

-- Document chunks for RAG (split documents into smaller pieces)
CREATE TABLE IF NOT EXISTS document_chunks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    chunk_text TEXT NOT NULL,
    embedding VECTOR(384),  -- sentence-transformers/all-MiniLM-L6-v2 dimension
    metadata JSONB,
    CONSTRAINT chunks_document_fk FOREIGN KEY (document_id) REFERENCES documents(id)
);

-- Index for vector similarity search (HNSW algorithm)
CREATE INDEX IF NOT EXISTS idx_chunks_embedding
    ON document_chunks USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

-- Index for document lookup
CREATE INDEX IF NOT EXISTS idx_chunks_document
    ON document_chunks(document_id, chunk_index);

-- LLM context storage
CREATE TABLE IF NOT EXISTS llm_contexts (
    llm_id VARCHAR(255) NOT NULL,
    context_key VARCHAR(255) NOT NULL,
    context_value JSONB NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (llm_id, context_key)
);

-- Global context storage
CREATE TABLE IF NOT EXISTS global_context (
    context_key VARCHAR(255) PRIMARY KEY,
    context_value JSONB NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Audit log table
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    llm_id VARCHAR(255),
    action VARCHAR(255) NOT NULL,
    details JSONB NOT NULL,
    approved BOOLEAN NOT NULL,
    reason TEXT
);

-- Index for audit log queries
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_llm ON audit_log(llm_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_approved ON audit_log(approved, timestamp DESC);

-- Lockdown events table
CREATE TABLE IF NOT EXISTS lockdown_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    triggered_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    released_at TIMESTAMP WITH TIME ZONE,
    reason JSONB NOT NULL,
    triggered_by_llm VARCHAR(255),
    released_by_user VARCHAR(255)
);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for updated_at
CREATE TRIGGER update_conversations_updated_at
    BEFORE UPDATE ON conversations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_llm_contexts_updated_at
    BEFORE UPDATE ON llm_contexts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_global_context_updated_at
    BEFORE UPDATE ON global_context
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- View for recent audit log
CREATE OR REPLACE VIEW recent_audit_log AS
SELECT
    id,
    timestamp,
    llm_id,
    action,
    details,
    approved,
    reason
FROM audit_log
ORDER BY timestamp DESC
LIMIT 1000;

-- View for document statistics
CREATE OR REPLACE VIEW document_stats AS
SELECT
    d.id,
    d.filename,
    d.uploaded_at,
    COUNT(dc.id) as chunk_count,
    d.llm_visibility
FROM documents d
LEFT JOIN document_chunks dc ON d.id = dc.document_id
GROUP BY d.id, d.filename, d.uploaded_at, d.llm_visibility;

-- Comments for documentation
COMMENT ON TABLE conversations IS 'Stores conversation history for LLM interactions';
COMMENT ON TABLE documents IS 'Uploaded documents for RAG (Retrieval-Augmented Generation)';
COMMENT ON TABLE document_chunks IS 'Document chunks with vector embeddings for semantic search';
COMMENT ON TABLE audit_log IS 'Complete audit trail of all LLM actions and permission requests';
COMMENT ON COLUMN document_chunks.embedding IS 'Vector embedding using sentence-transformers/all-MiniLM-L6-v2 (384 dimensions)';
COMMENT ON INDEX idx_chunks_embedding IS 'HNSW index for fast approximate nearest neighbor search';
