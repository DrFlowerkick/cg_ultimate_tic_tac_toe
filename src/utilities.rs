// utilities for optimization

use super::*;
use my_lib::my_optimizer::{
    increment_progress_counter_by, update_progress, ObjectiveFunction, ParamBound, ParamDescriptor,
};
use std::time::{Duration, Instant};
use tracing::{span, Level};
use uuid::Uuid;

const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(995);
const TIME_OUT_SUCCESSIVE_TURNS: Duration = Duration::from_millis(95);

type UltTTTExpandAll = ExpandAll<UltTTTMCTSGame<NoGameCache<UltTTT, UltTTTMove>>>;

pub struct EarlyBreakOff {
    pub num_initial_matches: usize,
    pub score_threshold: f64,
}

pub struct UltTTTObjectiveFunction {
    pub num_matches: usize,
    pub early_break_off: Option<EarlyBreakOff>,
    pub progress_step_size: usize,
    pub estimated_num_of_steps: usize,
}

impl ObjectiveFunction for UltTTTObjectiveFunction {
    type Config = Config;

    fn evaluate(&self, config: Config) -> anyhow::Result<f64> {
        let eval_id = Uuid::new_v4().to_string();

        let span_search = span!(
            Level::DEBUG,
            "UltTTT Objective Function",
            eval_id = eval_id,
            ?config
        );
        let _enter = span_search.enter();

        let mut sum_score: f64 = 0.0;
        let mut num_early_matches = 0;
        if let Some(ref ebo) = self.early_break_off {
            sum_score += (0..ebo.num_initial_matches)
                .map(|i| {
                    update_progress(Some(self.estimated_num_of_steps), self.progress_step_size);
                    run_match(config, i % 2 == 0)
                })
                .sum::<f64>();

            let early_score = sum_score / (ebo.num_initial_matches as f64);
            if early_score < ebo.score_threshold {
                increment_progress_counter_by(self.num_matches);
                tracing::debug!(eval_id, early_score, "Evaluation early cut-off.");
                return Ok(early_score);
            }
            num_early_matches = ebo.num_initial_matches;
        }

        sum_score += (0..self.num_matches)
            .map(|i| {
                update_progress(Some(self.estimated_num_of_steps), self.progress_step_size);
                run_match(config, i % 2 == 0)
            })
            .sum::<f64>();
        let score = sum_score / (self.num_matches + num_early_matches) as f64;

        tracing::debug!(eval_id, score, "Evaluation completed.");

        Ok(score)
    }
}

pub fn run_match(config: Config, heuristic_is_start_player: bool) -> f64 {
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
        as f64
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Config {
    pub mcts: UltTTTMCTSConfig,
    pub heuristic: UltTTTHeuristicConfig,
}

impl TryFrom<&[f64]> for Config {
    type Error = anyhow::Error;

    fn try_from(value: &[f64]) -> Result<Self, Self::Error> {
        if value.len() != 15 {
            return Err(anyhow::anyhow!("Wrong number of parameters"));
        }
        Ok(Config {
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
                control_base_weight: value[8] as f32,
                control_progress_offset: value[9] as f32,
                meta_cell_big_threat: value[10] as f32,
                meta_cell_small_threat: value[11] as f32,
                constraint_factor: value[12] as f32,
                free_choice_constraint_factor: value[13] as f32,
                direct_loss_value: value[14] as f32,
            },
        })
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
            value.heuristic.control_base_weight as f64,
            value.heuristic.control_progress_offset as f64,
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
            "control_base_weight".into(),
            "control_progress_offset".into(),
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
                control_base_weight: 0.3,
                control_progress_offset: 0.2,
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
                control_base_weight: 0.6,
                control_progress_offset: 0.4,
                meta_cell_big_threat: 4.0,
                meta_cell_small_threat: 1.5,
                constraint_factor: 2.0,
                free_choice_constraint_factor: 2.0,
                direct_loss_value: 0.025,
            },
        }
    }
    pub fn param_bounds() -> Vec<ParamDescriptor> {
        let lower_bounds: Vec<f64> = Config::lower_bounds().into();
        let upper_bounds: Vec<f64> = Config::upper_bounds().into();
        lower_bounds
            .into_iter()
            .zip(upper_bounds.into_iter())
            .zip(Config::parameter_names().iter())
            .map(|((min, max), name)| ParamDescriptor {
                name: name.to_owned(),
                bound: ParamBound::MinMax(min, max),
            })
            .collect()
    }
}
