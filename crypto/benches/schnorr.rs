use criterion::{black_box, criterion_group, criterion_main, Criterion};
use goldilocks_crypto::{sign_with_nonce, verify_signature, ScalarField, Point};

fn bench_schnorr_sign(c: &mut Criterion) {
    let mut group = c.benchmark_group("schnorr_sign");
    
    // Generate test data
    let private_key_bytes = ScalarField::sample_crypto().to_bytes_le();
    let nonce_bytes = ScalarField::sample_crypto().to_bytes_le();
    let message = [0u8; 40]; // Standard 40-byte message (Fp5 element)
    
    group.bench_function("sign_with_nonce", |b| {
        b.iter(|| {
            sign_with_nonce(
                black_box(&private_key_bytes),
                black_box(&message),
                black_box(&nonce_bytes)
            )
        })
    });
    
    group.finish();
}

fn bench_schnorr_verify(c: &mut Criterion) {
    let mut group = c.benchmark_group("schnorr_verify");
    
    // Generate test data and signature
    let private_key = ScalarField::sample_crypto();
    let private_key_bytes = private_key.to_bytes_le();
    let public_key = Point::generator().mul(&private_key);
    let public_key_bytes = public_key.encode().to_bytes_le();
    
    let nonce_bytes = ScalarField::sample_crypto().to_bytes_le();
    let message = [0u8; 40];
    let signature = sign_with_nonce(&private_key_bytes, &message, &nonce_bytes).unwrap();
    
    group.bench_function("verify_signature", |b| {
        b.iter(|| {
            verify_signature(
                black_box(&signature),
                black_box(&message),
                black_box(&public_key_bytes)
            )
        })
    });
    
    group.finish();
}

fn bench_point_mul(c: &mut Criterion) {
    let mut group = c.benchmark_group("point_mul");
    
    let generator = Point::generator();
    let scalar = ScalarField::sample_crypto();
    
    group.bench_function("point_mul_by_scalar", |b| {
        b.iter(|| black_box(&generator).mul(black_box(&scalar)))
    });
    
    group.finish();
}

fn bench_point_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("point_encode");
    
    let private_key = ScalarField::sample_crypto();
    let public_key = Point::generator().mul(&private_key);
    
    group.bench_function("point_encode", |b| {
        b.iter(|| black_box(&public_key).encode())
    });
    
    group.finish();
}

fn bench_point_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("point_decode");
    
    let private_key = ScalarField::sample_crypto();
    let public_key = Point::generator().mul(&private_key);
    let encoded = public_key.encode();
    
    group.bench_function("point_decode", |b| {
        b.iter(|| Point::decode(black_box(&encoded)))
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_schnorr_sign,
    bench_schnorr_verify,
    bench_point_mul,
    bench_point_encode,
    bench_point_decode
);
criterion_main!(benches);

