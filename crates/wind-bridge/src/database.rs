use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

use crate::types::{GitSha, NodeId, WindOid};

pub struct MappingDatabase {
    conn: Connection,
}

impl MappingDatabase {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sha_oid_mapping (
                git_sha TEXT PRIMARY KEY,
                wind_oid TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS node_path_mapping (
                node_id INTEGER PRIMARY KEY,
                current_path TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS path_history (
                node_id INTEGER NOT NULL,
                path TEXT NOT NULL,
                git_sha TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (node_id) REFERENCES node_path_mapping(node_id)
            );

            CREATE INDEX IF NOT EXISTS idx_wind_oid ON sha_oid_mapping(wind_oid);
            CREATE INDEX IF NOT EXISTS idx_node_path ON node_path_mapping(current_path);
            CREATE INDEX IF NOT EXISTS idx_path_history_node ON path_history(node_id);
            "#,
        )?;
        Ok(())
    }

    pub fn insert_mapping(&self, git_sha: &GitSha, wind_oid: &WindOid) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO sha_oid_mapping (git_sha, wind_oid, created_at) VALUES (?1, ?2, ?3)",
            params![git_sha.0, wind_oid.0, now],
        )?;
        Ok(())
    }

    pub fn get_wind_oid(&self, git_sha: &GitSha) -> Result<Option<WindOid>> {
        let mut stmt = self
            .conn
            .prepare("SELECT wind_oid FROM sha_oid_mapping WHERE git_sha = ?1")?;
        let result = stmt
            .query_row(params![git_sha.0], |row| row.get::<_, String>(0))
            .optional()?
            .map(WindOid);
        Ok(result)
    }

    pub fn get_git_sha(&self, wind_oid: &WindOid) -> Result<Option<GitSha>> {
        let mut stmt = self
            .conn
            .prepare("SELECT git_sha FROM sha_oid_mapping WHERE wind_oid = ?1")?;
        let result = stmt
            .query_row(params![wind_oid.0], |row| row.get::<_, String>(0))
            .optional()?
            .map(GitSha);
        Ok(result)
    }

    pub fn insert_node_mapping(&self, node_id: &NodeId, path: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO node_path_mapping (node_id, current_path, updated_at) VALUES (?1, ?2, ?3)",
            params![node_id.0, path, now],
        )?;
        Ok(())
    }

    pub fn get_node_id(&self, path: &str) -> Result<Option<NodeId>> {
        let mut stmt = self
            .conn
            .prepare("SELECT node_id FROM node_path_mapping WHERE current_path = ?1")?;
        let result = stmt
            .query_row(params![path], |row| row.get::<_, u64>(0))
            .optional()?
            .map(NodeId);
        Ok(result)
    }

    pub fn get_node_path(&self, node_id: &NodeId) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT current_path FROM node_path_mapping WHERE node_id = ?1")?;
        let result = stmt
            .query_row(params![node_id.0], |row| row.get::<_, String>(0))
            .optional()?;
        Ok(result)
    }

    pub fn add_path_history(
        &self,
        node_id: &NodeId,
        path: &str,
        git_sha: &GitSha,
        timestamp: i64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO path_history (node_id, path, git_sha, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![node_id.0, path, git_sha.0, timestamp],
        )?;
        Ok(())
    }

    pub fn get_next_node_id(&self) -> Result<NodeId> {
        let mut stmt = self
            .conn
            .prepare("SELECT COALESCE(MAX(node_id), 0) + 1 FROM node_path_mapping")?;
        let id = stmt.query_row([], |row| row.get::<_, u64>(0))?;
        Ok(NodeId(id))
    }
}
