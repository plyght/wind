use anyhow::Result;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;
use std::sync::Arc;

use crate::object_store::ObjectStore;

#[derive(Debug, Clone, PartialEq)]
pub enum DiffType {
    Text { hunks: Vec<DiffHunk> },
    Binary { old_size: u64, new_size: u64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffLine {
    pub change: LineChange,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineChange {
    Added,
    Removed,
    Unchanged,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: PathBuf,
    pub old_oid: Option<String>,
    pub new_oid: Option<String>,
    pub diff_type: DiffType,
}

pub struct DiffEngine {
    storage: Arc<ObjectStore>,
}

impl DiffEngine {
    pub fn new(storage: Arc<ObjectStore>) -> Self {
        Self { storage }
    }

    pub fn diff_files(&self, old_oid: Option<&str>, new_oid: Option<&str>) -> Result<DiffType> {
        match (old_oid, new_oid) {
            (Some(old), Some(new)) if old != new => self.diff_blobs(old, new),
            (Some(_), None) | (None, Some(_)) => Ok(DiffType::Text { hunks: vec![] }),
            _ => Ok(DiffType::Text { hunks: vec![] }),
        }
    }

    fn diff_blobs(&self, old_oid: &str, new_oid: &str) -> Result<DiffType> {
        let old_content = self.storage.read(old_oid)?;
        let new_content = self.storage.read(new_oid)?;

        if self.is_binary(&old_content) || self.is_binary(&new_content) {
            return Ok(DiffType::Binary {
                old_size: old_content.len() as u64,
                new_size: new_content.len() as u64,
            });
        }

        let old_text = String::from_utf8_lossy(&old_content);
        let new_text = String::from_utf8_lossy(&new_content);

        let diff = TextDiff::from_lines(&old_text, &new_text);
        let mut hunks = Vec::new();

        for group in diff.grouped_ops(3) {
            let mut lines = Vec::new();
            let mut old_start = 0;
            let mut new_start = 0;
            let mut old_count = 0;
            let mut new_count = 0;

            for op in &group {
                if old_start == 0 {
                    old_start = op.old_range().start;
                    new_start = op.new_range().start;
                }
                old_count += op.old_range().len();
                new_count += op.new_range().len();

                for change in diff.iter_changes(op) {
                    let line_change = match change.tag() {
                        ChangeTag::Insert => LineChange::Added,
                        ChangeTag::Delete => LineChange::Removed,
                        ChangeTag::Equal => LineChange::Unchanged,
                    };

                    lines.push(DiffLine {
                        change: line_change,
                        content: change.value().to_string(),
                    });
                }
            }

            hunks.push(DiffHunk {
                old_start,
                old_count,
                new_start,
                new_count,
                lines,
            });
        }

        Ok(DiffType::Text { hunks })
    }

    fn is_binary(&self, content: &[u8]) -> bool {
        content.iter().take(8000).any(|&b| b == 0)
    }
}
