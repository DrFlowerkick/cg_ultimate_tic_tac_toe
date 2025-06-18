// configuration of UltTTT for MCTS and heuristic

use std::collections::HashMap;

use my_lib::{
    my_mcts::{BaseConfig, BaseHeuristicConfig, HeuristicConfig, MCTSConfig},
    my_tic_tac_toe::TicTacToeStatus,
};

#[derive(Debug, Clone, PartialEq)]
pub struct UltTTTMCTSConfig {
    pub base_config: BaseConfig<TicTacToeStatus>,
}

// exploration_constant,progressive_widening_constant,progressive_widening_exponent,early_cut_off_depth,
// old: 1.259,1.371,0.343,18.840,
// new: 1.185,1.361,0.407,17.954,
// intermediate: 1.778,1.652,0.333,15.361

impl UltTTTMCTSConfig {
    pub fn optimized() -> Self {
        UltTTTMCTSConfig {
            base_config: BaseConfig {
                exploration_constant: 1.298,
                exploration_boost: [
                    (TicTacToeStatus::First, 1.0),
                    (TicTacToeStatus::Second, 1.0),
                ]
                .into(),
                progressive_widening_constant: 1.602,
                progressive_widening_exponent: 0.333,
                early_cut_off_depth: 15,
            },
        }
    }
    pub fn new_optimized() -> Self {
        UltTTTMCTSConfig {
            base_config: BaseConfig {
                exploration_constant: 1.992,
                exploration_boost: [
                    (TicTacToeStatus::First, 1.0),
                    (TicTacToeStatus::Second, 1.0),
                ]
                .into(),
                progressive_widening_constant: 1.584,
                progressive_widening_exponent: 0.333,
                early_cut_off_depth: 12,
            },
        }
    }
    pub fn optimized_v05() -> Self {
        UltTTTMCTSConfig {
            base_config: BaseConfig {
                exploration_constant: 1.112,
                exploration_boost: [
                    (TicTacToeStatus::First, 1.084),
                    (TicTacToeStatus::Second, 0.914),
                ]
                .into(),
                progressive_widening_constant: 1.432,
                progressive_widening_exponent: 0.333,
                early_cut_off_depth: 10,
            },
        }
    }
    pub fn optimized_v05_initial_phase() -> Self {
        let mut config = Self::optimized_v05();
        config.base_config.exploration_boost.clear();
        config
    }
    pub fn optimized_v05_set_exploration_boost(&mut self, my_playing_position: TicTacToeStatus) {
        let config = Self::optimized_v05();
        match my_playing_position {
            TicTacToeStatus::First => {
                self.base_config.exploration_boost = config.base_config.exploration_boost;
            }
            TicTacToeStatus::Second => {
                // exploration boost in config is defined as
                // me: First and opp: Second
                // If my playing position is second, I have to switch the exploration boost parameters
                let mut exploration_boost: HashMap<TicTacToeStatus, f32> = HashMap::new();
                exploration_boost.insert(
                    TicTacToeStatus::First,
                    config.exploration_boost(TicTacToeStatus::Second),
                );
                exploration_boost.insert(
                    TicTacToeStatus::Second,
                    config.exploration_boost(TicTacToeStatus::First),
                );
                self.base_config.exploration_boost = exploration_boost;
            }
            _ => panic!("My playing position must always be First or Second"),
        }
    }
}

impl Default for UltTTTMCTSConfig {
    fn default() -> Self {
        UltTTTMCTSConfig {
            base_config: BaseConfig {
                exploration_constant: 1.4,
                exploration_boost: [
                    (TicTacToeStatus::First, 1.0),
                    (TicTacToeStatus::Second, 1.0),
                ]
                .into(),
                progressive_widening_constant: 2.0,
                progressive_widening_exponent: 0.5,
                early_cut_off_depth: 30,
            },
        }
    }
}

