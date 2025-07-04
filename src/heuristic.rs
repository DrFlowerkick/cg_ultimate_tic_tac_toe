// heuristic of UltTTT

use super::{NextActionConstraint, UltTTT, UltTTTHeuristicConfig, UltTTTMCTSGame, UltTTTMove};
use my_lib::{
    my_map_3x3::CellIndex3x3,
    my_mcts::{Heuristic, HeuristicCache, MCTSGame, NoHeuristicCache},
    my_tic_tac_toe::TicTacToeStatus,
};
use std::collections::HashSet;

#[derive(Clone)]
pub struct UltTTTHeuristic {}

impl UltTTTHeuristic {
    pub fn get_constraint_factors(
        last_player: TicTacToeStatus,
        first_threats_of_mini_board: &HashSet<CellIndex3x3>,
        first_meta_threats: &HashSet<CellIndex3x3>,
        second_threats_of_mini_board: &HashSet<CellIndex3x3>,
        second_meta_threats: &HashSet<CellIndex3x3>,
        mini_board_index: CellIndex3x3,
        constraint_factor: f32,
    ) -> Option<(f32, f32)> {
        // check direct loss -> look at threats of other player
        if match last_player {
            TicTacToeStatus::First => {
                !second_threats_of_mini_board.is_empty()
                    && second_meta_threats.contains(&mini_board_index)
            }
            TicTacToeStatus::Second => {
                !first_threats_of_mini_board.is_empty()
                    && first_meta_threats.contains(&mini_board_index)
            }
            _ => unreachable!("Player is always First or Second"),
        } {
            // direct loss -> no constraint factors
            return None;
        }

        match last_player {
            TicTacToeStatus::First => {
                let first_threat_overlap_ratio =
                    UltTTTHeuristic::get_threat_overlap_ratio_for_last_player(
                        first_meta_threats,
                        second_threats_of_mini_board,
                    );
                Some((
                    1.0 + first_threat_overlap_ratio * constraint_factor,
                    1.0 + (1.0 - first_threat_overlap_ratio) * constraint_factor,
                ))
            }
            TicTacToeStatus::Second => {
                let second_threat_overlap_ratio =
                    UltTTTHeuristic::get_threat_overlap_ratio_for_last_player(
                        second_meta_threats,
                        first_threats_of_mini_board,
                    );
                Some((
                    1.0 + (1.0 - second_threat_overlap_ratio) * constraint_factor,
                    1.0 + second_threat_overlap_ratio * constraint_factor,
                ))
            }
            _ => unreachable!("Player is always First or Second"),
        }
    }
    fn get_threat_overlap_ratio_for_last_player(
        last_player_meta_threats: &HashSet<CellIndex3x3>,
        current_player_threats: &HashSet<CellIndex3x3>,
    ) -> f32 {
        if current_player_threats.is_empty() {
            // no threats for current player, so no distribution of constraint factor
            return 0.0;
        }
        let num_last_player_back_to_threat_line = last_player_meta_threats
            .intersection(current_player_threats)
            .count();
        num_last_player_back_to_threat_line as f32 / current_player_threats.len() as f32
    }
    pub fn normalized_tanh(first_score: f32, second_score: f32, steepness: f32) -> f32 {
        // tanh is in range [-1.0, 1.0]
        // so we normalize it to [0.0, 1.0]
        // use steepness to control the steepness of the tanh curve
        let delta_score = steepness * (first_score - second_score);
        (delta_score.tanh() + 1.0) / 2.0
    }
}

impl Heuristic<UltTTTMCTSGame> for UltTTTHeuristic {
    type Cache = NoHeuristicCache<UltTTT, UltTTTMove>;
    type Config = UltTTTHeuristicConfig;

