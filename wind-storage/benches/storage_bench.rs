use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tempfile::TempDir;
use wind_storage::object_store::{Object, ObjectType};
use wind_storage::{ChunkStore, Chunker, FileSystemStore, ObjectStore, Oid};

fn bench_oid_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("oid_hashing");

    for size in [1024, 4096, 16384, 65536].iter() {
        let data = vec![0u8; *size];
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| Oid::hash_bytes(black_box(&data)));
        });
    }

    group.finish();
}

fn bench_chunking(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunking");

    let chunker = Chunker::default();

    for size in [64 * 1024, 256 * 1024, 1024 * 1024].iter() {
        let data = vec![0u8; *size];
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| chunker.chunk_bytes(black_box(&data)));
        });
    }

    group.finish();
}

fn bench_chunk_store_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_store_write");

    let temp = TempDir::new().unwrap();
    let mut store = ChunkStore::new(temp.path().join("chunks")).unwrap();
    let chunker = Chunker::default();

    for size in [4096, 65536].iter() {
        let data = vec![0u8; *size];
        let chunks = chunker.chunk_bytes(&data);

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                for chunk in &chunks {
                    store.write_chunk(black_box(chunk)).unwrap();
                }
            });
        });
    }

    group.finish();
}

fn bench_object_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("object_store");

    let temp = TempDir::new().unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let store = FileSystemStore::new(temp.path().join("objects")).unwrap();

    for size in [1024, 65536].iter() {
        let data = vec![0u8; *size];
        let obj = Object {
            obj_type: ObjectType::Blob,
            data: data.clone(),
        };

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| rt.block_on(async { store.write_object(black_box(&obj)).await.unwrap() }));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_oid_hashing,
    bench_chunking,
    bench_chunk_store_write,
    bench_object_store
);
criterion_main!(benches);
