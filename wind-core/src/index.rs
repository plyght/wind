use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub path: PathBuf,
    pub node_id: String,
    pub oid: String,
    pub mtime: u64,
    pub size: u64,
    pub permissions: u32,
}

pub struct Index {
    conn: Connection,
}

impl Index {
    pub fn new(wind_dir: &Path) -> Result<Self> {
        let db_path = wind_dir.join("index.db");
        let conn = Connection::open(&db_path).context("Failed to open index database")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS paths (
                path TEXT PRIMARY KEY,
                node_id TEXT NOT NULL,
                oid TEXT NOT NULL,
                mtime INTEGER NOT NULL,
                size INTEGER NOT NULL,
                permissions INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_node_id ON paths(node_id)",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn add(&mut self, entry: &IndexEntry) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO paths (path, node_id, oid, mtime, size, permissions) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.path.to_string_lossy().as_ref(),
                &entry.node_id,
                &entry.oid,
                entry.mtime as i64,
                entry.size as i64,
                entry.permissions as i64,
            ],
        )?;
        Ok(())
    }

    pub fn remove(&mut self, path: &Path) -> Result<()> {
        self.conn.execute(
            "DELETE FROM paths WHERE path = ?1",
            params![path.to_string_lossy().as_ref()],
        )?;
        Ok(())
    }

    pub fn update(&mut self, entry: &IndexEntry) -> Result<()> {
        self.add(entry)
    }

    pub fn lookup(&self, path: &Path) -> Result<Option<IndexEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT path, node_id, oid, mtime, size, permissions FROM paths WHERE path = ?1",
        )?;

        let mut rows = stmt.query(params![path.to_string_lossy().as_ref()])?;

        if let Some(row) = rows.next()? {
            Ok(Some(IndexEntry {
                path: PathBuf::from(row.get::<_, String>(0)?),
                node_id: row.get(1)?,
                oid: row.get(2)?,
                mtime: row.get::<_, i64>(3)? as u64,
                size: row.get::<_, i64>(4)? as u64,
                permissions: row.get::<_, i64>(5)? as u32,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn lookup_by_node_id(&self, node_id: &str) -> Result<Vec<IndexEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT path, node_id, oid, mtime, size, permissions FROM paths WHERE node_id = ?1",
        )?;

        let rows = stmt.query_map(params![node_id], |row| {
            Ok(IndexEntry {
                path: PathBuf::from(row.get::<_, String>(0)?),
                node_id: row.get(1)?,
                oid: row.get(2)?,
                mtime: row.get::<_, i64>(3)? as u64,
                size: row.get::<_, i64>(4)? as u64,
                permissions: row.get::<_, i64>(5)? as u32,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn list_all(&self) -> Result<Vec<IndexEntry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, node_id, oid, mtime, size, permissions FROM paths")?;

        let rows = stmt.query_map([], |row| {
            Ok(IndexEntry {
                path: PathBuf::from(row.get::<_, String>(0)?),
                node_id: row.get(1)?,
                oid: row.get(2)?,
                mtime: row.get::<_, i64>(3)? as u64,
                size: row.get::<_, i64>(4)? as u64,
                permissions: row.get::<_, i64>(5)? as u32,
            })
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.conn.execute("DELETE FROM paths", [])?;
        Ok(())
    }
}

pub fn get_mtime(path: &Path) -> Result<u64> {
    let metadata = std::fs::metadata(path)?;
    let mtime = metadata
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    Ok(mtime)
}
