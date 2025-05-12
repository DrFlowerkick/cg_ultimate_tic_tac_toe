// heuristic of UltTTT

use super::*;
pub struct UltTTTHeuristic {}

impl UltTTTHeuristic {
    pub fn is_direct_loss(
        player: TicTacToeStatus,
        my_threats: usize,
        my_meta_threats: i8,
        opp_threats: usize,
        opp_meta_threats: i8,
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
        if let Some(score) = heuristic_cache.get_intermediate_score(state) {
            return score;
        }
        // Check direct loss for perspective player only, if he is last_player.
        // If perspective_player is current_player, he is next to move and cannot loose directly.
        let (player, check_direct_loss) = match perspective_player {
            Some(player) => (player, player == state.last_player),
            None => (state.last_player, true),
        };
        let score = match UltTTTMCTSGame::evaluate(state, game_cache) {
            Some(value) => value,
            None => {
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
                    let my_meta_factor = heuristic_config.meta_cell_big_threat
                        * my_meta_threats as f32
                        + heuristic_config.meta_cell_small_threat * my_meta_small_threats as f32;
                    let opp_meta_factor = heuristic_config.meta_cell_big_threat
                        * opp_meta_threats as f32
                        + heuristic_config.meta_cell_small_threat * opp_meta_small_threats as f32;
                    // constraint factor
                    let constraint_factor = match state.next_action_constraint {
                        NextActionConstraint::MiniBoard(next_board) => {
                            if status_index == next_board {
                                if check_direct_loss
                                    && UltTTTHeuristic::is_direct_loss(
                                        player,
                                        my_threats,
                                        my_meta_threats,
                                        opp_threats,
                                        opp_meta_threats,
                                    )
                                {
                                    return heuristic_config.direct_loss_value;
                                }
                                heuristic_config.constraint_factor
                            } else {
                                1.0
                            }
                        }
                        NextActionConstraint::None => {
                            if check_direct_loss
                                && UltTTTHeuristic::is_direct_loss(
                                    player,
                                    my_threats,
                                    my_meta_threats,
                                    opp_threats,
                                    opp_meta_threats,
                                )
                            {
                                return heuristic_config.direct_loss_value;
                            }
                            heuristic_config.free_choice_constraint_factor
                        }
                        NextActionConstraint::Init => {
                            unreachable!("Init is reserved for initial tree root node.")
                        }
                    };
                    let (my_constraint_factor, opp_constraint_factor) = match state.current_player {
                        TicTacToeStatus::Me => (constraint_factor, 1.0),
                        TicTacToeStatus::Opp => (1.0, constraint_factor),
                        _ => unreachable!("Only Me and Opp are allowed for player."),
                    };
                    my_threat_sum +=
                        my_constraint_factor * my_meta_factor * cell_weight * my_threats as f32;
                    opp_threat_sum +=
                        opp_constraint_factor * opp_meta_factor * cell_weight * opp_threats as f32;
                }

                // meta progress: wins on status_map
                let (my_wins, opp_wins) = game_cache.get_board_wins(&state.status_map);

                // calculate heuristic value
                let progress = (my_wins + opp_wins) as f32 / 9.0;
                let meta_weight = heuristic_config.meta_weight_base
                    + heuristic_config.meta_weight_progress_offset * progress;
                let threat_weight = 1.0 - meta_weight;
                let max_threat_score = 1.0_f32.max(my_threat_sum + opp_threat_sum);

                let final_score = 0.5
                    + meta_weight * (my_wins as f32 - opp_wins as f32) / 9.0
                    + threat_weight * (my_threat_sum - opp_threat_sum) / max_threat_score;

                final_score.clamp(0.0, 1.0)
            }
        };

        let score = match player {
            TicTacToeStatus::Me => score,
            TicTacToeStatus::Opp => 1.0 - score,
            _ => unreachable!("Player is alway Me or Opp"),
        };
        heuristic_cache.insert_intermediate_score(state, score);
        score
    }

    fn evaluate_state_recursive(
        state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
        game_cache: &mut <UltTTTMCTSGame<GC> as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
        depth: usize,
        alpha: f32,
    ) -> f32 {
        let base_heuristic =
            Self::evaluate_state(state, game_cache, heuristic_cache, None, heuristic_config);

        if depth == 0 || UltTTTMCTSGame::evaluate(state, game_cache).is_some() {
            return base_heuristic;
        }

        let mut worst_response = f32::NEG_INFINITY;
        let next_player_alpha = alpha
            - (alpha - 0.5) * heuristic_config.evaluate_state_recursive_alpha_reduction_factor;
        // If no constraint on next move, this will be many moves to consider.
        // Therefore we use early exit to reduce calculation time.
        for next_player_move in UltTTTMCTSGame::<GC>::available_moves(state) {
            let next_player_state =
                UltTTTMCTSGame::apply_move(state, &next_player_move, game_cache);

            let response_value = Self::evaluate_state_recursive(
                &next_player_state,
                game_cache,
                heuristic_cache,
                heuristic_config,
                depth - 1,
                next_player_alpha,
            );

            if response_value > worst_response {
                worst_response = response_value;
                // early exit, because next player does have guaranteed win
                if worst_response >= heuristic_config.evaluate_state_recursive_early_exit_threshold
                {
                    break;
                }
            }
        }

        // combine base heuristic with worst case response
        alpha * base_heuristic + (1.0 - alpha) * (1.0 - worst_response)
    }

    fn evaluate_move(
        state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
        mv: &<UltTTTMCTSGame<GC> as MCTSGame>::Move,
        game_cache: &mut <UltTTTMCTSGame<GC> as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> f32 {
        let new_state = UltTTTMCTSGame::apply_move(state, mv, game_cache);
        UltTTTHeuristic::evaluate_state_recursive(
            &new_state,
            game_cache,
            heuristic_cache,
            heuristic_config,
            heuristic_config.evaluate_state_recursive_depth(),
            heuristic_config.evaluate_state_recursive_alpha(),
        )
    }
}
