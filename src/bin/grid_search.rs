// coarse grid search to optimize parameters of UltTTT

use cg_ultimate_tic_tac_toe::utilities::*;
use my_lib::my_optimizer::*;
use tracing::{info, span, Level};

struct GridSearchConfigHandler {}

impl ConfigHandler for GridSearchConfigHandler {
    fn params_to_config(params: &[f64]) -> Config {
        let mut cgs_config = Config::from(params);
        cgs_config.heuristic.free_choice_constraint_factor = cgs_config.heuristic.constraint_factor;
        cgs_config
    }
}

fn main() {
    // enable tracing
    let _log_guard = TracingConfig {
        default_level: "debug",
        format: LogFormat::PlainText,
        file_log: Some(FileLogConfig {
            directory: ".",
            prefix: "grid_search_log".into(),
        }),
    }
    .init();

    let span_search = span!(Level::INFO, "UltTTTGridSearch");
    let _enter = span_search.enter();

    info!("Starting UltTTT Grid Search");

    let default_config = Config::default();
    let param_bounds = vec![
        ParamBound::Static(default_config.mcts.base_config.exploration_constant as f64),
        ParamBound::Static(
            default_config
                .mcts
                .base_config
                .progressive_widening_constant as f64,
        ),
        ParamBound::Static(
            default_config
                .mcts
                .base_config
                .progressive_widening_exponent as f64,
        ),
        ParamBound::Static(default_config.mcts.base_config.early_cut_off_depth as f64),
        ParamBound::Static(
            default_config
                .heuristic
                .base_config
                .progressive_widening_initial_threshold as f64,
        ),
        ParamBound::Static(
            default_config
                .heuristic
                .base_config
                .progressive_widening_decay_rate as f64,
        ),
        ParamBound::Static(
            default_config
                .heuristic
                .base_config
                .early_cut_off_lower_bound as f64,
        ),
        ParamBound::Static(
            default_config
                .heuristic
                .base_config
                .early_cut_off_upper_bound as f64,
        ),
        ParamBound::List([0.2, 0.3, 0.5].into()), // meta_weight_base
        ParamBound::Static(default_config.heuristic.meta_weight_progress_offset as f64),
        ParamBound::List([2.0, 3.0, 4.0].into()), // meta_cell_big_threat
        ParamBound::List([0.5, 1.0, 1.5].into()), // meta_cell_small_threat
        ParamBound::List([1.0, 1.5, 2.0].into()), // constraint_factor
        ParamBound::Static(default_config.heuristic.free_choice_constraint_factor as f64),
        ParamBound::List([0.0, 0.005, 0.01, 0.025].into()), // direct_loss_value
    ];

    let grid_configuration = GridSearch {
        steps_per_param: 0, // only list ParamBound
        channel_capacity: 1_000,
        worker_threads: 4,
    };

    let grid_evaluation = UltTTTObjectiveFunction::<GridSearchConfigHandler> {
        num_matches: 90,
        early_break_off: Some(EarlyBreakOff {
            num_initial_matches: 10,
            score_threshold: 0.4,
        }),
        progress_step_size: 10,
        estimated_num_of_steps: grid_configuration.get_estimate_of_cycles(&param_bounds) * 100, // 100 matches per candidate
        phantom: std::marker::PhantomData,
    };

    let population_size = 20;

    let population = grid_configuration.explore(&grid_evaluation, &param_bounds, population_size);
    let best_config: Config = population.best().expect("Empty population").params[..].into();

    info!(
        "Finished UltTTT Grid Search with best candidate: {:?}",
        best_config
    );

    save_population(
        &population,
        &Config::parameter_names(),
        "grid_search_results.csv",
    );
}
