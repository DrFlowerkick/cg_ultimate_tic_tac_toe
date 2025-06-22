// setting up matches to collect learning data for linfa

use super::{collect_labels, LabelSink};
use crate::{UltTTT, UltTTTMCTSConfig, UltTTTMCTSGame};
use anyhow::Result;
use my_lib::my_mcts::{
    ExpansionPolicy, Heuristic, MCTSAlgo, MCTSGame, NoTranspositionTable, PlainMCTS, PruneToRoot,
    SimulationPolicy, UCTPolicy, UTCCache,
};
use my_lib::my_optimizer::SharedError;
use rayon::prelude::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tracing::{info, span, Level};
use uuid::Uuid;

// NoTranspositionTable is required, because during training nodes before root will be pruned,
// which conflicts with usage of a transposition table of PlainTree
pub type UltMCTSTraining<H, UC, UP, EP, SP> =
    PlainMCTS<UltTTTMCTSGame, H, UltTTTMCTSConfig, UC, NoTranspositionTable, UP, EP, SP>;

type MatchResult<H, UC, UP, EP, SP> = (
    f64,
    UltMCTSTraining<H, UC, UP, EP, SP>,
    UltMCTSTraining<H, UC, UP, EP, SP>,
);

pub fn run_training_match<H, UC, UP, EP, SP, S>(
    mut first_mcts_ult_ttt: UltMCTSTraining<H, UC, UP, EP, SP>,
    mut second_mcts_ult_ttt: UltMCTSTraining<H, UC, UP, EP, SP>,
    turn_duration: Duration,
    generation: u32,
    min_visits: usize,
    label_sink: S,
) -> Result<MatchResult<H, UC, UP, EP, SP>>
where
    H: Heuristic<UltTTTMCTSGame>,
    UC: UTCCache<UltTTTMCTSGame, UP, UltTTTMCTSConfig>,
    UP: UCTPolicy<UltTTTMCTSGame, UltTTTMCTSConfig>,
    EP: ExpansionPolicy<UltTTTMCTSGame, H, UltTTTMCTSConfig>,
    SP: SimulationPolicy<UltTTTMCTSGame, H, UltTTTMCTSConfig>,
    S: LabelSink,
{
    let mut first_ult_ttt_game_data = UltTTT::new();
    let mut second_ult_ttt_game_data = UltTTT::new();

    // player first always starts
    let mut first = true;

    let mut turn_counter = 0;
    while UltTTTMCTSGame::evaluate(&first_ult_ttt_game_data, &mut first_mcts_ult_ttt.game_cache)
        .is_none()
    {
        turn_counter += 1;
        if first {
            // iterate first tree from first perspective
            let reset_tree = first_mcts_ult_ttt.set_root(&first_ult_ttt_game_data);
            if !reset_tree && turn_counter > 1 {
                tracing::debug!(
                    turn_counter,
                    "Reset tree root of first. All labels are lost!"
                );
            }

            if reset_tree && turn_counter > 1 {
                tracing::debug!(
                    turn_counter,
                    "Collecting labels of first up until new root."
                );
                collect_labels(
                    &first_mcts_ult_ttt.tree,
                    0,
                    generation,
                    min_visits,
                    false,
                    label_sink.clone(),
                )?;
                first_mcts_ult_ttt.tree.prune_to_root();
            }

            tracing::debug!(turn_counter, "Starting iteration of first.");
            let start = Instant::now();
            while start.elapsed() < turn_duration {
                first_mcts_ult_ttt.iterate();
            }
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
            tracing::debug!(turn_counter, "Collecting labels iteration of first.");
            first = false;
        } else {
            // iterate second tree from second perspective
            let reset_tree = second_mcts_ult_ttt.set_root(&second_ult_ttt_game_data);
            if !reset_tree && turn_counter > 2 {
                tracing::debug!(turn_counter, "Reset tree root of second.");
            }

            if reset_tree && turn_counter > 2 {
                tracing::debug!(
                    turn_counter,
                    "Collecting labels of second up until new root."
                );
                collect_labels(
                    &second_mcts_ult_ttt.tree,
                    0,
                    generation,
                    min_visits,
                    false,
                    label_sink.clone(),
                )?;
                second_mcts_ult_ttt.tree.prune_to_root();
            }

            tracing::debug!(turn_counter, "Starting iteration of second.");
            let start = Instant::now();
            while start.elapsed() < turn_duration {
                second_mcts_ult_ttt.iterate();
            }
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
    Ok((
        UltTTTMCTSGame::evaluate(&first_ult_ttt_game_data, &mut first_mcts_ult_ttt.game_cache)
            .unwrap() as f64,
        first_mcts_ult_ttt,
        second_mcts_ult_ttt,
    ))
}

pub fn run_training<H, UC, UP, EP, SP, S>(
    first_mcts_ult_ttt: UltMCTSTraining<H, UC, UP, EP, SP>,
    second_mcts_ult_ttt: UltMCTSTraining<H, UC, UP, EP, SP>,
    turn_duration: Duration,
    generation: u32,
    min_visits: usize,
    label_sink: S,
    num_matches: usize,
) -> Result<()>
where
    H: Heuristic<UltTTTMCTSGame>,
    UC: UTCCache<UltTTTMCTSGame, UP, UltTTTMCTSConfig>,
    UP: UCTPolicy<UltTTTMCTSGame, UltTTTMCTSConfig>,
    EP: ExpansionPolicy<UltTTTMCTSGame, H, UltTTTMCTSConfig>,
    SP: SimulationPolicy<UltTTTMCTSGame, H, UltTTTMCTSConfig>,
    S: LabelSink,
{
    let match_counter = Arc::new(AtomicUsize::new(1));
    let shared_error = SharedError::new();
    (0..num_matches).into_par_iter().for_each(|_| {
        if shared_error.is_set() {
            return;
        }
        let match_number = match_counter.fetch_add(1, Ordering::Relaxed);
        let id = Uuid::new_v4().to_string();
        let match_span = span!(Level::INFO, "Training match", generation, id, match_number);
        let _match_enter = match_span.enter();

        info!(
            "Starting Training match {} of {}",
            match_number, num_matches,
        );
        let (_, first, second) = match run_training_match(
            first_mcts_ult_ttt.clone(),
            second_mcts_ult_ttt.clone(),
            turn_duration,
            generation,
            min_visits,
            label_sink.clone(),
        ) {
            Ok(res) => res,
            Err(e) => {
                tracing::error!(error = %e, "Failed to log selected parent");
                shared_error.set_if_empty(e);
                return;
            }
        };
        info!(
            "Collecting final labels of first tree of match {}",
            match_number
        );
        if let Err(e) = collect_labels(
            &first.tree,
            0,
            generation,
            min_visits,
            true,
            label_sink.clone(),
        ) {
            tracing::error!(error = %e, "Failed to log selected parent");
            shared_error.set_if_empty(e);
            return;
        }
        info!(
            "Collecting final labels of second tree of match {}",
            match_number
        );
        if let Err(e) = collect_labels(
            &second.tree,
            0,
            generation,
            min_visits,
            true,
            label_sink.clone(),
        ) {
            tracing::error!(error = %e, "Failed to log selected parent");
            shared_error.set_if_empty(e);
            return;
        }
        info!(
            "Finished Training match {} of {}",
            match_number, num_matches,
        );
    });
    Ok(())
}
