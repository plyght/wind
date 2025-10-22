use anyhow::Result;
use std::fs;
use tempfile::TempDir;
use wind_bridge::{GitExporter, GitImporter, MappingDatabase};

#[test]
fn test_import_git_commits() -> Result<()> {
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    let repo = git2::Repository::init(repo_path)?;

    let mut index = repo.index()?;
    fs::write(repo_path.join("test.txt"), "Hello World")?;
    index.add_path(std::path::Path::new("test.txt"))?;
    index.write()?;

    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let sig = git2::Signature::now("Test", "test@example.com")?;

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    let db_path = repo_path.join(".wind/bridge/mapping.db");
    fs::create_dir_all(repo_path.join(".wind/bridge"))?;

    let mut importer = GitImporter::new(repo_path, &db_path)?;
    let changesets = importer.import_all()?;

    assert_eq!(changesets.len(), 1);
    assert_eq!(changesets[0].message, "Initial commit");
    assert_eq!(changesets[0].ops.len(), 1);

    Ok(())
}

#[test]
fn test_database_mapping() -> Result<()> {
    let temp = TempDir::new()?;
    let db_path = temp.path().join("mapping.db");

    let db = MappingDatabase::open(&db_path)?;

    let git_sha = wind_bridge::GitSha("abc123".to_string());
    let wind_oid = wind_bridge::WindOid("wabc123".to_string());

    db.insert_mapping(&git_sha, &wind_oid)?;

    let retrieved = db.get_wind_oid(&git_sha)?;
    assert_eq!(retrieved, Some(wind_oid.clone()));

    let retrieved_sha = db.get_git_sha(&wind_oid)?;
    assert_eq!(retrieved_sha, Some(git_sha));

    Ok(())
}

#[test]
fn test_node_id_tracking() -> Result<()> {
    let temp = TempDir::new()?;
    let db_path = temp.path().join("mapping.db");

    let db = MappingDatabase::open(&db_path)?;

    let node_id = db.get_next_node_id()?;
    assert_eq!(node_id.0, 1);

    db.insert_node_mapping(&node_id, "src/main.rs")?;

    let retrieved_path = db.get_node_path(&node_id)?;
    assert_eq!(retrieved_path, Some("src/main.rs".to_string()));

    let retrieved_node = db.get_node_id("src/main.rs")?;
    assert_eq!(retrieved_node, Some(node_id));

    Ok(())
}
