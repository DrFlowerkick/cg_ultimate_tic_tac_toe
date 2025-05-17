// random search of optimal parameters

use rand::prelude::*;
use rayon::prelude::*;

use cg_ultimate_tic_tac_toe::utilities::*;

fn get_config_candidate(config_lower_bounds: Config, config_upper_bounds: Config) -> Config {
    let mut rng = rand::thread_rng();

    let lower_bounds: Vec<f64> = config_lower_bounds.into();
    let upper_bounds: Vec<f64> = config_upper_bounds.into();
    let candidate: Vec<f64> = lower_bounds
        .into_iter()
        .zip(upper_bounds.into_iter())
        .map(|(min, max)| rng.gen_range(min..=max))
        .collect();
    candidate.into()
}

fn random_search(
    config_lower_bounds: Config,
    config_upper_bounds: Config,
    iterations: usize,
    num_matches: usize,
) -> (Config, f32) {
    let mut best_config = Config::default();
    let mut best_score = f32::NEG_INFINITY;

    for _ in 0..iterations {
        let candidate = get_config_candidate(config_lower_bounds, config_upper_bounds);

        let scores: f32 = (0..num_matches)
            .into_par_iter()
            .map(|i| {
                let is_starting_player = i % 2 == 0;
                run_match(candidate, is_starting_player)
            })
            .sum();
        let score = scores / num_matches as f32;

        if score > best_score {
            best_score = score;
            best_config = candidate;
        }
    }

    (best_config, best_score)
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    let iterations = 10_000;
    let num_matches = 100;

    let (best_config, best_score) = random_search(
        Config::lower_bounds(),
        Config::upper_bounds(),
        iterations,
        num_matches,
    );

    println!("Best Score: {}", best_score);
    println!("Best best_config: {:?}", best_config);
}
