use my_lib::my_mcts::{
    CachedUTC, DynamicC, HeuristicCutoff, MCTSAlgo, MCTSGame, PlainMCTS, PlainTTHashMap,
};

use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use cg_ultimate_tic_tac_toe::{
    HPWDefaultTTTNoGameCache, UltTTT, UltTTTHeuristic, UltTTTHeuristicConfig, UltTTTMCTSConfig,
    UltTTTMCTSGame, UltTTTMove,
};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/

// Write an action using println!("message...");
// To debug: eprintln!("Debug message...");

fn main() {
    let mut start = Instant::now();
    let time_out_first_turn = Duration::from_millis(990);
    let time_out_successive_turns = Duration::from_millis(85);
    let time_out_codingame_input = Duration::from_millis(10_000);

    // start parallel thread for input of codingame
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            // get opponent's move
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(' ').collect::<Vec<_>>();
            let opponent_row = parse_input!(inputs[0], i32);
            let opponent_col = parse_input!(inputs[1], i32);
            if tx.send((opponent_row, opponent_col)).is_err() {
                break;
            }
            // read remaining input, which is not needed
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let valid_action_count = parse_input!(input_line, i32);
            for _i in 0..valid_action_count as usize {
                let mut input_line = String::new();
                io::stdin().read_line(&mut input_line).unwrap();
                let inputs = input_line.split(' ').collect::<Vec<_>>();
                let _row = parse_input!(inputs[0], i32);
                let _col = parse_input!(inputs[1], i32);
            }
        }
    });

    // prepare mcts
    let expected_num_nodes = 160_000;
    type UltTTTMCTS = PlainMCTS<
        UltTTTMCTSGame,
        UltTTTHeuristic,
        UltTTTMCTSConfig,
        CachedUTC,
        PlainTTHashMap<UltTTT>,
        DynamicC,
        HPWDefaultTTTNoGameCache,
        HeuristicCutoff,
    >;
    let mut game_data = UltTTT::new();
    let mut mcts_ult_ttt = UltTTTMCTS::new(
        UltTTTMCTSConfig::new_optimized(),
        UltTTTHeuristicConfig::new_optimized(),
        expected_num_nodes,
    );
    mcts_ult_ttt.set_root(&game_data);

    // init variables
    let mut turn_counter = 0;
    let mut first_turn = true;
    let mut time_out = time_out_first_turn;
    let mut instant_input_received = Instant::now();
    let mut input_received = false;
    let mut number_of_iterations = 0;
    eprintln!("Starting pre-filling tree");
    loop {
        match rx.try_recv() {
            Ok((opponent_row, opponent_col)) => {
                // codingame input received: opponent move or initial input
                let time_elapsed = start.elapsed();
                if first_turn {
                    assert_eq!(
                        turn_counter, 0,
                        "Should have reset first_turn flag before receiving second input."
                    );
                    eprintln!("time of initial input: {:?}", time_elapsed);
                    // check if opponent is starting_player
                    // me is start player, if (opponent_row, opponent_col) == (-1, -1)
                    if opponent_row >= 0 {
                        eprintln!("I'm second player.");
                        // opponent made a move, increment turn counter
                        turn_counter += 1;
                        // opponent is start player
                        let opp_action = (opponent_col as u8, opponent_row as u8);
                        // set game_data to secondary_game_data with applied opponent move
                        game_data = UltTTTMCTSGame::apply_move(
                            &game_data,
                            &UltTTTMove::try_from(opp_action).unwrap(),
                            &mut mcts_ult_ttt.game_cache,
                        );
                        if !mcts_ult_ttt.set_root(&game_data) {
                            eprintln!("Reset root of secondary_mcts_ult_ttt.");
                        }
                        // ToDo: set exploration boost for first and second
                    } else {
                        eprintln!("I'm first player.");
                        // ToDo: set exploration boost for second and first
                    }
                } else {
                    // successive turn
                    eprintln!("time from opp perspective: {:?}", time_elapsed);
                    turn_counter += 1;
                    let opp_action = (opponent_col as u8, opponent_row as u8);
                    game_data = UltTTTMCTSGame::apply_move(
                        &game_data,
                        &UltTTTMove::try_from(opp_action).unwrap(),
                        &mut mcts_ult_ttt.game_cache,
                    );
                    // set root to opponent move
                    if !mcts_ult_ttt.set_root(&game_data) {
                        eprintln!("Reset root after opponent move in turn {}.", turn_counter);
                    }
                }
                instant_input_received = Instant::now();
                input_received = true;
            }
            Err(mpsc::TryRecvError::Empty) => {
                // no new input received
                let time_elapsed_since_start = start.elapsed();
                if time_elapsed_since_start > time_out_codingame_input {
                    panic!("Timeout while waiting for codingame input");
                }
                // expand mcts tree until new input is received and
                // time_out after received input is reached
                mcts_ult_ttt.iterate();
                number_of_iterations += 1;
                if input_received && instant_input_received.elapsed() > time_out {
                    eprintln!(
                        "time from my perspective: {:?}",
                        instant_input_received.elapsed()
                    );
                    eprintln!("total time of iterations: {:?}", start.elapsed());
                    turn_counter += 1;
                    eprintln!(
                        "Iterations of turn {}: {}",
                        turn_counter, number_of_iterations
                    );
                    // select my move and send it to codingame
                    let selected_move = *mcts_ult_ttt.select_move();
                    game_data = UltTTTMCTSGame::apply_move(
                        &game_data,
                        &selected_move,
                        &mut mcts_ult_ttt.game_cache,
                    );
                    selected_move.execute_action();
                    // set root to my move; we expect to always find root, since we selected move from existing nodes
                    assert!(mcts_ult_ttt.set_root(&game_data));
                    eprintln!("Size of tree: {}", mcts_ult_ttt.tree.nodes.len());
                    // reset variables and timer
                    number_of_iterations = 0;
                    time_out = time_out_successive_turns;
                    first_turn = false;
                    input_received = false;
                    start = Instant::now();
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("Codingame input thread disconnected");
            }
        }
    }
}
