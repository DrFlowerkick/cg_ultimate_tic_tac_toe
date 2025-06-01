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
        console_format: LogFormat::PlainText,
        file_log: Some(FileLogConfig {
            directory: ".",
            prefix: "evolutionary_optimizer_log".into(),
            format: LogFormat::Json,
        }),
    }
    .init();

    let span_search = span!(Level::INFO, "UltTTTEvolutionaryOptimize");
    let _enter = span_search.enter();

    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    info!("Starting UltTTT Evolutionary Optimize");

    let filename = "evolutionary_optimizer_results.csv";
    let population_size = 20;

    let (initial_population, parameter_names) = load_population(filename, true)?;
    assert_eq!(parameter_names, Config::parameter_names());
    assert_eq!(initial_population.size(), population_size);

    let param_bounds = Config::param_bounds();

    let evolutionary_optimizer_configuration = EvolutionaryOptimizer::<
        ConstantSchedule,
        ConstantSchedule,
        LinearSchedule,
        DefaultTolerance,
    > {
        generations: 100,
        population_size,
        hard_mutation_rate: ConstantSchedule {
            value: 0.05,    // 5% of parameters are mutated in each offspring
        },
        soft_mutation_relative_std_dev: LinearSchedule {
            start: 0.1,     // start with 10% of value range standard deviation
            end: 0.01,      // end with 1% of value range standard deviation
        },
        max_attempts: 5,
        selection_schedule: ConstantSchedule {
            value: 0.25,    // 25% of the population is selected for crossover
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
            score_threshold: 0.7,
        }),
        progress_step_size: 10,
        estimated_num_of_steps: evolutionary_optimizer_configuration
            .get_estimate_of_cycles(&param_bounds)?
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
    )?;
    Ok(())
}
