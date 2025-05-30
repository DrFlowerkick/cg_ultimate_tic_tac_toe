// configuration of UltTTT for MCTS and heuristic

use super::*;

#[derive(Debug, Clone, Copy)]
pub struct UltTTTMCTSConfig {
    pub base_config: BaseConfig,
}

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

#[derive(Debug, Clone, Copy)]
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
