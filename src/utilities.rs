// utilities for optimization

use super::{
    HPWDefaultTTTNoGameCache, TicTacToeStatus, UltTTT, UltTTTHeuristic, UltTTTHeuristicConfig,
    UltTTTMCTSConfig, UltTTTMCTSGame,
};
use anyhow::Context;
use my_lib::my_mcts::{
    BaseConfig, BaseHeuristicConfig, CachedUTC, DynamicC, DynamicCWithExplorationBoost,
    HeuristicCutoff, MCTSAlgo, MCTSConfig, MCTSGame, PlainMCTS, PlainTTHashMap,
};
use my_lib::my_optimizer::{
    increment_progress_counter_by, update_progress, LogFormat, ObjectiveFunction, ParamBound,
    ParamDescriptor,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use tracing::{span, Level};
use uuid::Uuid;

const TIME_OUT_TREE_BUILD_UP: Duration = Duration::from_millis(2500);
const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(990);
const TIME_OUT_OPP_PERSPECTIVE: Duration = Duration::from_millis(80);
const TIME_OUT_ME_PERSPECTIVE: Duration = Duration::from_millis(85);
const EXPECTED_NUM_NODES: usize = 200_000;

pub struct EarlyBreakOff {
    pub num_check_matches: usize,
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

        let span_search = span!(Level::DEBUG, "UltTTT Objective Function", eval_id = eval_id,);
        let _enter = span_search.enter();
        match LogFormat::get_global() {
            Some(LogFormat::Json) => {
                let json = serde_json::to_string(&config)
                    .context("Failed to serialize candidate to JSON")?;
                tracing::debug!(
                    config = %json,
                    "Starting evaluation of UltTTTObjectiveFunction"
                );
            }
            Some(LogFormat::PlainText) => {
                tracing::debug!(
            config = ?config, "Starting evaluation of UltTTTObjectiveFunction");
            }
            None => {
                println!(
                    "Starting evaluation of UltTTTObjectiveFunction with config: {:?}",
                    config
                );
            }
        }

        let mut sum_score: f64 = 0.0;
        for i in 0..self.num_matches {
            update_progress(Some(self.estimated_num_of_steps), self.progress_step_size);
            let (score, _, _) = run_match(config.clone(), i % 2 == 0);
            sum_score += score;
            if let Some(ref ebo) = self.early_break_off {
                let count_matches = i + 1;
                if count_matches % ebo.num_check_matches == 0 && count_matches < self.num_matches {
                    let early_score = sum_score / (count_matches as f64);
                    let expected_threshold = ebo.score_threshold
                        - 0.1 * (1.0 - count_matches as f64 / self.num_matches as f64);
                    if early_score < expected_threshold {
                        increment_progress_counter_by(self.num_matches - count_matches);
                        tracing::debug!(eval_id, early_score, "Evaluation early cut-off.");
                        return Ok(early_score);
                    }
                }
            }
        }

        let score = sum_score / self.num_matches as f64;

        tracing::debug!(eval_id, score, "Evaluation completed.");

        Ok(score)
    }
}

pub type UltTTTMCTSFirst = PlainMCTS<
    UltTTTMCTSGame,
    UltTTTHeuristic,
    UltTTTMCTSConfig,
    CachedUTC,
    PlainTTHashMap<UltTTT>,
    DynamicCWithExplorationBoost,
    HPWDefaultTTTNoGameCache,
    HeuristicCutoff,
>;
pub type UltTTTMCTSSecond = PlainMCTS<
    UltTTTMCTSGame,
    UltTTTHeuristic,
    UltTTTMCTSConfig,
    CachedUTC,
    PlainTTHashMap<UltTTT>,
    DynamicC,
    HPWDefaultTTTNoGameCache,
    HeuristicCutoff,
>;

// structure of run_match() tries to represent timing on codingame, which was measured with debug messages
// 1.) first turn: long time out
// 2.) than iterate from perspective of opponent (about 70 ms)
// 3.) than iterate frm my perspective (about 70 ms)
// 4.) if not terminal, go to 2.)
// Since we have here two MCTS players, both players get same timings

