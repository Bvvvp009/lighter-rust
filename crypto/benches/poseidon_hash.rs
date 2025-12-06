use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use poseidon_hash::{Goldilocks, hash_to_quintic_extension};
use goldilocks_crypto::Fp5Element;

fn bench_poseidon_hash_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("poseidon_hash");
    
    let elements: Vec<Goldilocks> = (0..3)
        .map(|i| Goldilocks::from_canonical_u64(i as u64))
        .collect();
    
    group.bench_with_input(
        BenchmarkId::new("hash_to_quintic_extension", "3_elements"),
        &elements,
        |b, input| {
            b.iter(|| hash_to_quintic_extension(black_box(input)))
        },
    );
    
    group.finish();
}

fn bench_poseidon_hash_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("poseidon_hash");
    
    let elements: Vec<Goldilocks> = (0..10)
        .map(|i| Goldilocks::from_canonical_u64(i as u64))
        .collect();
    
    group.bench_with_input(
        BenchmarkId::new("hash_to_quintic_extension", "10_elements"),
        &elements,
        |b, input| {
            b.iter(|| hash_to_quintic_extension(black_box(input)))
        },
    );
    
    group.finish();
}

fn bench_poseidon_hash_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("poseidon_hash");
    
    let elements: Vec<Goldilocks> = (0..100)
        .map(|i| Goldilocks::from_canonical_u64(i as u64))
        .collect();
    
    group.bench_with_input(
        BenchmarkId::new("hash_to_quintic_extension", "100_elements"),
        &elements,
        |b, input| {
            b.iter(|| hash_to_quintic_extension(black_box(input)))
        },
    );
    
    group.finish();
}

fn bench_goldilocks_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("goldilocks_field");
    
    let a = Goldilocks::from_canonical_u64(42);
    let b = Goldilocks::from_canonical_u64(10);
    
    group.bench_function("add", |bencher| {
        bencher.iter(|| black_box(a).add(black_box(&b)))
    });
    
    group.finish();
}

fn bench_goldilocks_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("goldilocks_field");
    
    let a = Goldilocks::from_canonical_u64(42);
    let b = Goldilocks::from_canonical_u64(10);
    
    group.bench_function("mul", |bencher| {
        bencher.iter(|| black_box(a).mul(black_box(&b)))
    });
    
    group.finish();
}

fn bench_fp5_element_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("fp5_element");
    
    let a = Fp5Element::zero();
    let b = Fp5Element::zero();
    
    group.bench_function("add", |bencher| {
        bencher.iter(|| black_box(a).add(black_box(&b)))
    });
    
    group.finish();
}

fn bench_fp5_element_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("fp5_element");
    
    let a = Fp5Element::zero();
    let b = Fp5Element::zero();
    
    group.bench_function("mul", |bencher| {
        bencher.iter(|| black_box(a).mul(black_box(&b)))
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_poseidon_hash_small,
    bench_poseidon_hash_medium,
    bench_poseidon_hash_large,
    bench_goldilocks_add,
    bench_goldilocks_mul,
    bench_fp5_element_add,
    bench_fp5_element_mul
);
criterion_main!(benches);



