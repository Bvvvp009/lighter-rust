use criterion::{black_box, criterion_group, criterion_main, Criterion};
use goldilocks_crypto::ScalarField;

fn bench_scalar_from_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_field_from_bytes");
    
    let bytes = [0u8; 40];
    group.bench_function("from_bytes_le", |b| {
        b.iter(|| ScalarField::from_bytes_le(black_box(&bytes)))
    });
    
    group.finish();
}

fn bench_scalar_to_bytes(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_field_to_bytes");
    
    let scalar = ScalarField::sample_crypto();
    group.bench_function("to_bytes_le", |b| {
        b.iter(|| black_box(scalar).to_bytes_le())
    });
    
    group.finish();
}

fn bench_scalar_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_field_add");
    
    let a = ScalarField::sample_crypto();
    let b = ScalarField::sample_crypto();
    
    group.bench_function("add", |bencher| {
        bencher.iter(|| black_box(a).add(black_box(b)))
    });
    
    group.finish();
}

fn bench_scalar_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_field_mul");
    
    let a = ScalarField::sample_crypto();
    let b = ScalarField::sample_crypto();
    
    group.bench_function("mul", |bencher| {
        bencher.iter(|| black_box(a).mul(black_box(&b)))
    });
    
    group.finish();
}

fn bench_scalar_square(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_field_square");
    
    let a = ScalarField::sample_crypto();
    
    group.bench_function("square", |bencher| {
        bencher.iter(|| black_box(a).square())
    });
    
    group.finish();
}

fn bench_scalar_to_canonical(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_field_to_canonical");
    
    let scalar = ScalarField::sample_crypto();
    let montgomery = scalar.mul(&ScalarField::sample_crypto());
    
    group.bench_function("to_canonical", |bencher| {
        bencher.iter(|| black_box(montgomery).to_canonical())
    });
    
    group.finish();
}

fn bench_scalar_sample_crypto(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalar_field_sample");
    
    group.bench_function("sample_crypto", |bencher| {
        bencher.iter(|| ScalarField::sample_crypto())
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_scalar_from_bytes,
    bench_scalar_to_bytes,
    bench_scalar_add,
    bench_scalar_mul,
    bench_scalar_square,
    bench_scalar_to_canonical,
    bench_scalar_sample_crypto
);
criterion_main!(benches);