pub fn run_match(
    config: Config,
    heuristic_is_start_player: bool,
) -> (f64, UltTTTMCTSFirst, UltTTTMCTSSecond) {
    // Initial config without exploration_boost
    let mut initial_config = config.mcts.clone();
    initial_config.base_config.exploration_boost = [
        (TicTacToeStatus::First, 1.0),
        (TicTacToeStatus::Second, 1.0),
    ]
    .into();

    let mut first_mcts_ult_ttt: UltTTTMCTSFirst =
        PlainMCTS::new(initial_config, config.heuristic, EXPECTED_NUM_NODES);
    let mut first_ult_ttt_game_data = UltTTT::new();
    first_mcts_ult_ttt.set_root(&first_ult_ttt_game_data);
    let mut first_time_out = TIME_OUT_FIRST_TURN;
    let mut second_mcts_ult_ttt: UltTTTMCTSSecond = PlainMCTS::new(
        UltTTTMCTSConfig::new_optimized(),
        UltTTTHeuristicConfig::new_optimized(),
        EXPECTED_NUM_NODES,
    );
    let mut second_ult_ttt_game_data = UltTTT::new();
    second_mcts_ult_ttt.set_root(&second_ult_ttt_game_data);
    let mut second_time_out = TIME_OUT_FIRST_TURN;

    // player first is always heuristic player, but only every second game start player
    let mut first = if heuristic_is_start_player {
        true
    } else {
        first_ult_ttt_game_data.set_current_player(TicTacToeStatus::Second);
        second_ult_ttt_game_data.set_current_player(TicTacToeStatus::Second);
        false
    };

    // initial tree build up before codingame sends first initial input
    // first first
    let start = Instant::now();
    while start.elapsed() < TIME_OUT_TREE_BUILD_UP {
        first_mcts_ult_ttt.iterate();
    }
    // apply exploration boost to config of first
    first_mcts_ult_ttt.mcts_config.base_config.exploration_boost =
        config.mcts.base_config.exploration_boost;
    // second second
    let start = Instant::now();
    while start.elapsed() < TIME_OUT_TREE_BUILD_UP {
        second_mcts_ult_ttt.iterate();
    }

    let mut turn_counter = 0;
    while UltTTTMCTSGame::evaluate(&first_ult_ttt_game_data, &mut first_mcts_ult_ttt.game_cache)
        .is_none()
    {
        if first {
            // iterate first tree from first perspective
            if turn_counter > 0 && !first_mcts_ult_ttt.set_root(&first_ult_ttt_game_data) {
                tracing::debug!(
                    heuristic_is_start_player,
                    turn_counter,
                    "Reset tree root of first."
                );
            }
            let start = Instant::now();
            while start.elapsed() < first_time_out {
                first_mcts_ult_ttt.iterate();
            }
            first_time_out = TIME_OUT_ME_PERSPECTIVE;
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
            if UltTTTMCTSGame::evaluate(
                &first_ult_ttt_game_data,
                &mut first_mcts_ult_ttt.game_cache,
            )
            .is_none()
            {
                // if not terminal, iterate first tree from second perspective
                first_mcts_ult_ttt.set_root(&first_ult_ttt_game_data);
                let start = Instant::now();
                while start.elapsed() < TIME_OUT_OPP_PERSPECTIVE {
                    first_mcts_ult_ttt.iterate();
                }
            }
            first = false;
        } else {
            // iterate second tree from second perspective
            if turn_counter > 0 && !second_mcts_ult_ttt.set_root(&second_ult_ttt_game_data) {
                tracing::debug!(
                    heuristic_is_start_player,
                    turn_counter,
                    "Reset tree root of second."
                );
            }
            let start = Instant::now();
            while start.elapsed() < second_time_out {
                second_mcts_ult_ttt.iterate();
            }
            second_time_out = TIME_OUT_ME_PERSPECTIVE;
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
            if UltTTTMCTSGame::evaluate(
                &first_ult_ttt_game_data,
                &mut first_mcts_ult_ttt.game_cache,
            )
            .is_none()
            {
                // if not terminal, iterate second tree from first perspective
                let start = Instant::now();
                second_mcts_ult_ttt.set_root(&second_ult_ttt_game_data);
                while start.elapsed() < TIME_OUT_OPP_PERSPECTIVE {
                    second_mcts_ult_ttt.iterate();
                }
            }
            first = true;
        }
        turn_counter += 1;
    }
    (
        UltTTTMCTSGame::evaluate(&first_ult_ttt_game_data, &mut first_mcts_ult_ttt.game_cache)
            .unwrap() as f64,
        first_mcts_ult_ttt,
        second_mcts_ult_ttt,
    )
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Config {
    pub mcts: UltTTTMCTSConfig,
    pub heuristic: UltTTTHeuristicConfig,
}

impl TryFrom<&[f64]> for Config {
    type Error = anyhow::Error;

    fn try_from(value: &[f64]) -> Result<Self, Self::Error> {
        if value.len() != 20 {
            return Err(anyhow::anyhow!("Wrong number of parameters"));
        }
        Ok(Config {
            mcts: UltTTTMCTSConfig {
                base_config: BaseConfig {
                    exploration_constant: value[0] as f32,
                    exploration_boost: [
                        (TicTacToeStatus::First, value[1] as f32),
                        (TicTacToeStatus::Second, value[2] as f32),
                    ]
                    .into(),
                    progressive_widening_constant: value[3] as f32,
                    progressive_widening_exponent: value[4] as f32,
                    early_cut_off_depth: value[5].round() as usize,
                },
            },
            heuristic: UltTTTHeuristicConfig {
                base_config: BaseHeuristicConfig {
                    progressive_widening_initial_threshold: value[6] as f32,
                    progressive_widening_decay_rate: value[7] as f32,
                    early_cut_off_lower_bound: value[8] as f32,
                    early_cut_off_upper_bound: value[9] as f32,
                },
                control_base_weight: value[10] as f32,
                control_progress_offset: value[11] as f32,
                control_local_steepness: value[12] as f32,
                control_global_steepness: value[13] as f32,
                meta_cell_big_threat: value[14] as f32,
                meta_cell_small_threat: value[15] as f32,
                threat_steepness: value[16] as f32,
                constraint_factor: value[17] as f32,
                free_choice_constraint_factor: value[18] as f32,
                direct_loss_value: value[19] as f32,
            },
        })
    }
}

impl From<Config> for Vec<f64> {
    fn from(value: Config) -> Self {
        let exploration_boost_first = value.mcts.exploration_boost(TicTacToeStatus::First) as f64;
        let exploration_boost_second = value.mcts.exploration_boost(TicTacToeStatus::Second) as f64;
        vec![
            value.mcts.base_config.exploration_constant as f64,
            exploration_boost_first,
            exploration_boost_second,
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
            value.heuristic.control_local_steepness as f64,
            value.heuristic.control_global_steepness as f64,
            value.heuristic.meta_cell_big_threat as f64,
            value.heuristic.meta_cell_small_threat as f64,
            value.heuristic.threat_steepness as f64,
            value.heuristic.constraint_factor as f64,
            value.heuristic.free_choice_constraint_factor as f64,
            value.heuristic.direct_loss_value as f64,
        ]
    }
}

impl Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let values: Vec<f64> = self.clone().into();
        let names = Config::parameter_names();

        if names.len() != values.len() {
            return Err(serde::ser::Error::custom(
                "Mismatched config name/value length",
            ));
        }

        let map: BTreeMap<_, _> = names.into_iter().zip(values).collect();
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map: BTreeMap<String, f64> = BTreeMap::deserialize(deserializer)?;
        let names = Config::parameter_names();

        let values: Vec<f64> = names
            .iter()
            .map(|key| map.get(key).cloned().unwrap_or(0.0))
            .collect();

        Config::try_from(&values[..]).map_err(serde::de::Error::custom)
    }
}

