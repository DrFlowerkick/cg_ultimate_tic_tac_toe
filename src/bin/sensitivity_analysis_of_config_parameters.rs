// analyze sensitive of each available config parameter

use rayon::prelude::*;
use statrs::statistics::Statistics;

use cg_ultimate_tic_tac_toe::utilities::*;

fn run_matches(config: Config, total_matches: usize) -> Vec<f64> {
    (0..total_matches)
        .into_par_iter()
        .map(|i| {
            let is_starting_player = i % 2 == 0;
            run_match(config, is_starting_player) as f64
        })
        .collect()
}

fn sensitivity_analysis(
    config: Config,
    lower_bound: Config,
    upper_bound: Config,
    parameter: &str,
    param_index: usize,
    steps: usize,
    matches_per_step: usize,
) {
    let mut config_vec: Vec<f64> = config.into();
    let min_value = <Vec<f64>>::from(lower_bound)[param_index];
    let max_value = <Vec<f64>>::from(upper_bound)[param_index];
    println!("analysis of sensitivity of {parameter}");
    println!(
        "variance of {:.2} to {:.2} in {} steps\n",
        min_value, max_value, steps
    );

    for step in 0..=steps {
        let param_value = min_value + (max_value - min_value) * step as f64 / steps as f64;
        config_vec[param_index] = param_value;

        let test_params = config_vec.clone().into();

        let scores = run_matches(test_params, matches_per_step);
        let mean = scores.clone().mean();
        let std_dev = scores.std_dev();

        println!(
            "value: {:>6.3} | mean score: {:.3} | std dev: {:.3}",
            param_value, mean, std_dev
        );
    }
    println!();
}

fn main() {
    for (param_index, parameter) in Config::parameter_names().iter().enumerate() {
        sensitivity_analysis(
            Config::default(),
            Config::lower_bounds(),
            Config::upper_bounds(),
            parameter,
            param_index,
            10, // 10 steps from lower bound to upper bound -> 11 variations
            30, // 30 matches per variation should be enough
        );
    }
}
