// configuration of UltTTT for MCTS and heuristic

use my_lib::my_mcts::{BaseConfig, BaseHeuristicConfig, HeuristicConfig, MCTSConfig};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UltTTTMCTSConfig {
    pub base_config: BaseConfig,
}

// exploration_constant,progressive_widening_constant,progressive_widening_exponent,early_cut_off_depth,
// old: 1.259,1.371,0.343,18.840,
// new: 1.185,1.361,0.407,17.954,
// intermediate: 1.778,1.652,0.333,15.361

impl UltTTTMCTSConfig {
    pub fn optimized() -> Self {
        UltTTTMCTSConfig {
            base_config: BaseConfig {
                exploration_constant: 1.259,
                progressive_widening_constant: 1.371,
                progressive_widening_exponent: 0.343,
                early_cut_off_depth: 19,
            },
        }
    }
    pub fn new_optimized() -> Self {
        UltTTTMCTSConfig {
            base_config: BaseConfig {
                exploration_constant: 1.298,
                progressive_widening_constant: 1.602,
                progressive_widening_exponent: 0.333,
                early_cut_off_depth: 15,
            },
        }
    }
}

impl Default for UltTTTMCTSConfig {
    fn default() -> Self {
        UltTTTMCTSConfig {
            base_config: BaseConfig {
                exploration_constant: 1.4,
                progressive_widening_constant: 2.0,
                progressive_widening_exponent: 0.5,
                early_cut_off_depth: 30,
            },
        }
    }
}

impl MCTSConfig for UltTTTMCTSConfig {
    fn exploration_constant(&self) -> f32 {
        self.base_config.exploration_constant
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
                progressive_widening_decay_rate: 0.807,
                early_cut_off_lower_bound: 0.161,
                early_cut_off_upper_bound: 0.941,
            },
            control_base_weight: 0.573,
            control_progress_offset: 0.271,
            control_local_steepness: 0.15,
            control_global_steepness: 0.3,
            meta_cell_big_threat: 3.931,
            meta_cell_small_threat: 1.17,
            threat_steepness: 0.5,
            constraint_factor: 1.291,
            free_choice_constraint_factor: 1.344,
            direct_loss_value: 0.0,
        }
    }
    pub fn new_optimized() -> Self {
        UltTTTHeuristicConfig {
            base_config: BaseHeuristicConfig {
                progressive_widening_initial_threshold: 0.676,
                progressive_widening_decay_rate: 0.814,
                early_cut_off_lower_bound: 0.068,
                early_cut_off_upper_bound: 0.947,
            },
            control_base_weight: 0.538,
            control_progress_offset: 0.228,
            control_local_steepness: 0.099,
            control_global_steepness: 0.599,
            meta_cell_big_threat: 2.143,
            meta_cell_small_threat: 0.746,
            threat_steepness: 0.171,
            constraint_factor: 0.128,
            free_choice_constraint_factor: 0.982,
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
