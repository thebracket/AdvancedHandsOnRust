use rand::{prelude::StdRng, Rng, SeedableRng};
use std::ops::Range;

pub struct RandomNumberGenerator {
  rng: StdRng,
}

impl Default for RandomNumberGenerator {
  fn default() -> Self {
    Self::new()
  }
}

impl RandomNumberGenerator {
  pub fn new() -> Self {
    Self {
      rng: StdRng::from_entropy(),
    }
  }

  pub fn seeded(seed: u64) -> Self {
    Self {
      rng: StdRng::seed_from_u64(seed),
    }
  }

  //START: next
  pub fn next<T>(&mut self) -> T
  where rand::distributions::Standard: rand::prelude::Distribution<T>
  {
    self.rng.gen()
  }
  //END: next

  //START: GenericRange
  //START: JustRange
  pub fn range<T>(&mut self, range: Range<T>) -> T
  //END: JustRange
  where
  T: rand::distributions::uniform::SampleUniform + PartialOrd,
  {
    self.rng.gen_range(range)
  }
  //END: GenericRange
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

  // START: test_next_types
  #[test]
  fn test_next_types() {
    let mut rng = RandomNumberGenerator::new();
    let _ : i32 = rng.next();//<callout id="first_library_generic.return_type" />
    let _ = rng.next::<f32>();//<callout id="first_library_generic.turbofish" />
  }
  //END: test_next_types

  // START: test_float
  #[test]
  fn test_float() {
    let mut rng = RandomNumberGenerator::new();
    for _ in 0..1000 {
      let n = rng.range(-5000.0f32..5000.0f32);
      assert!(n.is_finite());
      assert!(n > -5000.0);
      assert!(n < 5000.0);
    }
  }
  //END: test_float
}
