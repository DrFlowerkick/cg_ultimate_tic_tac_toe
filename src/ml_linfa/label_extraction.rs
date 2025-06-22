// extract labels from MCTS tree

use super::{hash_features, FeatureExtraction};
use anyhow::{Context, Result};
use my_lib::my_mcts::{
    DfsWalker, Heuristic, MCTSAlgo, MCTSGame, MCTSNode, MCTSTree, NoTranspositionTable,
};
use serde::Serialize;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Serialize, Clone)]
pub struct LabeledExample {
    pub hash: i64,
    pub generation: i64,
    pub visits: i64,
    pub score: f64,
    pub features: Vec<f64>,
}

// trait for Label-Sink
pub trait LabelSink: Clone + Send + Sync {
    fn insert(&mut self, labeled_example: LabeledExample) -> Result<()>;
}

// collect labels of all nodes of MCTSTree
// defined generic over all MCTS trait configurations
pub fn collect_labels<G, H, A, T, S>(
    tree: &T,
    start: A::NodeID,
    generation: u32,
    min_visits: usize,
    is_final_tree: bool,
    mut sink: S,
) -> Result<()>
where
    G: MCTSGame,
    G::State: FeatureExtraction,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H, Tree = T, TranspositionTable = NoTranspositionTable>,
    A::NodeID: Hash,
    T: MCTSTree<G, H, A>,
    S: LabelSink,
{
    let walker = if let Some(root_id) = tree.root_id() {
        let skip = if is_final_tree { vec![] } else { vec![root_id] };
        DfsWalker::<G, H, A>::new(start, skip)
    } else {
        // uninitialized tree
        return Ok(());
    };

    let error_counter = AtomicUsize::new(0);
    for node_id in walker.into_iter(tree) {
        let node = tree.get_node(node_id);
        let visits = node.get_visits();
        if visits >= min_visits {
            let features = node.get_state().extract_features();
            let hash = hash_features(&features);
            let generation =
                i64::try_from(generation).context("generation value too large for i64")?;
            let visits = i64::try_from(visits).context("visits too large")?;
            let labeled_example = LabeledExample {
                hash,
                generation: generation as i64,
                visits: visits as i64,
                score: node.get_accumulated_value() as f64,
                features,
            };
            if let Err(e) = sink.insert(labeled_example) {
                let count = error_counter.fetch_add(1, Ordering::Relaxed);
                if count % 100 == 0 {
                    tracing::warn!("Label send failed ({} total so far): {e}", count + 1);
                }
            }
        }
    }

    let failed = error_counter.load(Ordering::Relaxed);
    if failed > 0 {
        tracing::warn!("Labeling finished with {failed} failed insertions.");
    } else {
        tracing::debug!("Labeling finished without any insert errors.");
    }
    Ok(())
}
