// search for optimal parameters with evolutionary optimizer

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
        format: LogFormat::PlainText,
        file_log: Some(FileLogConfig {
            directory: ".",
            prefix: "evolutionary_optimizer_log".into(),
        }),
    }
    .init();

    let span_search = span!(Level::INFO, "UltTTTEvolutionaryOptimize");
    let _enter = span_search.enter();

    info!("Starting UltTTT Evolutionary Optimize");

    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    let param_bounds = Config::param_bounds();

    let filename = "random_search_results.csv";
    let Some((initial_population, param_names)) = load_population(filename, true) else {
        panic!("Failed to load population from '{}'.", filename);
    };
    assert_eq!(param_names, Config::parameter_names());
    let filename = "evolutionary_optimizer_results.csv";
    let population_size = initial_population.capacity();

    let evolutionary_optimizer_configuration = EvolutionaryOptimizer::<ExponentialSchedule> {
        generations: 100,
        population_size,
        mutation_rate: 0.2,
        hard_mutation_rate: 0.05,
        soft_mutation_std_dev: 0.05,
        selection_schedule: ExponentialSchedule {
            start: 0.5,
            end: 0.05,
            exponent: 2.0,
        },
        initial_population,
        population_saver: Some(PopulationSaver {
            file_path: filename.into(),
            step_size: 10,
            precision: 3,
        }),
    };

    let evolutionary_optimizer_evaluation = UltTTTObjectiveFunction {
        num_matches: 90,
        early_break_off: Some(EarlyBreakOff {
            num_initial_matches: 10,
            score_threshold: 0.4,
        }),
        progress_step_size: 10,
        estimated_num_of_steps: evolutionary_optimizer_configuration
            .get_estimate_of_cycles(&param_bounds)
            * 100, // 100 matches
    };

    let population = evolutionary_optimizer_configuration.optimize(
        &evolutionary_optimizer_evaluation,
        &param_bounds,
        population_size,
    )?;
    let best_config: Config = population.best().expect("Empty population").params[..].try_into()?;

    info!(
        "Finished UltTTT Evolutionary Optimize with best candidate: {:?}",
        best_config
    );

    save_population(
        &population,
        &Config::parameter_names(),
        "evolutionary_optimizer_results.csv",
        3,
    );
    Ok(())
}
