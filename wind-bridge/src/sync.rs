use anyhow::Result;
use git2::Repository;
use std::path::Path;
use tracing::{info, warn};

use crate::exporter::GitExporter;
use crate::importer::GitImporter;
use crate::types::Changeset;

pub struct SyncStats {
    pub imported_count: usize,
    pub exported_count: usize,
    pub conflicts: usize,
}

pub fn sync_repositories<P: AsRef<Path>>(
    repo_path: P,
    wind_path: P,
    db_path: P,
) -> Result<SyncStats> {
    info!("Starting repository synchronization");

    let mut importer = GitImporter::new(&repo_path, &db_path)?;

    let new_changesets = import_new_commits(&mut importer)?;
    let imported_count = new_changesets.len();

    let exported_count = 0;

    let conflicts = detect_conflicts(&repo_path)?;

    info!(
        "Sync complete: {} imported, {} exported, {} conflicts",
        imported_count, exported_count, conflicts
    );

    Ok(SyncStats {
        imported_count,
        exported_count,
        conflicts,
    })
}

fn import_new_commits(importer: &mut GitImporter) -> Result<Vec<Changeset>> {
    let changesets = importer.import_all()?;
    Ok(changesets)
}

fn export_wind_changes<P: AsRef<Path>>(
    _exporter: &mut GitExporter,
    _wind_path: P,
) -> Result<usize> {
    Ok(0)
}

fn detect_conflicts<P: AsRef<Path>>(repo_path: P) -> Result<usize> {
    let repo = Repository::open(repo_path)?;
    let index = repo.index()?;

    let conflicts = index.conflicts()?.count();

    if conflicts > 0 {
        warn!("Detected {} conflicts that need resolution", conflicts);
    }

    Ok(conflicts)
}

pub fn handle_divergence<P: AsRef<Path>>(repo_path: P, _db_path: P) -> Result<()> {
    let repo = Repository::open(&repo_path)?;
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;

    info!("Handling repository divergence at {}", head_commit.id());

    Ok(())
}
