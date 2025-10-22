use anyhow::Result;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
#[ignore] // Requires full git submodule setup
fn test_submodule_detection() -> Result<()> {
    let temp = TempDir::new()?;
    let main_repo = temp.path().join("main");
    let sub_repo = temp.path().join("sub");

    fs::create_dir(&main_repo)?;
    fs::create_dir(&sub_repo)?;

    Command::new("git")
        .args(["init"])
        .current_dir(&main_repo)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&main_repo)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&main_repo)
        .output()?;

    Command::new("git")
        .args(["init"])
        .current_dir(&sub_repo)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&sub_repo)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&sub_repo)
        .output()?;

    fs::write(sub_repo.join("sub.txt"), "submodule content")?;

    Command::new("git")
        .args(["add", "sub.txt"])
        .current_dir(&sub_repo)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Submodule init"])
        .current_dir(&sub_repo)
        .output()?;

    Command::new("git")
        .args([
            "submodule",
            "add",
            sub_repo.to_str().unwrap(),
            "mysubmodule",
        ])
        .current_dir(&main_repo)
        .output()?;

    let has_subs = wind::submodule::has_submodules(&main_repo)?;
    assert!(has_subs, "Should detect submodules");

    let submodules = wind::submodule::list_submodules(&main_repo)?;
    assert_eq!(submodules.len(), 1, "Should have one submodule");

    let sub = &submodules[0];
    assert_eq!(sub.name, "mysubmodule");
    assert!(sub.initialized, "Submodule should be initialized");

    Ok(())
}

#[test]
fn test_submodule_list() -> Result<()> {
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()?;

    let gitmodules = r#"[submodule "module1"]
    path = lib/module1
    url = https://github.com/example/module1
[submodule "module2"]
    path = lib/module2
    url = https://github.com/example/module2
"#;

    fs::write(repo_path.join(".gitmodules"), gitmodules)?;

    let submodules = wind::submodule::list_submodules(repo_path)?;
    assert_eq!(submodules.len(), 2, "Should parse 2 submodules");

    let module1 = submodules.iter().find(|s| s.name == "module1");
    assert!(module1.is_some(), "Should find module1");

    let m1 = module1.unwrap();
    assert_eq!(m1.path.to_string_lossy(), "lib/module1");
    assert_eq!(m1.url, "https://github.com/example/module1");
    assert!(!m1.initialized, "Should not be initialized without .git");

    let module2 = submodules.iter().find(|s| s.name == "module2");
    assert!(module2.is_some(), "Should find module2");

    Ok(())
}

#[test]
fn test_submodule_status() -> Result<()> {
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()?;

    let gitmodules = r#"[submodule "testmod"]
    path = lib/testmod
    url = https://github.com/example/testmod
"#;

    fs::write(repo_path.join(".gitmodules"), gitmodules)?;

    let submodules = wind::submodule::list_submodules(repo_path)?;
    assert_eq!(submodules.len(), 1);

    let sub = &submodules[0];
    let status = wind::submodule::get_submodule_status(repo_path, sub)?;
    assert_eq!(status, "not initialized", "Should be not initialized");

    fs::create_dir_all(repo_path.join("lib/testmod"))?;
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path.join("lib/testmod"))
        .output()?;

    let submodules = wind::submodule::list_submodules(repo_path)?;
    let sub = &submodules[0];
    let status = wind::submodule::get_submodule_status(repo_path, sub)?;
    assert_eq!(
        status, "initialized",
        "Should be initialized after git init"
    );

    Ok(())
}

#[test]
#[ignore] // Requires full git submodule setup
fn test_inside_submodule_detection() -> Result<()> {
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    fs::write(repo_path.join(".gitmodules"), "[submodule \"test\"]\n")?;

    let submodule_path = repo_path.join("lib/submodule");
    fs::create_dir_all(&submodule_path)?;

    Command::new("git")
        .args(["init"])
        .current_dir(&submodule_path)
        .output()?;

    let is_inside = wind::submodule::is_inside_submodule(&submodule_path)?;
    assert!(is_inside, "Should detect being inside submodule");

    let is_root_inside = wind::submodule::is_inside_submodule(repo_path)?;
    assert!(
        !is_root_inside,
        "Root repo should not be detected as inside submodule"
    );

    Ok(())
}

#[test]
#[ignore] // Requires full git submodule setup
fn test_repository_submodule_integration() -> Result<()> {
    let temp = TempDir::new()?;
    let main_repo = temp.path().join("main");
    let sub_repo = temp.path().join("sub");

    fs::create_dir(&main_repo)?;
    fs::create_dir(&sub_repo)?;

    Command::new("git")
        .args(["init"])
        .current_dir(&sub_repo)
        .output()?;

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&sub_repo)
        .output()?;

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&sub_repo)
        .output()?;

    fs::write(sub_repo.join("sub.txt"), "content")?;

    Command::new("git")
        .args(["add", "."])
        .current_dir(&sub_repo)
        .output()?;

    Command::new("git")
        .args(["commit", "-m", "Init sub"])
        .current_dir(&sub_repo)
        .output()?;

    let repo = wind::Repository::init(&main_repo)?;

    Command::new("git")
        .args(["submodule", "add", sub_repo.to_str().unwrap(), "mysub"])
        .current_dir(&main_repo)
        .output()?;

    let status = repo.status()?;
    assert!(
        !status.submodules.is_empty(),
        "Should detect submodules in status"
    );

    let subs = repo.list_submodules()?;
    assert_eq!(subs.len(), 1, "Should have one submodule");

    Ok(())
}
