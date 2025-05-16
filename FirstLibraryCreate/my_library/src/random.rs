// START: random_type
use rand::{prelude::StdRng, Rng, SeedableRng};//<callout id="first_library_create.use" />
use std::ops::Range;

pub struct RandomNumberGenerator {//<callout id="first_library_create.struct" />
  rng: StdRng,//<callout id="first_library_create.private" />
}
// END: random_type

// START: default
impl Default for RandomNumberGenerator {
  fn default() -> Self {
    Self::new()
  }
}
//END: default

// START: constructors
impl RandomNumberGenerator {
  pub fn new() -> Self {//<callout id="first_library_create.new" />
    Self {
      rng: StdRng::from_entropy(),//<callout id="first_library_create.from_entropy" />
    }
  }

  pub fn seeded(seed: u64) -> Self {//<callout id="first_library_create.seeded" />
    Self {
      rng: StdRng::seed_from_u64(seed),
    }
  }
  //END: constructors

  // START: randomness
  // START: range_header
  pub fn range(&mut self, range: Range<u32>) -> u32 {
  // END: range_header
    self.rng.gen_range(range)
  }
  //END: randomness
}

//START: mod_test
#[cfg(test)]
mod test {
  use super::*;
  //END: mod_test

  //START: test_range_bounds
  #[test]//<callout id="first_library_create.tests.test_decoration" />
  fn test_range_bounds() {
    let mut rng = RandomNumberGenerator::new();
    for _ in 0..1000 {//<callout id="first_library_create.tests.repeat" />
      let n = rng.range(1..10);
      assert!(n >= 1);//<callout id="first_library_create.tests.assert_ge" />
      assert!(n < 10);//<callout id="first_library_create.tests.assert_lt" />
    }
  }
  //END: test_range_bounds

  //START: test_reproducibility
  #[test]
  fn test_reproducibility() {
    let mut rng = (
      RandomNumberGenerator::seeded(1),
      RandomNumberGenerator::seeded(1),
    );//<callout id="first_library_create.tests.rng_tuple" />
    (0..1000).for_each(|_| {//<callout id="first_library_create.tests.range_for" />
      assert_eq!(//<callout id="first_library_create.tests.assert_eq" />
        rng.0.range(u32::MIN..u32::MAX),
        rng.1.range(u32::MIN..u32::MAX),
      );
    });
  }
  //END: test_reproducibility
}
