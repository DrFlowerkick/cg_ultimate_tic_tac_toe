// heuristic of UltTTT

use super::*;
pub struct OldUltTTTHeuristic {}

impl OldUltTTTHeuristic {
    pub fn is_direct_loss(
        player: TicTacToeStatus,
        my_threats: usize,
        my_meta_threats: u8,
        opp_threats: usize,
        opp_meta_threats: u8,
    ) -> bool {
        // direct loss -> look at threats of other player
        match player {
            TicTacToeStatus::Me => opp_threats > 0 && opp_meta_threats > 0,
            TicTacToeStatus::Opp => my_threats > 0 && my_meta_threats > 0,
            _ => unreachable!("Player is alway Me or Opp"),
        }
    }
}

impl<GC: UltTTTGameCacheTrait + GameCache<UltTTT, UltTTTMove>> Heuristic<UltTTTMCTSGame<GC>>
    for OldUltTTTHeuristic
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

                // mini board threats, weighted with cell_weight, meta factor and constraint factor
                let mut my_threat_sum = 0.0;
                let mut opp_threat_sum = 0.0;
                for (status_index, _) in state.status_map.iter_map().filter(|(_, c)| c.is_vacant())
                {
                    // mini board threats
                    let (my_threats, opp_threats) =
                        game_cache.get_board_threats(state.map.get_cell(status_index));
                    // cell weight
                    let cell_weight = status_index.cell_weight();
                    // meta factors
                    let (
                        my_meta_threats,
                        my_meta_small_threats,
                        opp_meta_threats,
                        opp_meta_small_threats,
                    ) = game_cache.get_meta_cell_threats(&state.status_map, status_index);
                    let my_meta_factor = 1.0
                        + heuristic_config.meta_cell_big_threat * my_meta_threats as f32
                        + heuristic_config.meta_cell_small_threat * my_meta_small_threats as f32;
                    let opp_meta_factor = 1.0
                        + heuristic_config.meta_cell_big_threat * opp_meta_threats as f32
                        + heuristic_config.meta_cell_small_threat * opp_meta_small_threats as f32;
                    // constraint factor
                    let constraint_factor = match state.next_action_constraint {
                        NextActionConstraint::MiniBoard(next_board) => {
                            if status_index == next_board {
                                if OldUltTTTHeuristic::is_direct_loss(
                                    state.last_player,
                                    my_threats.len(),
                                    my_meta_threats,
                                    opp_threats.len(),
                                    opp_meta_threats,
                                ) {
                                    heuristic_cache.insert_intermediate_score(
                                        state,
                                        heuristic_config.direct_loss_value,
                                    );
                                    return if perspective_is_last_player {
                                        heuristic_config.direct_loss_value
                                    } else {
                                        1.0 - heuristic_config.direct_loss_value
                                    };
                                }
                                heuristic_config.constraint_factor
                            } else {
                                1.0
                            }
                        }
                        NextActionConstraint::None => {
                            if OldUltTTTHeuristic::is_direct_loss(
                                state.last_player,
                                my_threats.len(),
                                my_meta_threats,
                                opp_threats.len(),
                                opp_meta_threats,
                            ) {
                                heuristic_cache.insert_intermediate_score(
                                    state,
                                    heuristic_config.direct_loss_value,
                                );
                                return if perspective_is_last_player {
                                    heuristic_config.direct_loss_value
                                } else {
                                    1.0 - heuristic_config.direct_loss_value
                                };
                            }
                            heuristic_config.free_choice_constraint_factor
                        }
                        NextActionConstraint::Init => {
                            unreachable!("Init is reserved for initial tree root node.")
                        }
                    };
                    // constraint factor is applied to current_player, because NextActionConstraint constrains
                    // next moves of current_player.
                    let (my_constraint_factor, opp_constraint_factor) = match state.current_player {
                        TicTacToeStatus::Me => (constraint_factor, 1.0),
                        TicTacToeStatus::Opp => (1.0, constraint_factor),
                        _ => unreachable!("Only Me and Opp are allowed for player."),
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

                // meta progress: wins on status_map
                let (my_wins, opp_wins, played_cells) =
                    game_cache.get_board_progress(&state.status_map);

                // calculate heuristic value
                let progress = played_cells as f32 / 9.0;
                let meta_weight = heuristic_config.control_base_weight
                    + heuristic_config.control_progress_offset * progress;
                let threat_weight = 1.0 - meta_weight;
                let max_threat_score = (my_threat_sum + opp_threat_sum).max(1.0);

                let final_score = 0.5
                    + 0.5 * meta_weight * (my_wins as f32 - opp_wins as f32) / 9.0
                    + 0.5 * threat_weight * (my_threat_sum - opp_threat_sum) / max_threat_score;

                final_score.clamp(0.0, 1.0)
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
        OldUltTTTHeuristic::evaluate_state(
            &new_state,
            game_cache,
            heuristic_cache,
            None,
            heuristic_config,
        )
    }
}
