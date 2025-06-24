use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Database connection and operations
pub struct Database {
    conn: Connection,
}

/// File record from the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: i64,
    pub path: String,
    pub hash: String,
    pub modified_at: DateTime<Utc>,
    pub size_bytes: i64,
    pub indexed_at: DateTime<Utc>,
}

/// Chunk record from the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub id: i64,
    pub file_id: i64,
    pub file_path: String,
    pub line_number: usize,
    pub start_char: usize,
    pub end_char: usize,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
}

impl Database {
    /// Create a new database connection and initialize schema
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        // Load schema
        let schema = include_str!("../../migrations/001_initial.sql");
        conn.execute_batch(schema)?;

        Ok(Self { conn })
    }

    /// Insert or update a file record
    pub fn insert_file(
        &self,
        path: &str,
        hash: &str,
        modified_at: DateTime<Utc>,
        size_bytes: i64,
    ) -> Result<i64> {
        let mut stmt = self.conn.prepare_cached(
            "INSERT OR REPLACE INTO files (path, hash, modified_at, size_bytes, indexed_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        let indexed_at = Utc::now();

        stmt.execute(params![
            path,
            hash,
            modified_at.timestamp(),
            size_bytes,
            indexed_at.timestamp()
        ])?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Insert a chunk for a file
    pub fn insert_chunk(
        &self,
        file_id: i64,
        line_number: usize,
        start_char: usize,
        end_char: usize,
        content: &str,
        embedding: Option<&[f32]>,
    ) -> Result<i64> {
        let embedding_bytes = embedding.map(|e| {
            e.iter()
                .flat_map(|&f| f.to_le_bytes().to_vec())
                .collect::<Vec<u8>>()
        });

        let mut stmt = self.conn.prepare_cached(
            "INSERT INTO chunks (file_id, line_number, start_char, end_char, content, embedding) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;

        stmt.execute(params![
            file_id,
            line_number as i64,
            start_char as i64,
            end_char as i64,
            content,
            embedding_bytes
        ])?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Check if a file needs reindexing
    pub fn needs_reindexing(&self, path: &str, current_hash: &str) -> Result<bool> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT hash FROM files WHERE path = ?1")?;

        match stmt.query_row(params![path], |row| {
            let stored_hash: String = row.get(0)?;
            Ok(stored_hash)
        }) {
            Ok(stored_hash) => Ok(stored_hash != current_hash),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(true), // File not indexed yet
            Err(e) => Err(e.into()),
        }
    }

    /// Get file record by path
    pub fn get_file_by_path(&self, path: &str) -> Result<Option<FileRecord>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, path, hash, modified_at, size_bytes, indexed_at FROM files WHERE path = ?1",
        )?;

        match stmt.query_row(params![path], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                path: row.get(1)?,
                hash: row.get(2)?,
                modified_at: DateTime::from_timestamp(row.get::<_, i64>(3)?, 0).unwrap_or_default(),
                size_bytes: row.get(4)?,
                indexed_at: DateTime::from_timestamp(row.get::<_, i64>(5)?, 0).unwrap_or_default(),
            })
        }) {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Search chunks by content
    pub fn search_chunks(&self, query: &str, limit: usize) -> Result<Vec<ChunkRecord>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT c.id, c.file_id, f.path, c.line_number, c.start_char, c.end_char, c.content, c.embedding
             FROM chunks c
             JOIN files f ON c.file_id = f.id
             WHERE c.content LIKE ?1
             ORDER BY c.id
             LIMIT ?2"
        )?;

        let search_pattern = format!("%{}%", query);
        let rows = stmt.query_map(params![search_pattern, limit], |row| {
            let embedding_bytes: Option<Vec<u8>> = row.get(7)?;
            let embedding = embedding_bytes.map(|bytes| {
                bytes
                    .chunks(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect()
            });

            Ok(ChunkRecord {
                id: row.get(0)?,
                file_id: row.get(1)?,
                file_path: row.get(2)?,
                line_number: row.get::<_, i64>(3)? as usize,
                start_char: row.get::<_, i64>(4)? as usize,
                end_char: row.get::<_, i64>(5)? as usize,
                content: row.get(6)?,
                embedding,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Get all chunks for a file
    pub fn get_chunks_for_file(&self, file_id: i64) -> Result<Vec<ChunkRecord>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT c.id, c.file_id, f.path, c.line_number, c.start_char, c.end_char, c.content, c.embedding
             FROM chunks c
             JOIN files f ON c.file_id = f.id
             WHERE c.file_id = ?1
             ORDER BY c.line_number"
        )?;

        let rows = stmt.query_map(params![file_id], |row| {
            let embedding_bytes: Option<Vec<u8>> = row.get(7)?;
            let embedding = embedding_bytes.map(|bytes| {
                bytes
                    .chunks(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect()
            });

            Ok(ChunkRecord {
                id: row.get(0)?,
                file_id: row.get(1)?,
                file_path: row.get(2)?,
                line_number: row.get::<_, i64>(3)? as usize,
                start_char: row.get::<_, i64>(4)? as usize,
                end_char: row.get::<_, i64>(5)? as usize,
                content: row.get(6)?,
                embedding,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Remove file and all its chunks
    pub fn remove_file(&self, path: &str) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare_cached("DELETE FROM files WHERE path = ?1")?;
        stmt.execute(params![path])?;
        Ok(())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let file_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))?;

        let chunk_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))?;

        let total_size: i64 = self
            .conn
            .query_row("SELECT SUM(size_bytes) FROM files", [], |row| row.get(0))
            .unwrap_or(0);

        Ok(DatabaseStats {
            file_count: file_count as usize,
            chunk_count: chunk_count as usize,
            total_size_bytes: total_size as u64,
        })
    }

    /// Clean up old query cache entries
    pub fn cleanup_query_cache(&self, max_entries: usize) -> Result<()> {
        let mut stmt = self.conn.prepare_cached(
            "DELETE FROM query_cache WHERE query_hash NOT IN (
                SELECT query_hash FROM query_cache 
                ORDER BY last_accessed DESC 
                LIMIT ?1
            )",
        )?;

        stmt.execute(params![max_entries])?;
        Ok(())
    }
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub file_count: usize,
    pub chunk_count: usize,
    pub total_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_db() -> (Database, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).unwrap();
        (db, temp_file)
    }

    #[test]
    fn test_database_creation() {
        let (db, _temp_file) = create_test_db();
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.file_count, 0);
        assert_eq!(stats.chunk_count, 0);
    }

    #[test]
    fn test_file_operations() {
        let (db, _temp_file) = create_test_db();
        let now = Utc::now();

        // Insert file
        let file_id = db
            .insert_file("/test/path.txt", "hash123", now, 1024)
            .unwrap();
        assert!(file_id > 0);

        // Check if file exists
        let file_record = db.get_file_by_path("/test/path.txt").unwrap();
        assert!(file_record.is_some());

        let record = file_record.unwrap();
        assert_eq!(record.path, "/test/path.txt");
        assert_eq!(record.hash, "hash123");
        assert_eq!(record.size_bytes, 1024);
    }

    #[test]
    fn test_needs_reindexing() {
        let (db, _temp_file) = create_test_db();
        let now = Utc::now();

        // File not indexed yet
        assert!(db.needs_reindexing("/test/path.txt", "hash123").unwrap());

        // Insert file
        db.insert_file("/test/path.txt", "hash123", now, 1024)
            .unwrap();

        // Same hash - no reindexing needed
        assert!(!db.needs_reindexing("/test/path.txt", "hash123").unwrap());

        // Different hash - reindexing needed
        assert!(db.needs_reindexing("/test/path.txt", "hash456").unwrap());
    }

    #[test]
    fn test_chunk_operations() {
        let (db, _temp_file) = create_test_db();
        let now = Utc::now();

        // Insert file first
        let file_id = db
            .insert_file("/test/path.txt", "hash123", now, 1024)
            .unwrap();

        // Insert chunk
        let chunk_id = db
            .insert_chunk(file_id, 1, 0, 10, "test content", None)
            .unwrap();
        assert!(chunk_id > 0);

        // Get chunks for file
        let chunks = db.get_chunks_for_file(file_id).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "test content");
        assert_eq!(chunks[0].line_number, 1);
    }

    #[test]
    fn test_search_chunks() {
        let (db, _temp_file) = create_test_db();
        let now = Utc::now();

        // Insert file and chunks
        let file_id = db
            .insert_file("/test/path.txt", "hash123", now, 1024)
            .unwrap();
        db.insert_chunk(file_id, 1, 0, 10, "hello world", None)
            .unwrap();
        db.insert_chunk(file_id, 2, 11, 20, "goodbye world", None)
            .unwrap();
        db.insert_chunk(file_id, 3, 21, 30, "test content", None)
            .unwrap();

        // Search for chunks
        let results = db.search_chunks("world", 10).unwrap();
        assert_eq!(results.len(), 2);

        let results = db.search_chunks("test", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "test content");
    }

    #[test]
    fn test_database_stats() {
        let (db, _temp_file) = create_test_db();
        let now = Utc::now();

        // Initially empty
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.file_count, 0);
        assert_eq!(stats.chunk_count, 0);
        assert_eq!(stats.total_size_bytes, 0);

        // Add some data
        let file_id = db
            .insert_file("/test/path.txt", "hash123", now, 1024)
            .unwrap();
        db.insert_chunk(file_id, 1, 0, 10, "test content", None)
            .unwrap();

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.chunk_count, 1);
        assert_eq!(stats.total_size_bytes, 1024);
    }

    #[test]
    fn test_embedding_storage() {
        let (db, _temp_file) = create_test_db();
        let now = Utc::now();

        let file_id = db
            .insert_file("/test/path.txt", "hash123", now, 1024)
            .unwrap();
        let embedding = vec![0.1, 0.2, 0.3, 0.4];

        // Insert chunk with embedding
        db.insert_chunk(file_id, 1, 0, 10, "test content", Some(&embedding))
            .unwrap();

        // Retrieve and verify
        let chunks = db.get_chunks_for_file(file_id).unwrap();
        assert_eq!(chunks.len(), 1);

        let stored_embedding = chunks[0].embedding.as_ref().unwrap();
        assert_eq!(stored_embedding.len(), 4);
        assert!((stored_embedding[0] - 0.1).abs() < 0.001);
        assert!((stored_embedding[1] - 0.2).abs() < 0.001);
        assert!((stored_embedding[2] - 0.3).abs() < 0.001);
        assert!((stored_embedding[3] - 0.4).abs() < 0.001);
    }
}
