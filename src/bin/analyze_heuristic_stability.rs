// test ih heuristic results in mor or less stable score

use rayon::prelude::*;
use statrs::statistics::Statistics;

use cg_ultimate_tic_tac_toe::utilities::*;

/// Run multiple matches
fn run_matches(config: Config, total_matches: usize) -> Vec<f64> {
    (0..total_matches)
        .into_par_iter()
        .map(|i| {
            let is_starting_player = i % 2 == 0;
            run_match(config.clone(), is_starting_player).0
        })
        .collect()
}

/// do stability analysis
fn analyze_stability(config: Config) {
    let eval_sizes = [10, 25, 50, 100];

    for &size in &eval_sizes {
        let scores = run_matches(config.clone(), size);
        let mean = scores.clone().mean();
        let std_dev = scores.std_dev();

        println!(
            "After {:>3} matches: Average Score = {:.3}, Std Dev = {:.3}",
            size, mean, std_dev
        );
    }
}

fn main() {
    println!("heuristic stability analysis is starting...\n");
    analyze_stability(Config::default());
}
