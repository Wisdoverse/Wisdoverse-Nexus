use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nexis_doc::CRDTDocument;

fn benchmark_apply_update(c: &mut Criterion) {
    let doc = CRDTDocument::new();
    let update = vec![7_u8; 256];

    c.bench_function("crdt/apply_update", |b| {
        b.iter(|| {
            doc.apply_update(black_box(&update))
                .expect("benchmark update should succeed");
        });
    });
}

fn benchmark_encode_update(c: &mut Criterion) {
    let doc = CRDTDocument::new();

    c.bench_function("crdt/encode_update", |b| {
        b.iter(|| {
            black_box(doc.encode_update());
        });
    });
}

criterion_group!(
    crdt_benches,
    benchmark_apply_update,
    benchmark_encode_update
);
criterion_main!(crdt_benches);
