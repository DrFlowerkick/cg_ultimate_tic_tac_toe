// coarse grid search to optimize parameters of UltTTT

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
            prefix: "grid_search_log".into(),
            format: LogFormat::Json,
        }),
    }
    .init();

    let span_search = span!(Level::INFO, "UltTTTGridSearch");
    let _enter = span_search.enter();

    info!("Starting UltTTT Grid Search");

    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();

    let mut param_bounds = Config::param_bounds();
    let grid_list_config = vec![
        ("meta_weight_base", vec![0.2, 0.3, 0.5]),
        ("meta_cell_big_threat", vec![2.0, 3.0, 4.0]),
        ("meta_cell_small_threat", vec![0.5, 1.0, 1.5]),
        ("constraint_factor", vec![1.0, 1.5, 2.0]),
        ("direct_loss_value", vec![0.0, 0.005, 0.01, 0.025]),
    ];
    for (name, list) in grid_list_config {
        let name_index = param_bounds
            .iter()
            .position(|pb| pb.name == name)
            .expect("Unknown parameter name.");
        param_bounds.get_mut(name_index).unwrap().bound = ParamBound::List(list);
    }

    let filename = "grid_search_results.csv";

    let grid_configuration = GridSearch {
        steps_per_param: 0, // only list ParamBound
        chunk_size: 100,
        population_saver: Some(PopulationSaver {
            file_path: filename.into(),
            step_size: 10,
            precision: 3,
        }),
    };

    let grid_evaluation = UltTTTObjectiveFunction {
        num_matches: 90,
        early_break_off: Some(EarlyBreakOff {
            num_initial_matches: 10,
            score_threshold: 0.4,
        }),
        progress_step_size: 10,
        estimated_num_of_steps: grid_configuration.get_estimate_of_cycles(&param_bounds)? * 100, // 100 matches per candidate
    };

    let population_size = 20;

    let population =
        grid_configuration.explore(&grid_evaluation, &param_bounds, population_size)?;
    let best_config: Config = population.best().expect("Empty population").params[..].try_into()?;

    info!(
        "Finished UltTTT Grid Search with best candidate: {:?}",
        best_config
    );

    save_population(&population, &Config::parameter_names(), filename, 3)?;
    Ok(())
}
