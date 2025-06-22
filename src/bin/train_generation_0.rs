// train generation 0

use std::time::Duration;

use anyhow::Result;
use cg_ultimate_tic_tac_toe::{
    ml_linfa::{run_training, ChannelLabelSink, MCTSGenV00, PgLabelWriter},
    UltTTTMCTSConfig,
};
use crossbeam::channel;
use my_lib::my_mcts::{NoHeuristic, PlainMCTS};
use my_lib::my_optimizer::{FileLogConfig, LogFormat, TracingConfig};
use tokio::sync::mpsc;
use tracing::{info, span, Level};

#[tokio::main]
async fn main() -> Result<()> {
    // enable tracing
    let _log_guard = TracingConfig {
        default_level: "debug",
        console_format: LogFormat::PlainText,
        file_log: Some(FileLogConfig {
            directory: ".",
            prefix: "train_generation_00_log".into(),
            format: LogFormat::Json,
        }),
    }
    .init();

    let span_search = span!(Level::INFO, "UltTTTEvolutionaryOptimize");
    let _enter = span_search.enter();

    info!("Starting training MCTS gen 0");

    let uri = "host=192.168.178.3 port=5434 user=postgres password=password dbname=ultttt";
    let buffer_size = 1_000;
    let flush_duration = Duration::from_millis(2_000);
    let writer = PgLabelWriter::connect(uri, buffer_size, flush_duration).await?;
    let (sync_tx, sync_rx) = channel::unbounded();
    let (async_tx, async_rx) = mpsc::channel(8192);

    // starting Bridge crossbeam → tokio
    let bridge_handle = tokio::spawn(PgLabelWriter::start_crossbeam_bridge(sync_rx, async_tx));

    // Spawn consumer loop of writer
    let writer_handle = tokio::spawn(writer.run_db_writer(async_rx));

    // prepare MCTS for training
    let mcts_config = UltTTTMCTSConfig::config_gen_v00();
    let expected_num_nodes = 500_000;
    let mcts: MCTSGenV00 = PlainMCTS::new(mcts_config, NoHeuristic {}, expected_num_nodes);

    // prepare training parameters
    let turn_duration = Duration::from_millis(5_000);
    let num_matches = 8;
    let generation = 0;
    let min_visits = 20;
    let label_sink = ChannelLabelSink::new(sync_tx.clone());
    run_training(
        mcts.clone(),
        mcts,
        turn_duration,
        generation,
        min_visits,
        label_sink,
        num_matches,
    )?;

    // dropping tx stops consumer loop
    drop(sync_tx);

    bridge_handle.await?;
    writer_handle.await?;

    info!("Training MCTS gen 0 finished");

    Ok(())
}
