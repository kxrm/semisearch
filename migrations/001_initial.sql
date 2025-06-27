-- Initial database schema for semantic search
-- Phase 2: Storage Layer

-- Files table to track indexed files
CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE NOT NULL,
    hash TEXT NOT NULL,
    modified_at INTEGER NOT NULL,
    size_bytes INTEGER NOT NULL,
    indexed_at INTEGER NOT NULL
);

-- Chunks table to store text content with metadata
CREATE TABLE IF NOT EXISTS chunks (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL,
    line_number INTEGER NOT NULL,
    start_char INTEGER NOT NULL,
    end_char INTEGER NOT NULL,
    content TEXT NOT NULL,
    embedding BLOB,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- Query cache for improved performance
CREATE TABLE IF NOT EXISTS query_cache (
    query_hash TEXT PRIMARY KEY,
    results TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    access_count INTEGER DEFAULT 1,
    last_accessed INTEGER NOT NULL
);

-- Indexes for better performance
CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
CREATE INDEX IF NOT EXISTS idx_files_modified ON files(modified_at);
CREATE INDEX IF NOT EXISTS idx_chunks_file_id ON chunks(file_id);
CREATE INDEX IF NOT EXISTS idx_chunks_content ON chunks(content);
CREATE INDEX IF NOT EXISTS idx_chunks_line_number ON chunks(line_number);
CREATE INDEX IF NOT EXISTS idx_query_cache_accessed ON query_cache(last_accessed);
