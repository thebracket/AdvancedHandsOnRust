use rand::{Rng, SeedableRng, distributions::uniform::{SampleRange, SampleUniform}};

#[cfg(all(not(feature = "pcg"), not(feature = "xorshift")))]
type RngCore = rand::prelude::StdRng;

#[cfg(feature = "pcg")]
type RngCore = rand_pcg::Pcg64Mcg;

#[cfg(feature = "xorshift")]
type RngCore = rand_xorshift::XorShiftRng;

#[derive(bevy::prelude::Resource)]
pub struct RandomNumberGenerator {
  rng: RngCore,
}

impl Default for RandomNumberGenerator {
  fn default() -> Self {
    Self::new()
  }
}

impl RandomNumberGenerator {
  pub fn new() -> Self {
    Self {
      rng: RngCore::from_entropy(),
    }
  }

  pub fn seeded(seed: u64) -> Self {
    Self {
      rng: RngCore::seed_from_u64(seed),
    }
  }

  pub fn next<T>(&mut self) -> T
  where rand::distributions::Standard: rand::prelude::Distribution<T>
  {
    self.rng.gen()
  }

  pub fn range<T>(&mut self, range: impl SampleRange<T>) -> T
  where
    T: SampleUniform + PartialOrd,
  {
    self.rng.gen_range(range)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_range_bounds() {
    let mut rng = RandomNumberGenerator::new();
    for _ in 0..1000 {
      let n = rng.range(1..10);
      assert!(n >= 1);
      assert!(n < 10);
    }
  }

  #[test]
  fn test_inclusive_range_bounds() {
    let mut rng = RandomNumberGenerator::new();
    for _ in 0..1000 {
      let n = rng.range(1..10);
      assert!(n >= 1);
      assert!(n <= 10);
    }
  }

  #[test]
  fn test_reproducibility() {
    let mut rng = (
      RandomNumberGenerator::seeded(1),
      RandomNumberGenerator::seeded(1),
    );
    (0..1000).for_each(|_| {
      assert_eq!(
        rng.0.range(u32::MIN..u32::MAX),
        rng.1.range(u32::MIN..u32::MAX),
      );
    });
  }

  #[test]
  fn test_next_types() {
    let mut rng = RandomNumberGenerator::new();
    let _ : i32 = rng.next();
    let _ = rng.next::<f32>();
  }

  #[test]
  fn test_float() {
    let mut rng = RandomNumberGenerator::new();
    for _ in 0..1000 {
      // fun bug: rng.range(f32::MIN .. f32::MAX)
      // Crashes with a "range overflow" error.
      let n = rng.range(-3.40282347e+37_f32..3.40282347e+37_f32);
      assert!(n.is_finite());
      assert!(!n.is_infinite());
      assert!(!n.is_nan());
    }
  }
}

// START: plugin
pub struct RandomPlugin;// <callout id="random.bevy.plugin_struct" />

impl bevy::prelude::Plugin for RandomPlugin {// <callout id="random.bevy.plugin_impl" />
  fn build(&self, app: &mut bevy::prelude::App) {// <callout id="random.bevy.plugin_build" />
      app.insert_resource(RandomNumberGenerator::new());// <callout id="random.bevy.plugin_resource" />
  }
}
//END: plugin