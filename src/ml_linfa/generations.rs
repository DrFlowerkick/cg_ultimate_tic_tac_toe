// definitions of generations

use super::UltMCTSTraining;
use crate::{UltTTTMCTSConfig, UltTTTMCTSGame};
use my_lib::my_mcts::{
    BaseConfig, CachedUTC, DefaultSimulationPolicy, DynamicC, NoHeuristic, ProgressiveWidening,
};
use std::collections::HashMap;

// Generation 0
pub type MCTSGenV00 = UltMCTSTraining<
    NoHeuristic,
    CachedUTC,
    DynamicC,
    ProgressiveWidening<UltTTTMCTSGame, UltTTTMCTSConfig>,
    DefaultSimulationPolicy,
>;

impl UltTTTMCTSConfig {
    pub fn config_gen_v00() -> Self {
        Self {
            base_config: BaseConfig {
                exploration_constant: 3.0,
                exploration_boost: HashMap::new(),
                progressive_widening_constant: 1.5,
                progressive_widening_exponent: 0.5,
                early_cut_off_depth: 0, // not used
            },
        }
    }
}
