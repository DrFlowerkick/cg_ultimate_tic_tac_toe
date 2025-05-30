// heuristic of UltTTT

use super::*;
use std::collections::HashSet;

pub struct UltTTTHeuristic {}

impl UltTTTHeuristic {
    pub fn get_constraint_factors(
        last_player: TicTacToeStatus,
        my_threats_of_mini_board: &HashSet<CellIndex3x3>,
        my_meta_threats: &HashSet<CellIndex3x3>,
        opp_threats_of_mini_board: &HashSet<CellIndex3x3>,
        opp_meta_threats: &HashSet<CellIndex3x3>,
        mini_board_index: CellIndex3x3,
        constraint_factor: f32,
    ) -> Option<(f32, f32)> {
        // check direct loss -> look at threats of other player
        if match last_player {
            TicTacToeStatus::Me => {
                !opp_threats_of_mini_board.is_empty()
                    && opp_meta_threats.contains(&mini_board_index)
            }
            TicTacToeStatus::Opp => {
                !my_threats_of_mini_board.is_empty() && my_meta_threats.contains(&mini_board_index)
            }
            _ => unreachable!("Player is always Me or Opp"),
        } {
            // direct loss -> no constraint factors
            return None;
        }

        match last_player {
            TicTacToeStatus::Me => {
                let my_threat_overlap_ratio =
                    UltTTTHeuristic::get_threat_overlap_ratio_for_last_player(
                        my_meta_threats,
                        opp_threats_of_mini_board,
                    );
                Some((
                    1.0 + my_threat_overlap_ratio * constraint_factor,
                    1.0 + (1.0 - my_threat_overlap_ratio) * constraint_factor,
                ))
            }
            TicTacToeStatus::Opp => {
                let opp_threat_overlap_ratio =
                    UltTTTHeuristic::get_threat_overlap_ratio_for_last_player(
                        opp_meta_threats,
                        my_threats_of_mini_board,
                    );
                Some((
                    1.0 + (1.0 - opp_threat_overlap_ratio) * constraint_factor,
                    1.0 + opp_threat_overlap_ratio * constraint_factor,
                ))
            }
            _ => unreachable!("Player is always Me or Opp"),
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
    pub fn normalized_tanh(my_score: f32, opp_score: f32, steepness: f32) -> f32 {
        // tanh is in range [-1.0, 1.0]
        // so we normalize it to [0.0, 1.0]
        // use steepness to control the steepness of the tanh curve
        let delta_score = steepness * (my_score - opp_score);
        (delta_score.tanh() + 1.0) / 2.0
    }
}

impl<GC: UltTTTGameCacheTrait + GameCache<UltTTT, UltTTTMove>> Heuristic<UltTTTMCTSGame<GC>>
    for UltTTTHeuristic
{
    type Cache = NoHeuristicCache<UltTTT, UltTTTMove>;
    type Config = UltTTTHeuristicConfig;

    fn evaluate_state(
        state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
        game_cache: &mut <UltTTTMCTSGame<GC> as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
        perspective_player: Option<<UltTTTMCTSGame<GC> as MCTSGame>::Player>,
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
                let mut my_control_sum = 0.0;
                let mut opp_control_sum = 0.0;

                // mini board threats, weighted with cell_weight, meta factor and constraint factor
                let mut my_threat_sum = 0.0;
                let mut opp_threat_sum = 0.0;

                // threats on status map
                let (my_meta_threats, opp_meta_threats) =
                    game_cache.get_board_threats(&state.status_map);

                for (status_index, status) in state.status_map.iter_map() {
                    match status {
                        TicTacToeStatus::Tie => {
                            // tie mini board, no control, no threats
                            continue;
                        }
                        TicTacToeStatus::Me => {
                            // my mini board, full control by me, no threats
                            my_control_sum += status_index.cell_weight();
                        }
                        TicTacToeStatus::Opp => {
                            // opponent mini board, full control by opponent, no threats
                            opp_control_sum += status_index.cell_weight();
                        }
                        TicTacToeStatus::Vacant => {
                            // vacant mini board: get control and threats
                            // mini board control
                            let (my_control, opp_control) =
                                game_cache.get_board_control(state.map.get_cell(status_index));
                            let my_control_score = UltTTTHeuristic::normalized_tanh(
                                my_control,
                                opp_control,
                                heuristic_config.control_local_steepness,
                            );
                            let opp_control_score = 1.0 - my_control_score;
                            my_control_sum += my_control_score * status_index.cell_weight();
                            opp_control_sum += opp_control_score * status_index.cell_weight();

                            // mini board threats
                            let (my_threats, opp_threats) =
                                game_cache.get_board_threats(state.map.get_cell(status_index));
                            // cell weight
                            let cell_weight = status_index.cell_weight();
                            // meta factors
                            let (
                                num_my_meta_threats,
                                mum_my_meta_small_threats,
                                num_opp_meta_threats,
                                num_opp_meta_small_threats,
                            ) = game_cache.get_meta_cell_threats(&state.status_map, status_index);
                            let my_meta_factor = 1.0
                                + heuristic_config.meta_cell_big_threat
                                    * num_my_meta_threats as f32
                                + heuristic_config.meta_cell_small_threat
                                    * mum_my_meta_small_threats as f32;
                            let opp_meta_factor = 1.0
                                + heuristic_config.meta_cell_big_threat
                                    * num_opp_meta_threats as f32
                                + heuristic_config.meta_cell_small_threat
                                    * num_opp_meta_small_threats as f32;
                            // constraint factors
                            let (my_constraint_factor, opp_constraint_factor) = match state
                                .next_action_constraint
                            {
                                NextActionConstraint::MiniBoard(next_board) => {
                                    if status_index == next_board {
                                        match UltTTTHeuristic::get_constraint_factors(
                                            state.last_player,
                                            &my_threats,
                                            &my_meta_threats,
                                            &opp_threats,
                                            &opp_meta_threats,
                                            status_index,
                                            heuristic_config.constraint_factor,
                                        ) {
                                            Some((my_factor, opp_factor)) => {
                                                (my_factor, opp_factor)
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
                                        &my_threats,
                                        &my_meta_threats,
                                        &opp_threats,
                                        &opp_meta_threats,
                                        status_index,
                                        heuristic_config.free_choice_constraint_factor,
                                    ) {
                                        Some((my_factor, opp_factor)) => (my_factor, opp_factor),
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
                            my_threat_sum += my_constraint_factor
                                * my_meta_factor
                                * cell_weight
                                * my_threats.len() as f32;
                            opp_threat_sum += opp_constraint_factor
                                * opp_meta_factor
                                * cell_weight
                                * opp_threats.len() as f32;
                        }
                    }
                }

                // meta progress: wins on status_map
                let (_, _, played_cells) = game_cache.get_board_progress(&state.status_map);

                // calculate heuristic value
                let progress = played_cells as f32 / 9.0;
                let control_weight = heuristic_config.control_base_weight
                    + heuristic_config.control_progress_offset * progress;
                let threat_weight = 1.0 - control_weight;
                // calculate final score
                control_weight
                    * UltTTTHeuristic::normalized_tanh(
                        my_control_sum,
                        opp_control_sum,
                        heuristic_config.control_global_steepness,
                    )
                    + threat_weight
                        * UltTTTHeuristic::normalized_tanh(
                            my_threat_sum,
                            opp_threat_sum,
                            heuristic_config.threat_steepness,
                        )
            }
        };
        // score is calculated from perspective of me
        // --> invert score, if last_player is opp
        let score = match state.last_player {
            TicTacToeStatus::Me => score,
            TicTacToeStatus::Opp => 1.0 - score,
            _ => unreachable!("Player is alway Me or Opp"),
        };
        heuristic_cache.insert_intermediate_score(state, score);
        if perspective_is_last_player {
            score
        } else {
            1.0 - score
        }
    }

    fn evaluate_move(
        state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
        mv: &<UltTTTMCTSGame<GC> as MCTSGame>::Move,
        game_cache: &mut <UltTTTMCTSGame<GC> as MCTSGame>::Cache,
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
