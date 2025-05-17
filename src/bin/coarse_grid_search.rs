// coarse grid search to optimize parameters of UltTTT

use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use cg_ultimate_tic_tac_toe::*;

use cg_ultimate_tic_tac_toe::utilities::*;

fn main() {
    // config parameter sets
    let meta_weight_base_vals = [0.2, 0.3, 0.5];
    let constraint_factor_vals = [1.0, 1.5, 2.0];
    let meta_cell_big_threat_vals = [2.0, 3.0, 4.0];
    let meta_cell_small_threat_vals = [0.5, 1.0, 1.5];
    let direct_loss_value_vals = [0.0, 0.005, 0.01, 0.025];

    let num_matches = 100;

    let total_num_of_matches = meta_weight_base_vals.len()
        * constraint_factor_vals.len()
        * meta_cell_big_threat_vals.len()
        * meta_cell_small_threat_vals.len()
        * direct_loss_value_vals.len()
        * num_matches;

    let progress_counter = Arc::new(AtomicUsize::new(0));

    // use mutex for thread save handling of results
    let results = Mutex::new(Vec::new());

    // create all possible combinations of parameter
    meta_weight_base_vals
        .par_iter()
        .for_each(|&meta_weight_base| {
            constraint_factor_vals
                .iter()
                .for_each(|&constraint_factor| {
                    meta_cell_big_threat_vals
                        .iter()
                        .for_each(|&meta_cell_big_threat| {
                            meta_cell_small_threat_vals
                                .iter()
                                .for_each(|&meta_cell_small_threat| {
                                direct_loss_value_vals
                                    .iter()
                                    .for_each(|&direct_loss_value| {
                                        let config = Config {
                                            heuristic: UltTTTHeuristicConfig {
                                                meta_weight_base,
                                                constraint_factor,
                                                free_choice_constraint_factor: constraint_factor,
                                                meta_cell_big_threat,
                                                meta_cell_small_threat,
                                                direct_loss_value,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        };

                                        let mut total_score: f32 = 0.0;
                                        (0..num_matches).for_each(|match_counter| {
                                            total_score += run_match(config, match_counter % 2 == 0);
                                            let progress = progress_counter.fetch_add(1, Ordering::SeqCst) + 1;
                                            if progress % 10 == 0 {
                                                println!("[{}/{}] progress...", progress, total_num_of_matches);
                                            }
                                        });

                                        let avg_score = total_score / num_matches as f32;

                                        println!("Config: meta_weight_base = {:.2}, constraint_factor = {:.2}, meta_cell_big_threat = {:.2}, direct_loss_value = {:.2} → Avg Score: {:.3}",
                                            config.heuristic.meta_weight_base,
                                            config.heuristic.constraint_factor,
                                            config.heuristic.meta_cell_big_threat,
                                            config.heuristic.direct_loss_value,
                                            avg_score);

                                        results.lock().unwrap().push((config.heuristic, avg_score));
                                    });
                                });
                        });
                });
        });

    // save results
    let results = results.into_inner().unwrap();
    save_results(&results, "parallel_grid_search_results.csv");
}

fn save_results(results: &Vec<(UltTTTHeuristicConfig, f32)>, filename: &str) {
    let file = File::create(filename).expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    writeln!(
        writer,
        "meta_weight_base,constraint_factor,meta_cell_big_threat,direct_loss_value,average_score"
    )
    .unwrap();

    for (heuristic_config, avg_score) in results {
        writeln!(
            writer,
            "{},{},{},{},{}",
            heuristic_config.meta_weight_base,
            heuristic_config.constraint_factor,
            heuristic_config.meta_cell_big_threat,
            heuristic_config.direct_loss_value,
            avg_score
        )
        .unwrap();
    }

    println!("Results written to {}", filename);
}
