//! Roll 3d6 repeatedly and graph the resulting distribution.
use my_library_flappy_assets_loading::RandomNumberGenerator;

fn main() {
  // Create a random number generator
  let mut rng = RandomNumberGenerator::new();
  // Store the results (minus 3)
  let mut results = vec![0; 16];
  // Roll 1,000 sets of 3d6 and increment results to map distribution
  for _ in 0..1_000 {
    let roll: usize =
      rng.range(1..=6) + rng.range(1..=6) + rng.range(1..=6);
    results[roll - 3] += 1;
  }
  // Print the distribution histogram
  println!("Distribution of 3d6 rolls:");
  for (i, count) in results.iter().enumerate() {
    print!("{: >2} : ", i + 3);
    for _ in 0..*count {
      print! {"#"};
    }
    println!();
  }
}
