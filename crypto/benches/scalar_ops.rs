use criterion::{black_box, criterion_group, criterion_main, Criterion};
use goldilocks_crypto::ScalarField;

fn bench_scalar_add(c: &mut Criterion) {
    let a = ScalarField::sample_crypto();
    let b = ScalarField::sample_crypto();
    
    c.bench_function("scalar_add", |bencher| {
        bencher.iter(|| {
            black_box(a.add(black_box(b)))
        });
    });
}

fn bench_scalar_sub(c: &mut Criterion) {
    let a = ScalarField::sample_crypto();
    let b = ScalarField::sample_crypto();
    
    c.bench_function("scalar_sub", |bencher| {
        bencher.iter(|| {
            black_box(a.sub(black_box(b)))
        });
    });
}

fn bench_scalar_mul(c: &mut Criterion) {
    let a = ScalarField::sample_crypto();
    let b = ScalarField::sample_crypto();
    
    c.bench_function("scalar_mul", |bencher| {
        bencher.iter(|| {
            black_box(a.mul(black_box(&b)))
        });
    });
}

fn bench_signature_generation(c: &mut Criterion) {
    use goldilocks_crypto::schnorr::sign;
    use poseidon_hash::{Goldilocks, hash_to_quintic_extension};
    
    let sk = ScalarField::sample_crypto();
    let sk_bytes = sk.to_bytes_le();
    let data = [Goldilocks::from_canonical_u64(42); 10];
    let message = hash_to_quintic_extension(&data).to_bytes_le();
    
    c.bench_function("schnorr_sign", |bencher| {
        bencher.iter(|| {
            black_box(sign(&sk_bytes, &message))
        });
    });
}

criterion_group!(benches, bench_scalar_add, bench_scalar_sub, bench_scalar_mul, bench_signature_generation);
criterion_main!(benches);
