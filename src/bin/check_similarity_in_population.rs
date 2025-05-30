// just a small helper tool

use my_lib::my_optimizer::*;
use cg_ultimate_tic_tac_toe::utilities::*;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error occurred: {:?}", err);
    }
}

fn run() -> anyhow::Result<()> {
    let filename = "./optimization/evolutionary/merged_evolutionary_optimizer_results.csv";
    let (population, header) = load_population(filename, true)?;
    assert_eq!(header, Config::parameter_names());
    let top_scores = population.top_n(5).map(|c: &Candidate<DefaultTolerance>| c.score).collect::<Vec<_>>();
    println!("Top 5 scores: {:?}", top_scores);
    let tolerance = 0.01;
    println!("length of population: {}", population.size());
    for (i, candidate) in population.iter().enumerate() {
        let mut is_unique = true;
        for other in population.iter().skip(i + 1) {
            if candidate.is_similar_params(&other.params, tolerance) {
                is_unique = false;
                println!(
                    "Candidate {} and {} have identical parameters: {:?}",
                    i + 1,
                    i + 2 + (i + 1),
                    candidate.params
                );
            }
        }
        if is_unique {
            println!("Candidate {} is unique: {:?}", i + 1, candidate.params);
        }
    }

    Ok(())
}