    fn evaluate_state(
        state: &<UltTTTMCTSGame as MCTSGame>::State,
        game_cache: &mut <UltTTTMCTSGame as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
        perspective_player: Option<<UltTTTMCTSGame as MCTSGame>::Player>,
        heuristic_config: &Self::Config,
    ) -> f32 {
        // If perspective_player is not last_player, we need to invert heuristic score
        let perspective_is_last_player = match perspective_player {
            Some(player) => player == state.last_player,
            None => true,
        };
        // return cached score if available
        if let Some(score) = heuristic_cache.get_intermediate_score(state) {
            return if perspective_is_last_player {
                score
            } else {
                1.0 - score
            };
        }
        let score = match UltTTTMCTSGame::evaluate(state, game_cache) {
            Some(value) => value,
            None => {
                // ToDo: if status map contains won cells and vacant cells cannot change win status of player with
                // most won cells, than this is a win for player of most won cells. Try to get this in heuristic.

                // mini board control, weighted with cell_weight
                let mut first_control_sum = 0.0;
                let mut second_control_sum = 0.0;

                // mini board threats, weighted with cell_weight, meta factor and constraint factor
                let mut first_threat_sum = 0.0;
                let mut second_threat_sum = 0.0;

                // threats on status map
                let (first_meta_threats, second_meta_threats) = state.status_map.get_threats();

                for (status_index, status) in state.status_map.iter_map() {
                    match status {
                        TicTacToeStatus::Tie => {
                            // tie mini board, no control, no threats
                            continue;
                        }
                        TicTacToeStatus::First => {
                            // first mini board, full control by first, no threats
                            first_control_sum += status_index.cell_weight();
                        }
                        TicTacToeStatus::Second => {
                            // second mini board, full control by second, no threats
                            second_control_sum += status_index.cell_weight();
                        }
                        TicTacToeStatus::Vacant => {
                            // vacant mini board: get control and threats
                            // mini board control
                            let (first_control, second_control) =
                                state.map.get_cell(status_index).get_board_control();
                            let first_control_score = UltTTTHeuristic::normalized_tanh(
                                first_control,
                                second_control,
                                heuristic_config.control_local_steepness,
                            );
                            let second_control_score = 1.0 - first_control_score;
                            first_control_sum += first_control_score * status_index.cell_weight();
                            second_control_sum += second_control_score * status_index.cell_weight();

                            // mini board threats
                            let (first_threats, second_threats) =
                                state.map.get_cell(status_index).get_threats();
                            // cell weight
                            let cell_weight = status_index.cell_weight();
                            // meta factors
                            let (
                                num_first_meta_threats,
                                num_first_meta_small_threats,
                                num_second_meta_threats,
                                num_second_meta_small_threats,
                            ) = state.status_map.get_meta_cell_threats(status_index);
                            let first_meta_factor = 1.0
                                + heuristic_config.meta_cell_big_threat
                                    * num_first_meta_threats as f32
                                + heuristic_config.meta_cell_small_threat
                                    * num_first_meta_small_threats as f32;
                            let second_meta_factor = 1.0
                                + heuristic_config.meta_cell_big_threat
                                    * num_second_meta_threats as f32
                                + heuristic_config.meta_cell_small_threat
                                    * num_second_meta_small_threats as f32;
                            // constraint factors
                            let (first_constraint_factor, second_constraint_factor) =
                                match state.next_action_constraint {
                                    NextActionConstraint::MiniBoard(next_board) => {
                                        if status_index == next_board {
                                            match UltTTTHeuristic::get_constraint_factors(
                                                state.last_player,
                                                &first_threats,
                                                &first_meta_threats,
                                                &second_threats,
                                                &second_meta_threats,
                                                status_index,
                                                heuristic_config.constraint_factor,
                                            ) {
                                                Some((first_factor, second_factor)) => {
                                                    (first_factor, second_factor)
                                                }
                                                None => {
                                                    // direct loss
                                                    return if perspective_is_last_player {
                                                        heuristic_config.direct_loss_value
                                                    } else {
                                                        1.0 - heuristic_config.direct_loss_value
                                                    };
                                                }
                                            }
                                        } else {
                                            // no constraint factors for other mini boards
                                            (1.0, 1.0)
                                        }
                                    }
                                    NextActionConstraint::None => {
                                        match UltTTTHeuristic::get_constraint_factors(
                                            state.last_player,
                                            &first_threats,
                                            &first_meta_threats,
                                            &second_threats,
                                            &second_meta_threats,
                                            status_index,
                                            heuristic_config.free_choice_constraint_factor,
                                        ) {
                                            Some((first_factor, second_factor)) => {
                                                (first_factor, second_factor)
                                            }
                                            None => {
                                                // direct loss
                                                return if perspective_is_last_player {
                                                    heuristic_config.direct_loss_value
                                                } else {
                                                    1.0 - heuristic_config.direct_loss_value
                                                };
                                            }
                                        }
                                    }
                                    NextActionConstraint::Init => {
                                        unreachable!("Init is reserved for initial tree root node.")
                                    }
                                };
                            first_threat_sum += first_constraint_factor
                                * first_meta_factor
                                * cell_weight
                                * first_threats.len() as f32;
                            second_threat_sum += second_constraint_factor
                                * second_meta_factor
                                * cell_weight
                                * second_threats.len() as f32;
                        }
                    }
                }

                // meta progress: wins on status_map
                let played_cells = state.status_map.count_non_vacant_cells();

                // calculate heuristic value
                let progress = played_cells as f32 / 9.0;
                let control_weight = heuristic_config.control_base_weight
                    + heuristic_config.control_progress_offset * progress;
                let threat_weight = 1.0 - control_weight;
                // calculate final score
                control_weight
                    * UltTTTHeuristic::normalized_tanh(
                        first_control_sum,
                        second_control_sum,
                        heuristic_config.control_global_steepness,
                    )
                    + threat_weight
                        * UltTTTHeuristic::normalized_tanh(
                            first_threat_sum,
                            second_threat_sum,
                            heuristic_config.threat_steepness,
                        )
            }
        };
        // score is calculated from perspective of first
        // --> invert score, if last_player is second
        let score = match state.last_player {
            TicTacToeStatus::First => score,
            TicTacToeStatus::Second => 1.0 - score,
            _ => unreachable!("Player is always First or Second"),
        };
        heuristic_cache.insert_intermediate_score(state, score);
        if perspective_is_last_player {
            score
        } else {
            1.0 - score
        }
    }

    fn evaluate_move(
        state: &<UltTTTMCTSGame as MCTSGame>::State,
        mv: &<UltTTTMCTSGame as MCTSGame>::Move,
        game_cache: &mut <UltTTTMCTSGame as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> f32 {
        let new_state = UltTTTMCTSGame::apply_move(state, mv, game_cache);
        UltTTTHeuristic::evaluate_state(
            &new_state,
            game_cache,
            heuristic_cache,
            None,
            heuristic_config,
        )
    }
}
