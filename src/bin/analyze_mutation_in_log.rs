// util to analyze mutation events in the log files

use cg_ultimate_tic_tac_toe::{utilities::*, config::*};
use chrono::NaiveDate;
use my_lib::my_optimizer::{
    read_logs_from_dir, DefaultLogEntry, FileLogConfig, LogFormat,
    ObjectiveFunction, TracingConfig, EvoFields, EvoSpan, MutationStats, analyze_evo_log_entries,
};
use tracing::{info, span, Level};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error occurred: {:?}", err);
    }
}

fn run() -> anyhow::Result<()> {
    let log_entries: Vec<DefaultLogEntry<EvoFields, EvoSpan>> = read_logs_from_dir(
        "./optimization/evolutionary",
        "evolutionary_optimizer_log*",
        Some((
            NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 6, 2).unwrap(),
        )),
    )?;
    println!("Found {} log entries", log_entries.len());

    let mutation_stats: MutationStats<UltTTTObjectiveFunction> = analyze_evo_log_entries(log_entries)?;

    let mut count_best_parent = 0;
    let mut best_parent_config: Option<Config> = None;
    for (key, stats) in mutation_stats.iter() {
        if stats.parent_score <= stats.offspring_score {
            println!(
                "Generation: {}, ID: {}, Parent Score: {:.2}, Offspring Score: {:.2}",
                key.generation, key.id, stats.parent_score, stats.offspring_score
            );
        }
        if stats.parent_score > 0.82 {
            count_best_parent += 1;
            best_parent_config = Some(stats.parent_config);
            println!(
                "Best Parent Found - Generation: {}, ID: {}, Parent Score: {:.2}, Offspring Score: {:.2}",
                key.generation, key.id, stats.parent_score, stats.offspring_score
            );
        }
    }

    println!("Total best parents found: {}", count_best_parent);

    // enable tracing
    let _log_guard = TracingConfig {
        default_level: "debug",
        console_format: LogFormat::PlainText,
        file_log: Some(FileLogConfig {
            directory: ".",
            prefix: "ult_ttt_evaluate_best_parent_log".into(),
            format: LogFormat::Json,
        }),
    }
    .init();

    let span_search = span!(Level::INFO, "UltTTTEvaluateBestParent");
    let _enter = span_search.enter();

    info!("Starting UltTTT evaluation of best parent");

    let ult_ttt_objective_function = UltTTTObjectiveFunction {
        num_matches: 100,
        early_break_off: None,
        progress_step_size: 10,
        estimated_num_of_steps: 100,
    };

    let Some(config) = best_parent_config else {
        return Err(anyhow::anyhow!("No best parent configuration found"));
    };
    println!("best parent config: {:?}", config);

    let correct_config = Config {
        mcts: UltTTTMCTSConfig::new_optimized(),
        heuristic: UltTTTHeuristicConfig::new_optimized(),
    };

    println!(
        "Correct Configuration for Evaluation: {:?}",
        correct_config
    );

    let best_parent_result = ult_ttt_objective_function.evaluate(correct_config)?;

    info!(
        "Best Parent Configuration: {:?}, Score: {:.3}",
        config, best_parent_result
    );

    Ok(())
}
