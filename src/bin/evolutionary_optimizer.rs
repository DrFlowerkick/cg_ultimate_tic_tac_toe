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
            prefix: "evolutionary_optimizer_05_log".into(),
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

    let filename = "evolutionary_optimizer_results_05.csv";
    let population_size = 50;
    let param_bounds = Config::param_bounds();
    let population_saver = Some(PopulationSaver {
        file_path: filename.into(),
        step_size: 5,
        precision: 3,
    });

    info!("Starting building initial population");
    // Load initial population from file
    let (initial_population, parameter_names) =
        load_population::<&str, DefaultTolerance>(filename, true)?;
    assert_eq!(parameter_names, Config::parameter_names());

    let population_generator = UltTTTObjectiveFunction {
        num_matches: 100,
        early_break_off: Some(EarlyBreakOff {
            num_check_matches: 10,
            score_threshold: 0.5,
        }),
        progress_step_size: 10,
        estimated_num_of_steps: 50 * 100, // 50 candidates and 100 matches
    };

    /*let initial_population = initial_population.reevaluate_population(
        &population_generator,
        &param_bounds,
        population_saver.clone(),
    )?;*/

    let initial_population = initial_population.resize_population(
        population_size,
        Some((&population_generator, &param_bounds)),
        population_saver.clone(),
    )?;
    save_population(&initial_population, &parameter_names, filename, 3)?;
    reset_progress_counter();

    info!("Starting UltTTT Evolutionary Optimize");

    let (initial_population, parameter_names) = load_population(filename, true)?;
    assert_eq!(parameter_names, Config::parameter_names());
    assert_eq!(initial_population.size(), population_size);

    let evolutionary_optimizer_configuration = EvolutionaryOptimizer::<
        ExponentialSchedule,
        SigmoidSchedule,
        LinearSchedule,
        DefaultTolerance,
    > {
        generations: 300,
        population_size,
        hard_mutation_rate: SigmoidSchedule {
            start: 0.3,
            end: 0.01,
            steepness: 8.0,
        },
        soft_mutation_relative_std_dev: LinearSchedule {
            start: 0.1,
            end: 0.005,
        },
        max_attempts: 5,
        selection_schedule: ExponentialSchedule {
            start: 0.6,
            end: 0.1,
            exponent: 2.0,
        },
        initial_population,
        population_saver,
    };

    let evolutionary_optimizer_evaluation = UltTTTObjectiveFunction {
        num_matches: 100,
        early_break_off: Some(EarlyBreakOff {
            num_check_matches: 10,
            score_threshold: 0.55,
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

    save_population(&population, &Config::parameter_names(), filename, 3)?;
    Ok(())
}
