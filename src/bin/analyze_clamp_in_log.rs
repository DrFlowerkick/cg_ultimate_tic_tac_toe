// very simple tool to search clamp entries in log

use chrono::NaiveDate;
use my_lib::my_optimizer::*;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error occurred: {:?}", err);
    }
}

fn run() -> anyhow::Result<()> {
    analyze_clamps_from_dir(
        "./optimization/evolutionary",
        "evolutionary_optimizer_log*",
        Some((
            NaiveDate::from_ymd_opt(2025, 5, 23).unwrap(),
            NaiveDate::from_ymd_opt(2025, 5, 24).unwrap(),
        )),
    )
}
