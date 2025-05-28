// analyzing deviation of score with optimizer parameter values

use cg_ultimate_tic_tac_toe::utilities::*;
use my_lib::my_optimizer::*;
use statrs::statistics::Statistics;
use std::fs::File;
use std::io::{BufWriter, Write};
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
            prefix: "deviation_of_score_log".into(),
            format: LogFormat::Json,
        }),
    }
    .init();

    let span_search = span!(Level::INFO, "UltTTTDeviationOfScore");
    let _enter = span_search.enter();

    info!("Starting UltTTT analysis of deviation of score");
    // load random search results
    let (random_search_population, _) = load_population("random_search_results.csv", true)?;
    // load evolutionary optimizer results
    let (evolutionary_optimizer_population, _) =
        load_population("evolutionary_optimizer_results.csv", true)?;

    let mut merged_population = Population::new(50);
    merged_population.merge(evolutionary_optimizer_population);
    merged_population.merge(random_search_population);

    let num_test_runs = 10;
    let objective_function = UltTTTObjectiveFunction {
        num_matches: 100,
        early_break_off: None,
        progress_step_size: 10,
        estimated_num_of_steps: num_test_runs * merged_population.size() * 100, // 100 matches per candidate
    };

    let mut results: Vec<(Candidate, f64, f64)> = Vec::new();

    for candidate in merged_population.iter() {
        let config = Config::try_from(&candidate.params[..])?;
        let mut scores: Vec<f64> = Vec::with_capacity(num_test_runs + 1);
        scores.push(candidate.score);
        for _ in 0..num_test_runs {
            let score = objective_function.evaluate(config)?;
            scores.push(score);
        }
        let mean_score = (&scores).mean();
        let std_dev_score = scores.std_dev();

        info!(
            "Candidate: {:?}, Initial Score: {:.3}, Mean Score: {:.3}, Std Dev: {:.3}",
            config, candidate.score, mean_score, std_dev_score
        );
        results.push((candidate.clone(), mean_score, std_dev_score));
    }

    let file = File::create("deviation_of_score_results.csv")?;
    let mut writer = BufWriter::new(file);
    let param_names = Config::parameter_names().join(",");
    writeln!(
        writer,
        "{},average_score,mean_score,std_dev_score",
        param_names
    )?;
    for (candidate, mean_score, std_dev_score) in &results {
        writeln!(
            writer,
            "{},{:.3},{:.3}",
            candidate.to_csv(3),
            mean_score,
            std_dev_score
        )?;
    }

    info!("Finished UltTTT analysis of deviation of score");

    Ok(())
}
