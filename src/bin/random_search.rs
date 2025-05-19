// random search of optimal parameters

use cg_ultimate_tic_tac_toe::utilities::*;
use my_lib::my_optimizer::*;
use tracing::{info, span, Level};

fn main() {
    // enable tracing
    let _log_guard = TracingConfig {
        default_level: "debug",
        format: LogFormat::PlainText,
        file_log: Some(FileLogConfig {
            directory: ".",
            prefix: "random_search_log".into(),
        }),
    }
    .init();

    let span_search = span!(Level::INFO, "UltTTTRandomSearch");
    let _enter = span_search.enter();

    info!("Starting UltTTT Random Search");

    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    let param_bounds = Config::param_bounds();

    let random_search_configuration = RandomSearch { iterations: 5_000 };

    let random_search_evaluation = UltTTTObjectiveFunction::<DefaultConfigHandler> {
        num_matches: 90,
        early_break_off: Some(EarlyBreakOff {
            num_initial_matches: 10,
            score_threshold: 0.4,
        }),
        progress_step_size: 10,
        estimated_num_of_steps: random_search_configuration.get_estimate_of_cycles(&param_bounds) * 100, // 100 matches
        phantom: std::marker::PhantomData,
    };

    let population_size = 20;

    let population = random_search_configuration.explore(
        &random_search_evaluation,
        &param_bounds,
        population_size,
    );
    let best_config: Config = population.best().expect("Empty population").params[..].into();
    
    info!(
        "Finished UltTTT Random Search with best candidate: {:?}",
        best_config
    );

    save_population(
        &population,
        &Config::parameter_names(),
        "random_search_results.csv",
    );
}
