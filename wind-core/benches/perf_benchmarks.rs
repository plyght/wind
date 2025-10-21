use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use tempfile::TempDir;
use wind_core::Repository;

fn create_test_repo(file_count: usize) -> (TempDir, Repository) {
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path()).unwrap();

    for i in 0..file_count {
        let file_path = temp.path().join(format!("file{}.txt", i));
        std::fs::write(&file_path, format!("content {}", i)).unwrap();
    }

    (temp, repo)
}

fn bench_status_small(c: &mut Criterion) {
    let (_temp, repo) = create_test_repo(10);

    c.bench_function("status_10_files", |b| {
        b.iter(|| {
            let status = repo.status().unwrap();
            black_box(status);
        })
    });
}

fn bench_status_medium(c: &mut Criterion) {
    let (_temp, repo) = create_test_repo(100);

    c.bench_function("status_100_files", |b| {
        b.iter(|| {
            let status = repo.status().unwrap();
            black_box(status);
        })
    });
}

fn bench_status_large(c: &mut Criterion) {
    let (_temp, repo) = create_test_repo(1000);

    c.bench_function("status_1000_files", |b| {
        b.iter(|| {
            let status = repo.status().unwrap();
            black_box(status);
        })
    });
}

fn bench_status_cached(c: &mut Criterion) {
    let (_temp, repo) = create_test_repo(100);

    repo.status().unwrap();

    c.bench_function("status_cached", |b| {
        b.iter(|| {
            let status = repo.status().unwrap();
            black_box(status);
        })
    });
}

fn bench_log_paginated(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path()).unwrap();

    for i in 0..50 {
        let file_path = temp.path().join(format!("file{}.txt", i));
        std::fs::write(&file_path, format!("content {}", i)).unwrap();
        repo.add(&format!("file{}.txt", i)).unwrap();
        repo.commit(&format!("Commit {}", i)).unwrap();
    }

    c.bench_function("log_paginated_20", |b| {
        b.iter(|| {
            let commits = repo.log_paginated(0, 20).unwrap();
            black_box(commits);
        })
    });
}

fn bench_diff(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path()).unwrap();

    let file_path = temp.path().join("test.txt");
    std::fs::write(&file_path, "line 1\nline 2\nline 3\n").unwrap();
    repo.add("test.txt").unwrap();
    repo.commit("Initial").unwrap();

    std::fs::write(&file_path, "line 1\nmodified\nline 3\nnew line\n").unwrap();

    c.bench_function("diff_small_file", |b| {
        b.iter(|| {
            let diff = repo.get_diff("test.txt", 3).unwrap();
            black_box(diff);
        })
    });
}

criterion_group!(
    benches,
    bench_status_small,
    bench_status_medium,
    bench_status_large,
    bench_status_cached,
    bench_log_paginated,
    bench_diff
);
criterion_main!(benches);
