use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use goldilocks_crypto::{verify_signature, sign, ScalarField, Point};

// Helper to create deterministic signatures for benchmarking
fn create_test_signature(private_key_bytes: &[u8], message: &[u8]) -> Vec<u8> {
    sign(private_key_bytes, message).unwrap()
}

fn bench_schnorr_sign(c: &mut Criterion) {
    let mut group = c.benchmark_group("schnorr_sign");
    
    // Generate test data
    let private_key_bytes = ScalarField::sample_crypto().to_bytes_le();
    let message = [0u8; 40]; // Standard 40-byte message (Fp5 element)
    
    // Benchmark the automatic nonce generation version
    group.bench_function("sign_auto_nonce", |b| {
        b.iter(|| {
            sign(
                black_box(&private_key_bytes),
                black_box(&message)
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
    
    let message = [0u8; 40];
    let signature = create_test_signature(&private_key_bytes, &message);
    
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

fn bench_point_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("point_operations");
    
    let generator = Point::generator();
    let scalar = ScalarField::sample_crypto();
    let point = generator.mul(&scalar);
    
    group.bench_function("point_add", |b| {
        b.iter(|| black_box(&point).add(black_box(&generator)))
    });
    
    group.bench_function("point_double", |b| {
        b.iter(|| black_box(&point).double())
    });
    
    group.finish();
}

fn bench_batch_verify_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_verify");
    
    // Simulate batch verification by verifying multiple signatures
    for size in [1, 5, 10, 20, 50].iter() {
        let mut signatures = Vec::new();
        let mut messages = Vec::new();
        let mut public_keys = Vec::new();
        
        for _ in 0..*size {
            let private_key = ScalarField::sample_crypto();
            let private_key_bytes = private_key.to_bytes_le();
            let public_key = Point::generator().mul(&private_key);
            let public_key_bytes = public_key.encode().to_bytes_le();
            
            let message = [(*size % 256) as u8; 40];
            let signature = sign(&private_key_bytes, &message).unwrap();
            
            signatures.push(signature);
            messages.push(message);
            public_keys.push(public_key_bytes);
        }
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                for i in 0..signatures.len() {
                    verify_signature(
                        black_box(&signatures[i]),
                        black_box(&messages[i]),
                        black_box(&public_keys[i])
                    ).unwrap();
                }
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_schnorr_sign,
    bench_schnorr_verify,
    bench_point_mul,
    bench_point_operations,
    bench_point_encode,
    bench_point_decode,
    bench_batch_verify_simulation
);
criterion_main!(benches);