// START: use
use criterion::{criterion_group, criterion_main, Criterion};
use my_library_benchmark::*;
// END: use

// START: benchmark
pub fn criterion_benchmark(c: &mut Criterion) {
  // My benchmarks go here
  c.bench_function("random", |b| {// <callout id="criterion.closure" />
    let mut rng = RandomNumberGenerator::new();// <callout id="criterion.new_rng" />
    b.iter(|| {// <callout id="criterion.iter" />
      rng.range(1.0_f32..10_000_000_f32);// <callout id="criterion.rng_range" />
    })
  });
}
//END: benchmark

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
