use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput, BatchSize};
use kvs::engines::{kvs::KvStore, kvs_engine::KvsEngine, sled::SledKvsEngine};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

// generate 100 bytes of random length
fn generate_data(seed: u64, size: u64) -> Vec<String> {
    // generate random number generator
    let mut rng = ChaCha20Rng::seed_from_u64(seed);

    let mut data = Vec::new();
    let mut len: usize;
    for _ in 0..size {
        len = rng.gen_range(1..100000);
        let temp = rng
            .clone()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect();
        data.push(temp);
    }
    data
}

// kvs_write, generate 100 (key, value) pairs where, key, value \in 16^{100000} (really long key, values)
fn write(c: &mut Criterion) {
    // generate keys, values
    let keys: Vec<String> = generate_data(100, 200);
    let values: Vec<String> = generate_data(100, 200);
    // open KvsEngine for KvStore
    let mut kvs = KvStore::open("./").unwrap();
    let mut sled = SledKvsEngine::open("./db").unwrap();
    // create a benchmark group, to bench over an iterator of inputs
    let mut group = c.benchmark_group("kvs_write");
    // find throughput for each bench
    let mut dataSize: u64 = 0;
    for i in 0..keys.len() {
        dataSize += (keys[i].len() as u64) + (values[i].len() as u64);
    }
    // generate seeded RNG      
    let mut rng = ChaCha20Rng::seed_from_u64(1);
    // bench for kvs
    group.bench_with_input(BenchmarkId::from_parameter("kvs_write"), &vec![&keys, &values], |b, data|{
        let mut i = 0;
        b.iter_batched(|| {
                let i = rng.gen_range(0..keys.len());
                (data[0][i].clone(), data[1][i].clone())
            }, |key|{
                // set kvs
                kvs.set(key.0, key.1).unwrap();
            },
            BatchSize::SmallInput
        )
    });
    // bench for sled
    // bench for kvs
    group.bench_with_input(BenchmarkId::from_parameter("sled_write"), &vec![&keys, &values], |b, data|{
        let mut i = 0;
        b.iter_batched(|| {
                let i = rng.gen_range(0..keys.len());
                (data[0][i].clone(), data[1][i].clone())
            }, |key|{
                // set kvs
                sled.set(key.0, key.1).unwrap();
            },
            BatchSize::SmallInput
        )
    });
    group.finish();
}

// kvs_read, test reads of 1000 keys, each with values of over 100000 bytes as value
fn read(c: &mut Criterion) {
    // generate keys, values
    let keys: Vec<String> = generate_data(100, 100);
    let values: Vec<String> = generate_data(100, 100);
    // open Engines, and make writes in preparation of benches
    let mut kvs = KvStore::open("./").unwrap();
    let mut sled = SledKvsEngine::open("./db").unwrap();
    // set values at keys for both engines
    let mut  dataSize: u64 = 0;
    for i in 0..100 {
        // make writes for KvStore
        kvs.set(keys[i].to_owned(), values[i].to_owned()).unwrap();
        // make writes for sled
        sled.set(keys[i].to_owned(), values[i].to_owned()).unwrap();
        dataSize += (keys[i].len() as u64) + (values[i].len() as u64);
    }
    // generate rng for indices
    let mut rng = ChaCha20Rng::seed_from_u64(1);
    // keys, values set in state, now run benches
    let mut group = c.benchmark_group("reads");
    // set throughput
    group.throughput(Throughput::Bytes(dataSize));
    // bench for kvs
    group.bench_with_input(BenchmarkId::from_parameter("kvs_read".to_owned()), &keys, |b,keys| {
        b.iter_batched(
            || {
                let i = rng.gen_range(0..keys.len());
                keys[i].clone()
            },
            // iterate over all keys, and make gets
            // all should be valid gets
            |key|{
                if let None = kvs.get(key).unwrap() {
                    panic!();
                }
            },
            BatchSize::SmallInput
        )
    });
    group.bench_with_input(BenchmarkId::from_parameter("sled_read"), &keys, |b, keys| {
        b.iter_batched(
            || {
                let i = rng.gen_range(0..keys.len());
                keys[i].clone()
            },
            // iterate over all keys, and make gets
            // all should be valid gets
            |key|{
                if let None = sled.get(key).unwrap() {
                    panic!();
                }
            },
            BatchSize::SmallInput
        )
    });
    // finish bench
    group.finish();
}   

criterion_group!(benches, write, read);
criterion_main!(benches);
