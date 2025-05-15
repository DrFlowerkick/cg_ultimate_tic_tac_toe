// coarse grid search to optimize parameters of UltTTT

use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use cg_ultimate_tic_tac_toe::*;
use my_lib::my_monte_carlo_tree_search::*;
use my_lib::my_tic_tac_toe::*;

const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(995);
const TIME_OUT_SUCCESSIVE_TURNS: Duration = Duration::from_millis(95);

#[derive(Clone, Copy, Debug)]
struct FullConfig {
    mtcs: UltTTTMCTSConfig,
    heuristic: UltTTTHeuristicConfig,
}

fn run_match(params: &FullConfig, heuristic_is_start_player: bool) -> f32 {
    let mut first_mcts_ult_ttt: PlainMCTS<
        UltTTTMCTSGameNoGameCache,
        DynamicC,
        CachedUTC,
        HPWDefaultTTTNoGameCache,
        UltTTTHeuristic,
        HeuristicCutoff,
    > = PlainMCTS::new(params.mtcs, params.heuristic);
    let mut first_ult_ttt_game_data = UltTTT::new();
    let mut first_time_out = TIME_OUT_FIRST_TURN;
    let mut second_mcts_ult_ttt: PlainMCTS<
        UltTTTMCTSGameNoGameCache,
        DynamicC,
        CachedUTC,
        ExpandAll<UltTTTMCTSGame<NoGameCache<UltTTT, UltTTTMove>>>,
        NoHeuristic,
        DefaultSimulationPolicy,
    > = PlainMCTS::new(UltTTTMCTSConfig::default(), BaseHeuristicConfig::default());
    let mut second_ult_ttt_game_data = UltTTT::new();
    let mut second_time_out = TIME_OUT_FIRST_TURN;

    let mut first = if heuristic_is_start_player {
        first_ult_ttt_game_data.set_current_player(TicTacToeStatus::Me);
        second_ult_ttt_game_data.set_current_player(TicTacToeStatus::Opp);
        true
    } else {
        first_ult_ttt_game_data.set_current_player(TicTacToeStatus::Opp);
        second_ult_ttt_game_data.set_current_player(TicTacToeStatus::Me);
        false
    };

    while UltTTTMCTSGame::evaluate(&first_ult_ttt_game_data, &mut first_mcts_ult_ttt.game_cache)
        .is_none()
    {
        if first {
            let start = Instant::now();
            first_mcts_ult_ttt.set_root(&first_ult_ttt_game_data);
            while start.elapsed() < first_time_out {
                first_mcts_ult_ttt.iterate();
            }
            first_time_out = TIME_OUT_SUCCESSIVE_TURNS;
            let selected_move = *first_mcts_ult_ttt.select_move();
            first_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                &first_ult_ttt_game_data,
                &selected_move,
                &mut first_mcts_ult_ttt.game_cache,
            );
            second_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                &second_ult_ttt_game_data,
                &selected_move,
                &mut second_mcts_ult_ttt.game_cache,
            );
            first = false;
        } else {
            let start = Instant::now();
            second_mcts_ult_ttt.set_root(&second_ult_ttt_game_data);
            while start.elapsed() < second_time_out {
                second_mcts_ult_ttt.iterate();
            }
            second_time_out = TIME_OUT_SUCCESSIVE_TURNS;
            let selected_move = *second_mcts_ult_ttt.select_move();
            second_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                &second_ult_ttt_game_data,
                &selected_move,
                &mut second_mcts_ult_ttt.game_cache,
            );
            first_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                &first_ult_ttt_game_data,
                &selected_move,
                &mut first_mcts_ult_ttt.game_cache,
            );
            first = true;
        }
    }
    UltTTTMCTSGame::evaluate(&first_ult_ttt_game_data, &mut first_mcts_ult_ttt.game_cache).unwrap()
}

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
                                        let mut params = FullConfig {
                                            mtcs: UltTTTMCTSConfig::default(),
                                            heuristic: UltTTTHeuristicConfig::default(),
                                        };
                                        params.heuristic.meta_weight_base = meta_weight_base;
                                        params.heuristic.constraint_factor = constraint_factor;
                                        params.heuristic.free_choice_constraint_factor =
                                            constraint_factor;
                                        params.heuristic.meta_cell_big_threat = meta_cell_big_threat;
                                        params.heuristic.meta_cell_small_threat = meta_cell_small_threat;
                                        params.heuristic.direct_loss_value = direct_loss_value;

                                        let mut total_score: f32 = 0.0;
                                        (0..num_matches).for_each(|match_counter| {
                                            total_score += run_match(&params, match_counter % 2 == 0);
                                            let progress = progress_counter.fetch_add(1, Ordering::SeqCst) + 1;
                                            if progress % 10 == 0 {
                                                println!("[{}/{}] progress...", progress, total_num_of_matches);
                                            }
                                        });

                                        let avg_score = total_score / num_matches as f32;

                                        println!("Config: meta_weight_base = {:.2}, constraint_factor = {:.2}, meta_cell_big_threat = {:.2}, direct_loss_value = {:.2} → Avg Score: {:.3}",
                                            params.heuristic.meta_weight_base,
                                            params.heuristic.constraint_factor,
                                            params.heuristic.meta_cell_big_threat,
                                            params.heuristic.direct_loss_value,
                                            avg_score);

                                        results.lock().unwrap().push((params, avg_score));
                                    });
                                });
                        });
                });
        });

    // save results
    let results = results.into_inner().unwrap();
    save_results(&results, "parallel_grid_search_results.csv");
}

fn save_results(results: &Vec<(FullConfig, f32)>, filename: &str) {
    let file = File::create(filename).expect("Unable to create file");
    let mut writer = BufWriter::new(file);

    writeln!(
        writer,
        "meta_weight_base,constraint_factor,meta_cell_big_threat,direct_loss_value,average_score"
    )
    .unwrap();

    for (params, avg_score) in results {
        writeln!(
            writer,
            "{},{},{},{},{}",
            params.heuristic.meta_weight_base,
            params.heuristic.constraint_factor,
            params.heuristic.meta_cell_big_threat,
            params.heuristic.direct_loss_value,
            avg_score
        )
        .unwrap();
    }

    println!("Results written to {}", filename);
}
