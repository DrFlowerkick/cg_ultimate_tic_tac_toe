use my_lib::my_monte_carlo_tree_search::*;
use my_lib::my_tic_tac_toe::TicTacToeStatus;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use cg_ultimate_tic_tac_toe::{
    HPWDefaultTTTNoGameCache, UltTTT, UltTTTHeuristic, UltTTTMCTSGame, UltTTTMCTSGameNoGameCache,
    UltTTTMove, UltTTTSimulationPolicy,
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
    let exploitation_constant = 1.4;
    // recursive heuristic is too expansive regarding calculation time
    let depth_first_turn = 0;
    let depth_successive_turns = 0;
    let alpha = 0.7;
    let time_out_first_turn = Duration::from_millis(990);
    let time_out_successive_turns = Duration::from_millis(90);
    let time_out_codingame_input = Duration::from_millis(2000);
    let mut game_data = UltTTT::new();
    let mut mcts_ult_ttt: PlainMCTS<
        UltTTTMCTSGameNoGameCache,
        DynamicC,
        CachedUTC,
        HPWDefaultTTTNoGameCache,
        UltTTTHeuristic,
        UltTTTSimulationPolicy,
    > = PlainMCTS::new(exploitation_constant, depth_first_turn, alpha);

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

    let (opponent_row, opponent_col) = rx.recv().expect("Failed to receive initial input");
    // check if opponent is starting_player
    if opponent_row >= 0 {
        game_data.set_current_player(TicTacToeStatus::Opp);
        let opp_action = (opponent_col as u8, opponent_row as u8);
        game_data = UltTTTMCTSGame::apply_move(
            &game_data,
            &UltTTTMove::try_from(opp_action).unwrap(),
            &mut mcts_ult_ttt.game_cache,
        );
    }

    // time out for first turn
    let mut time_out = time_out_first_turn;

    // game loop
    loop {
        // set root to either initial game data or to last opponent move
        mcts_ult_ttt.set_root(&game_data);
        // start MCTS iterations
        let start = Instant::now();
        let mut number_of_iterations = 0;
        while start.elapsed() < time_out {
            mcts_ult_ttt.iterate();
            number_of_iterations += 1;
        }
        eprintln!("Iterations: {}", number_of_iterations);
        // set timeout for all following turns
        time_out = time_out_successive_turns;
        mcts_ult_ttt.set_depth(depth_successive_turns);

        // select my move and send it to codingame
        let selected_move = *mcts_ult_ttt.select_move();
        game_data =
            UltTTTMCTSGame::apply_move(&game_data, &selected_move, &mut mcts_ult_ttt.game_cache);
        selected_move.execute_action();
        // set root to my move
        mcts_ult_ttt.set_root(&game_data);

        // use Pre-Filling until new input from codingame arrives
        let start = Instant::now();
        number_of_iterations = 0;
        loop {
            match rx.try_recv() {
                Ok((opponent_row, opponent_col)) => {
                    // codingame input received
                    let opp_action = (opponent_col as u8, opponent_row as u8);
                    game_data = UltTTTMCTSGame::apply_move(
                        &game_data,
                        &UltTTTMove::try_from(opp_action).unwrap(),
                        &mut mcts_ult_ttt.game_cache,
                    );
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // no codingame input yet
                    mcts_ult_ttt.iterate();
                    number_of_iterations += 1;
                    if start.elapsed() > time_out_codingame_input {
                        panic!("Timeout while waiting for codingame input");
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    panic!("Codingame input thread disconnected");
                }
            }
        }
        eprintln!("Pre-Fill Iterations: {}", number_of_iterations);
    }
}
