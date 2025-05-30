// just a small helper tool

use my_lib::my_optimizer::*;
use cg_ultimate_tic_tac_toe::utilities::*;
use tracing::{info, span, Level};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error occurred: {:?}", err);
    }
}

fn run() -> anyhow::Result<()> {
    // enable tracing
    let _log_guard = TracingConfig::<&str> {
        default_level: "debug",
        console_format: LogFormat::PlainText,
        file_log: None,
    }
    .init();

    let span_search = span!(Level::INFO, "MergeEvoResults01_02");
    let _enter = span_search.enter();

    info!("Starting UltTTT merge of evolutionary optimizer results");
    let filename_01 = "./optimization/evolutionary/evolutionary_optimizer_results_gen01.csv";
    let filename_02 = "./optimization/evolutionary/evolutionary_optimizer_results_gen02.csv";

    let output_filename = "./optimization/evolutionary/merged_evolutionary_optimizer_results.csv";

    // load the first generation results
    let (population_01, _) = load_population(filename_01, true)?;
    let (population_02, _) = load_population(filename_02, true)?;

    let mut population = Population::new(20);

    let evaluation = UltTTTObjectiveFunction {
        num_matches: 100,
        early_break_off: None,
        progress_step_size: 10,
        estimated_num_of_steps: 20 // 20 candidates
            * 100, // 100 matches
    };

    for candidate in population_01.iter().chain(population_02.iter()).take(20) {
        let score = evaluation.evaluate((&candidate.params[..]).try_into()?)?;
        population.insert(Candidate { params: candidate.params.clone(), score });
    }

    // save the merged population
    save_population(&population, &Config::parameter_names(), output_filename, 3)?;

    Ok(())
}