impl MCTSConfig<TicTacToeStatus> for UltTTTMCTSConfig {
    fn exploration_constant(&self) -> f32 {
        self.base_config.exploration_constant
    }
    fn exploration_boost(&self, player: TicTacToeStatus) -> f32 {
        self.base_config.exploration_boost(player)
    }
    fn progressive_widening_constant(&self) -> f32 {
        self.base_config.progressive_widening_constant
    }
    fn progressive_widening_exponent(&self) -> f32 {
        self.base_config.progressive_widening_exponent
    }
    fn early_cut_off_depth(&self) -> usize {
        self.base_config.early_cut_off_depth
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UltTTTHeuristicConfig {
    pub base_config: BaseHeuristicConfig,
    pub control_base_weight: f32,
    pub control_progress_offset: f32,
    pub control_local_steepness: f32,
    pub control_global_steepness: f32,
    pub meta_cell_big_threat: f32,
    pub meta_cell_small_threat: f32,
    pub threat_steepness: f32,
    pub constraint_factor: f32,
    pub free_choice_constraint_factor: f32,
    pub direct_loss_value: f32,
}

// progressive_widening_initial_threshold,progressive_widening_decay_rate,early_cut_off_lower_bound,early_cut_off_upper_bound,control_base_weight,control_progress_offset,control_local_steepness,control_global_steepness,meta_cell_big_threat,meta_cell_small_threat,threat_steepness,constraint_factor,free_choice_constraint_factor,direct_loss_value,average_score
// old: 0.837,0.807,0.161,0.941,0.573,0.271,0.150,0.300,3.931,1.179,0.500,1.291,1.344,0.000
// old score: 0.835
// new: 0.806,0.739,0.069,0.982,0.612,0.290,0.150,0.300,3.882,1.126,0.500,1.298,1.334,0.000
// new score: 0.825
// intermediate: 0.632,0.837,0.051,0.980,0.566,0.354,0.052,0.324,2.380,0.989,0.103,0.247,0.938,0.000
// intermediate score: 0.845
impl UltTTTHeuristicConfig {
    pub fn optimized() -> Self {
        UltTTTHeuristicConfig {
            base_config: BaseHeuristicConfig {
                progressive_widening_initial_threshold: 0.837,
                progressive_widening_decay_rate: 0.844,
                early_cut_off_lower_bound: 0.025,
                early_cut_off_upper_bound: 0.942,
            },
            control_base_weight: 0.600,
            control_progress_offset: 0.365,
            control_local_steepness: 0.060,
            control_global_steepness: 0.505,
            meta_cell_big_threat: 3.132,
            meta_cell_small_threat: 1.106,
            threat_steepness: 0.721,
            constraint_factor: 1.390,
            free_choice_constraint_factor: 0.502,
            direct_loss_value: 0.0,
        }
    }
    pub fn new_optimized() -> Self {
        UltTTTHeuristicConfig {
            base_config: BaseHeuristicConfig {
                progressive_widening_initial_threshold: 0.861,
                progressive_widening_decay_rate: 0.837,
                early_cut_off_lower_bound: 0.078,
                early_cut_off_upper_bound: 0.947,
            },
            control_base_weight: 0.600,
            control_progress_offset: 0.231,
            control_local_steepness: 0.054,
            control_global_steepness: 0.413,
            meta_cell_big_threat: 3.415,
            meta_cell_small_threat: 0.689,
            threat_steepness: 0.116,
            constraint_factor: 0.100,
            free_choice_constraint_factor: 0.850,
            direct_loss_value: 0.0,
        }
    }
    pub fn optimized_v05() -> Self {
        UltTTTHeuristicConfig {
            base_config: BaseHeuristicConfig {
                progressive_widening_initial_threshold: 0.602,
                progressive_widening_decay_rate: 0.859,
                early_cut_off_lower_bound: 0.095,
                early_cut_off_upper_bound: 0.946,
            },
            control_base_weight: 0.523,
            control_progress_offset: 0.316,
            control_local_steepness: 0.088,
            control_global_steepness: 0.497,
            meta_cell_big_threat: 3.137,
            meta_cell_small_threat: 0.989,
            threat_steepness: 0.199,
            constraint_factor: 0.100,
            free_choice_constraint_factor: 0.836,
            direct_loss_value: 0.0,
        }
    }
}

impl HeuristicConfig for UltTTTHeuristicConfig {
    fn progressive_widening_initial_threshold(&self) -> f32 {
        self.base_config.progressive_widening_initial_threshold
    }
    fn progressive_widening_decay_rate(&self) -> f32 {
        self.base_config.progressive_widening_decay_rate
    }
    fn early_cut_off_lower_bound(&self) -> f32 {
        self.base_config.early_cut_off_lower_bound
    }
    fn early_cut_off_upper_bound(&self) -> f32 {
        self.base_config.early_cut_off_upper_bound
    }
}

impl Default for UltTTTHeuristicConfig {
    fn default() -> Self {
        UltTTTHeuristicConfig {
            base_config: BaseHeuristicConfig {
                progressive_widening_initial_threshold: 0.8,
                progressive_widening_decay_rate: 0.95,
                early_cut_off_lower_bound: 0.05,
                early_cut_off_upper_bound: 0.95,
            },
            control_base_weight: 0.3,
            control_progress_offset: 0.4,
            control_local_steepness: 0.15,
            control_global_steepness: 0.3,
            meta_cell_big_threat: 3.0,
            meta_cell_small_threat: 1.5,
            threat_steepness: 0.5,
            constraint_factor: 1.5,
            free_choice_constraint_factor: 1.5,
            direct_loss_value: 0.01,
        }
    }
}
