// random search of optimal parameters

use cg_ultimate_tic_tac_toe::utilities::*;
use my_lib::my_optimizer::*;
use tracing::{info, span, Level};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error occurred: {:?}", err);
    }
}

fn run() -> anyhow::Result<()> {
    // enable tracing
    let _log_guard = TracingConfig {
        default_level: "debug",
        console_format: LogFormat::PlainText,
        file_log: Some(FileLogConfig {
            directory: ".",
            prefix: "random_search_log".into(),
            format: LogFormat::Json,
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

    let filename = "random_search_results.csv";

    let random_search_configuration = RandomSearch {
        iterations: 5_000,

        population_saver: Some(PopulationSaver {
            file_path: filename.into(),
            step_size: 10,
            precision: 3,
        }),
    };

    let random_search_evaluation = UltTTTObjectiveFunction {
        num_matches: 90,
        early_break_off: Some(EarlyBreakOff {
            num_initial_matches: 10,
            score_threshold: 0.5,
        }),
        progress_step_size: 10,
        estimated_num_of_steps: random_search_configuration
            .get_estimate_of_cycles(&param_bounds)?
            * 100, // 100 matches
    };

    let population_size = 20;

    let population = random_search_configuration.explore(
        &random_search_evaluation,
        &param_bounds,
        population_size,
    )?;
    let best_config: Config = population.best().expect("Empty population").params[..].try_into()?;

    info!(
        "Finished UltTTT Random Search with best candidate: {:?}",
        best_config
    );

    save_population(&population, &Config::parameter_names(), filename, 3)?;
    Ok(())
}