impl Config {
    pub fn parameter_names() -> Vec<String> {
        vec![
            "exploration_constant".into(),
            "exploration_boost_first".into(),
            "exploration_boost_second".into(),
            "progressive_widening_constant".into(),
            "progressive_widening_exponent".into(),
            "early_cut_off_depth".into(),
            "progressive_widening_initial_threshold".into(),
            "progressive_widening_decay_rate".into(),
            "early_cut_off_lower_bound".into(),
            "early_cut_off_upper_bound".into(),
            "control_base_weight".into(),
            "control_progress_offset".into(),
            "control_local_steepness".into(),
            "control_global_steepness".into(),
            "meta_cell_big_threat".into(),
            "meta_cell_small_threat".into(),
            "threat_steepness".into(),
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
                    exploration_boost: [
                        (TicTacToeStatus::First, 0.5),
                        (TicTacToeStatus::Second, 0.5),
                    ]
                    .into(),
                    progressive_widening_constant: 1.0,
                    progressive_widening_exponent: 1.0 / 3.0,
                    early_cut_off_depth: 10,
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
                control_local_steepness: 0.05,
                control_global_steepness: 0.1,
                meta_cell_big_threat: 2.0,
                meta_cell_small_threat: 0.5,
                threat_steepness: 0.1,
                constraint_factor: 0.1,
                free_choice_constraint_factor: 0.1,
                direct_loss_value: 0.0,
            },
        }
    }

    pub fn upper_bounds() -> Self {
        Config {
            mcts: UltTTTMCTSConfig {
                base_config: BaseConfig {
                    exploration_constant: 2.0,
                    exploration_boost: [
                        (TicTacToeStatus::First, 2.5),
                        (TicTacToeStatus::Second, 2.5),
                    ]
                    .into(),
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
                control_local_steepness: 0.3,
                control_global_steepness: 0.6,
                meta_cell_big_threat: 4.0,
                meta_cell_small_threat: 1.5,
                threat_steepness: 1.0,
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
            .map(|((min, max), name)| match name.as_str() {
                "control_local_steepness" | "control_global_steepness" | "threat_steepness" => {
                    ParamDescriptor {
                        name: name.to_owned(),
                        bound: ParamBound::LogScale(min, max),
                    }
                }
                "direct_loss_value" => ParamDescriptor {
                    name: name.to_owned(),
                    bound: ParamBound::Static(0.0),
                },
                _ => ParamDescriptor {
                    name: name.to_owned(),
                    bound: ParamBound::MinMax(min, max),
                },
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_bounds() {
        let lower = Config::lower_bounds();
        let upper = Config::upper_bounds();
        assert!(
            lower.mcts.base_config.exploration_constant
                < upper.mcts.base_config.exploration_constant
        );
    }
}
