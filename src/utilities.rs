// utilities for optimization

use super::*;
use std::time::{Duration, Instant};

const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(995);
const TIME_OUT_SUCCESSIVE_TURNS: Duration = Duration::from_millis(95);

type UltTTTExpandAll = ExpandAll<UltTTTMCTSGame<NoGameCache<UltTTT, UltTTTMove>>>;

pub fn run_match(config: Config, heuristic_is_start_player: bool) -> f32 {
    let mut first_mcts_ult_ttt: PlainMCTS<
        UltTTTMCTSGameNoGameCache,
        DynamicC,
        CachedUTC,
        HPWDefaultTTTNoGameCache,
        UltTTTHeuristic,
        HeuristicCutoff,
    > = PlainMCTS::new(config.mcts, config.heuristic);
    let mut first_ult_ttt_game_data = UltTTT::new();
    let mut first_time_out = TIME_OUT_FIRST_TURN;
    let mut second_mcts_ult_ttt: PlainMCTS<
        UltTTTMCTSGameNoGameCache,
        DynamicC,
        CachedUTC,
        UltTTTExpandAll,
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

#[derive(Debug, Clone, Copy, Default)]
pub struct Config {
    pub mcts: UltTTTMCTSConfig,
    pub heuristic: UltTTTHeuristicConfig,
}

impl From<Vec<f64>> for Config {
    fn from(value: Vec<f64>) -> Self {
        Config {
            mcts: UltTTTMCTSConfig {
                base_config: BaseConfig {
                    exploration_constant: value[0] as f32,
                    progressive_widening_constant: value[1] as f32,
                    progressive_widening_exponent: value[2] as f32,
                    early_cut_off_depth: value[3].round() as usize,
                },
            },
            heuristic: UltTTTHeuristicConfig {
                base_config: BaseHeuristicConfig {
                    progressive_widening_initial_threshold: value[4] as f32,
                    progressive_widening_decay_rate: value[5] as f32,
                    early_cut_off_lower_bound: value[6] as f32,
                    early_cut_off_upper_bound: value[7] as f32,
                },
                meta_weight_base: value[8] as f32,
                meta_weight_progress_offset: value[9] as f32,
                meta_cell_big_threat: value[10] as f32,
                meta_cell_small_threat: value[11] as f32,
                constraint_factor: value[12] as f32,
                free_choice_constraint_factor: value[13] as f32,
                direct_loss_value: value[14] as f32,
            },
        }
    }
}

impl From<Config> for Vec<f64> {
    fn from(value: Config) -> Self {
        vec![
            value.mcts.base_config.exploration_constant as f64,
            value.mcts.base_config.progressive_widening_constant as f64,
            value.mcts.base_config.progressive_widening_exponent as f64,
            value.mcts.base_config.early_cut_off_depth as f64,
            value
                .heuristic
                .base_config
                .progressive_widening_initial_threshold as f64,
            value.heuristic.base_config.progressive_widening_decay_rate as f64,
            value.heuristic.base_config.early_cut_off_lower_bound as f64,
            value.heuristic.base_config.early_cut_off_upper_bound as f64,
            value.heuristic.meta_weight_base as f64,
            value.heuristic.meta_weight_progress_offset as f64,
            value.heuristic.meta_cell_big_threat as f64,
            value.heuristic.meta_cell_small_threat as f64,
            value.heuristic.constraint_factor as f64,
            value.heuristic.free_choice_constraint_factor as f64,
            value.heuristic.direct_loss_value as f64,
        ]
    }
}

impl Config {
    pub fn parameter_names() -> Vec<String> {
        vec![
            "exploration_constant".into(),
            "progressive_widening_constant".into(),
            "progressive_widening_exponent".into(),
            "early_cut_off_depth".into(),
            "progressive_widening_initial_threshold".into(),
            "progressive_widening_decay_rate".into(),
            "early_cut_off_lower_bound".into(),
            "early_cut_off_upper_bound".into(),
            "meta_weight_base".into(),
            "meta_weight_progress_offset".into(),
            "meta_cell_big_threat".into(),
            "meta_cell_small_threat".into(),
            "constraint_factor".into(),
            "free_choice_constraint_factor".into(),
            "direct_loss_value".into(),
        ]
    }
    pub fn lower_bounds() -> Self {
        Config {
            mcts: UltTTTMCTSConfig {
                base_config: BaseConfig {
                    exploration_constant: 1.0,
                    progressive_widening_constant: 1.0,
                    progressive_widening_exponent: 1.0 / 3.0,
                    early_cut_off_depth: 15,
                },
            },
            heuristic: UltTTTHeuristicConfig {
                base_config: BaseHeuristicConfig {
                    progressive_widening_initial_threshold: 0.6,
                    progressive_widening_decay_rate: 0.8,
                    early_cut_off_lower_bound: 0.0,
                    early_cut_off_upper_bound: 0.9,
                },
                meta_weight_base: 0.3,
                meta_weight_progress_offset: 0.2,
                meta_cell_big_threat: 2.0,
                meta_cell_small_threat: 0.5,
                constraint_factor: 1.0,
                free_choice_constraint_factor: 1.0,
                direct_loss_value: 0.0,
            },
        }
    }

    pub fn upper_bounds() -> Self {
        Config {
            mcts: UltTTTMCTSConfig {
                base_config: BaseConfig {
                    exploration_constant: 2.0,
                    progressive_widening_constant: 4.0,
                    progressive_widening_exponent: 2.0 / 3.0,
                    early_cut_off_depth: 35,
                },
            },
            heuristic: UltTTTHeuristicConfig {
                base_config: BaseHeuristicConfig {
                    progressive_widening_initial_threshold: 0.9,
                    progressive_widening_decay_rate: 1.0,
                    early_cut_off_lower_bound: 0.1,
                    early_cut_off_upper_bound: 1.0,
                },
                meta_weight_base: 0.6,
                meta_weight_progress_offset: 0.4,
                meta_cell_big_threat: 4.0,
                meta_cell_small_threat: 1.5,
                constraint_factor: 2.0,
                free_choice_constraint_factor: 1.0,
                direct_loss_value: 0.025,
            },
        }
    }
}
