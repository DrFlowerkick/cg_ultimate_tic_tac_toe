// MCTS may benefit from state caching (transposition table) to avoid recalculating the same state multiple times.
// With this tool we analyze the final game tree of a match of UltTTT for the number of equal states, which could have been cached.

use cg_ultimate_tic_tac_toe::{config::*, utilities::*};
use my_lib::my_monte_carlo_tree_search::*;
use std::collections::{HashMap, HashSet};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error occurred: {:?}", err);
    }
}

fn run() -> anyhow::Result<()> {
    let config = Config {
        mcts: UltTTTMCTSConfig::new_optimized(),
        heuristic: UltTTTHeuristicConfig::new_optimized(),
    };

    println!("Running match...");
    let (_, first, second) = run_match(config, true);

    println!("Collecting nodes of same tree level of first...");
    let mut nodes_of_same_tree_level: HashMap<usize, Vec<usize>> = HashMap::new();
    collect_nodes_of_same_tree_level_first(&first, 0, 0, &mut nodes_of_same_tree_level);

    println!("Comparing node states of same tree level...");
    let mut cross_check = 0;
    let mut number_of_duplicates_of_each_level: HashMap<usize, usize> = HashMap::new();
    for (level, nodes) in nodes_of_same_tree_level.iter() {
        let mut unique_nodes: HashSet<_> = HashSet::new();
        let mut duplicate_nodes: HashSet<_> = HashSet::new();
        for &node_index in nodes {
            let state = first.tree.get_node(node_index).get_state();
            if !unique_nodes.insert(state) {
                duplicate_nodes.insert(state);
            }
        }
        if !duplicate_nodes.is_empty() {
            number_of_duplicates_of_each_level.insert(*level, duplicate_nodes.len());
            cross_check += duplicate_nodes.len();
        }
    }
    for i in 0..nodes_of_same_tree_level.len() {
        if let Some(num_duplicates) = number_of_duplicates_of_each_level.get(&i) {
            println!("Level {}: {} duplicate states", i, num_duplicates);
        }
    }

    println!("Collecting unique states of first player...");
    let unique_states: HashSet<_> = first.tree.nodes.iter().map(|node| node.state).collect();
    println!(
        "First player: {} unique states in tree with tree size of {}",
        unique_states.len(),
        first.tree.nodes.len()
    );
    println!(
        "Cross-check of duplicates: {}, Delta num nodes and unique states: {}",
        cross_check,
        first.tree.nodes.len() - unique_states.len()
    );

    println!("Collecting nodes of same tree level of second...");
    nodes_of_same_tree_level.clear();
    collect_nodes_of_same_tree_level_second(&second, 0, 0, &mut nodes_of_same_tree_level);

    println!("Comparing node states of same tree level...");
    let mut number_of_duplicates_of_each_level: HashMap<usize, usize> = HashMap::new();
    for (level, nodes) in nodes_of_same_tree_level.iter() {
        let mut unique_nodes: HashSet<_> = HashSet::new();
        let mut duplicate_nodes: HashSet<_> = HashSet::new();
        for &node_index in nodes {
            let state = first.tree.get_node(node_index).get_state();
            if !unique_nodes.insert(state) {
                duplicate_nodes.insert(state);
            }
        }
        if !duplicate_nodes.is_empty() {
            number_of_duplicates_of_each_level.insert(*level, duplicate_nodes.len());
        }
    }
    for i in 0..nodes_of_same_tree_level.len() {
        if let Some(num_duplicates) = number_of_duplicates_of_each_level.get(&i) {
            println!("Level {}: {} duplicate states", i, num_duplicates);
        }
    }

    println!("Collecting unique states of second player...");
    let unique_states: HashSet<_> = second.tree.nodes.iter().map(|node| node.state).collect();
    println!(
        "Second player: {} unique states in tree with tree size of {}",
        unique_states.len(),
        second.tree.nodes.len()
    );

    Ok(())
}

fn collect_nodes_of_same_tree_level_first(
    mcts: &UltTTTMCTSFirst,
    index: usize,
    level: usize,
    nodes_of_same_tree_level: &mut HashMap<usize, Vec<usize>>,
) {
    if let Some(states) = nodes_of_same_tree_level.get_mut(&level) {
        states.push(index);
    } else {
        nodes_of_same_tree_level.insert(level, vec![index]);
    }
    for child in mcts.tree.nodes[index].get_children() {
        collect_nodes_of_same_tree_level_first(mcts, *child, level + 1, nodes_of_same_tree_level);
    }
}

fn collect_nodes_of_same_tree_level_second(
    mcts: &UltTTTMCTSSecond,
    index: usize,
    level: usize,
    nodes_of_same_tree_level: &mut HashMap<usize, Vec<usize>>,
) {
    if let Some(states) = nodes_of_same_tree_level.get_mut(&level) {
        states.push(index);
    } else {
        nodes_of_same_tree_level.insert(level, vec![index]);
    }
    for child in mcts.tree.nodes[index].get_children() {
        collect_nodes_of_same_tree_level_second(mcts, *child, level + 1, nodes_of_same_tree_level);
    }
}
