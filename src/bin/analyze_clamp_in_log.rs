// very simple tool to search clamp entries in log

use chrono::NaiveDate;
use my_lib::my_optimizer::trace_analysis::{read_logs_from_dir, DefaultLogEntry, LogEntryParser};
use serde::Deserialize;
use std::collections::HashMap;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error occurred: {:?}", err);
    }
}

fn run() -> anyhow::Result<()> {
    let log_entries: Vec<DefaultLogEntry<ClampedLogEntry>> = read_logs_from_dir(
        "./optimization/evolutionary",
        "evolutionary_optimizer_log*",
        Some((
            NaiveDate::from_ymd_opt(2025, 5, 23).unwrap(),
            NaiveDate::from_ymd_opt(2025, 5, 24).unwrap(),
        )),
    )?;
    println!("Found {} log entries", log_entries.len());
    let clamp_stats = analyze_clamp_events(log_entries)?;

    for (name, stats) in clamp_stats {
        stats.print(&name);
    }
    Ok(())
}

// We need this, because old log files saved delta_clamp as a string, but now it is a float.

// log entry of clamp events in json format
#[derive(Debug, Deserialize)]
struct ClampedLogEntry {
    pub message: String,
    pub name: String,
    #[serde(deserialize_with = "de_string_to_f64")]
    pub delta_clamp: f64,
}

fn de_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let s = String::deserialize(deserializer)?;
    s.parse::<f64>().map_err(Error::custom)
}

#[derive(Default, Debug)]
struct ClampStats {
    pub min_count: usize,
    pub max_count: usize,
    pub min_deviation_sum: f64,
    pub max_deviation_sum: f64,
}

impl ClampStats {
    pub fn record(&mut self, value: f64) {
        if value < 0.0 {
            self.min_count += 1;
            self.min_deviation_sum += value;
        } else {
            self.max_count += 1;
            self.max_deviation_sum += value;
        }
    }

    pub fn print(&self, name: &str) {
        println!("\nParameter: {name}");
        if self.min_count > 0 {
            println!(
                "  Min clamps: {:3} (avg deviation: {:>+.5})",
                self.min_count,
                self.min_deviation_sum / self.min_count as f64
            );
        }
        if self.max_count > 0 {
            println!(
                "  Max clamps: {:3} (avg deviation: {:>+.5})",
                self.max_count,
                self.max_deviation_sum / self.max_count as f64
            );
        }
    }
}

// simple example tool to analyze clamp events from log entries.
// it ignores spans and only looks for messages containing "clamped".
fn analyze_clamp_events<T, S>(log_entries: Vec<T>) -> anyhow::Result<HashMap<String, ClampStats>>
where
    T: LogEntryParser<ClampedLogEntry, S>,
{
    let mut stats: HashMap<String, ClampStats> = HashMap::new();
    for entry in log_entries {
        if let Some(clamp_entry) = entry.get_fields() {
            if clamp_entry.message.contains("clamped") {
                stats
                    .entry(clamp_entry.name.to_owned())
                    .or_default()
                    .record(clamp_entry.delta_clamp);
            }
        }
    }
    Ok(stats)
}
