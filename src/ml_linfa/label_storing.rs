// storing of labels

use super::{LabelSink, LabeledExample};
use anyhow::Result;
use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use crossbeam::channel;
use std::collections::HashMap;
use std::time::Duration;
use tokio::{select, sync::mpsc};
use tokio_postgres::NoTls;

#[derive(Clone)]
pub struct ChannelLabelSink {
    tx: channel::Sender<LabeledExample>,
}

impl ChannelLabelSink {
    pub fn new(tx: channel::Sender<LabeledExample>) -> Self {
        Self { tx }
    }
}

impl LabelSink for ChannelLabelSink {
    fn insert(&mut self, labeled_example: LabeledExample) -> anyhow::Result<()> {
        self.tx
            .send(labeled_example)
            .map_err(|e| anyhow::anyhow!("send failed: {:?}", e))
    }
}

#[derive(Clone)]
pub struct PgLabelWriter {
    pool: Pool<PostgresConnectionManager<NoTls>>,
    buffer: HashMap<(i64, i64), LabeledExample>,
    buffer_size: usize,
    flush_duration: Duration,
}

impl PgLabelWriter {
    // creates Pool + table (not existing)
    pub async fn connect(
        db_uri: &str,
        buffer_size: usize,
        flush_duration: Duration,
    ) -> Result<Self> {
        let mgr = PostgresConnectionManager::new_from_stringlike(db_uri, NoTls)?;
        let pool = Pool::builder().max_size(8).build(mgr).await?;

        {
            let conn = pool.get().await?;
            conn.batch_execute(
                r#"CREATE TABLE IF NOT EXISTS labels (
                       hash       BIGINT  NOT NULL,
                       generation BIGINT  NOT NULL,
                       visits     BIGINT  NOT NULL,
                       score      DOUBLE  PRECISION NOT NULL,
                       features   DOUBLE  PRECISION[] NOT NULL,
                       PRIMARY KEY (hash, generation)
                   );"#,
            )
            .await?;
        }

        Ok(Self {
            pool,
            buffer: HashMap::with_capacity(buffer_size),
            buffer_size,
            flush_duration,
        })
    }

    // bridge between sync and async
    pub async fn start_crossbeam_bridge(
        sync_rx: channel::Receiver<LabeledExample>,
        async_tx: mpsc::Sender<LabeledExample>,
    ) {
        while let Ok(label) = sync_rx.recv() {
            if let Err(e) = async_tx.send(label).await {
                tracing::error!("Failed to forward label to async DB writer: {:?}", e);
                break;
            }
        }
        tracing::info!("Crossbeam → Tokio bridge task finished.");
    }

    // start async consumer loop
    pub async fn run_db_writer(mut self, mut rx: mpsc::Receiver<LabeledExample>) {
        let mut flush_timer = tokio::time::interval(self.flush_duration);
        loop {
            select! {
                maybe_label = rx.recv() => {
                    match maybe_label {
                        Some(label) => {
                            let key = (label.hash, label.generation);
                            self.buffer
                                .entry(key)
                                .and_modify(|e| {
                                    e.visits += label.visits;
                                    e.score += label.score;
                                })
                                .or_insert(label);
                            if self.buffer.len() >= self.buffer_size {
                                if let Err(e) = self.flush_buffer().await {
                                    tracing::error!("Failed to flush buffer to db: {:?}", e);
                                }
                            }
                        },
                        None => {
                            // sender was closed → save last batch
                            if !self.buffer.is_empty() {
                                if let Err(e) = self.flush_buffer().await {
                                    tracing::error!("Failed to flush buffer to db: {:?}", e);
                                }
                            }
                            break;
                        }
                    }
                }

                _ = flush_timer.tick() => {
                    if !self.buffer.is_empty() {
                        if let Err(e) = self.flush_buffer().await {
                            tracing::error!("Failed to flush buffer to db: {:?}", e);
                        }
                    }
                }
            }
        }
        tracing::info!("db writer finished.");
    }

    // save buffer in db in one batch
    pub async fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let conn: PooledConnection<_> = self.pool.get().await?;

        let mut query =
            String::from("INSERT INTO labels (hash, generation, features, visits, score) VALUES ");
        let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            Vec::with_capacity(self.buffer.len() * 5);

        for (i, label) in self.buffer.values().enumerate() {
            if i > 0 {
                query.push_str(", ");
            }

            let param_index = i * 5;
            query.push_str(&format!(
                "(${}, ${}, ${}, ${}, ${})",
                param_index + 1,
                param_index + 2,
                param_index + 3,
                param_index + 4,
                param_index + 5
            ));

            params.push(&label.hash);
            params.push(&label.generation);
            params.push(&label.features);
            params.push(&label.visits);
            params.push(&label.score);
        }

        query.push_str(
            " ON CONFLICT (hash, generation) DO UPDATE \
            SET visits = labels.visits + EXCLUDED.visits, \
            score = labels.score + EXCLUDED.score",
        );

        conn.execute(query.as_str(), &params[..]).await?;
        self.buffer.clear();
        Ok(())
    }
}
