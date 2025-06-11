/*use my_lib::my_mcts::{
    CachedUTC, DynamicC, HeuristicCutoff, MCTSAlgo, MCTSGame, PlainMCTS, PlainTTHashMap,
};*/
use my_lib::my_mcts::{
    CachedUTC, DefaultSimulationPolicy, ExpandAll, MCTSAlgo, MCTSGame, NoHeuristic,
    NoTranspositionTable, PlainMCTS, StaticC,
};
use my_lib::my_tic_tac_toe::TicTacToeStatus;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/*use cg_ultimate_tic_tac_toe::{
    HPWDefaultTTTNoGameCache, UltTTT, UltTTTHeuristic, UltTTTHeuristicConfig, UltTTTMCTSConfig,
    UltTTTMCTSGame, UltTTTMove,
};*/
use cg_ultimate_tic_tac_toe::{UltTTT, UltTTTMCTSConfig, UltTTTMCTSGame, UltTTTMove};

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
    let time_out_first_turn = Duration::from_millis(990);
    let time_out_successive_turns = Duration::from_millis(90);
    let time_out_codingame_input = Duration::from_millis(2000);
    let mut game_data = UltTTT::new();
    let mut mcts_ult_ttt: PlainMCTS<
        UltTTTMCTSGame,
        NoHeuristic,
        UltTTTMCTSConfig,
        CachedUTC,
        NoTranspositionTable,
        StaticC,
        ExpandAll,
        DefaultSimulationPolicy,
    > = PlainMCTS::new(UltTTTMCTSConfig::default(), NoHeuristic {});

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

    let mut turn_counter = 0;

    let (opponent_row, opponent_col) = rx.recv().expect("Failed to receive initial input");
    // check if opponent is starting_player
    if opponent_row >= 0 {
        turn_counter += 1;
        game_data.set_current_player(TicTacToeStatus::Opp);
        let opp_action = (opponent_col as u8, opponent_row as u8);
        game_data = UltTTTMCTSGame::apply_move(
            &game_data,
            &UltTTTMove::try_from(opp_action).unwrap(),
            &mut mcts_ult_ttt.game_cache,
        );
    }

    // First turn MCTS
    // set root to initial state or to opponent move
    mcts_ult_ttt.set_root(&game_data);
    let start = Instant::now();
    let mut number_of_iterations = 0;
    while start.elapsed() < time_out_first_turn {
        mcts_ult_ttt.iterate();
        number_of_iterations += 1;
    }
    eprintln!("Iterations of first turn: {}", number_of_iterations);

    // game loop
    loop {
        turn_counter += 1;
        // select my move and send it to codingame
        let selected_move = *mcts_ult_ttt.select_move();
        game_data =
            UltTTTMCTSGame::apply_move(&game_data, &selected_move, &mut mcts_ult_ttt.game_cache);
        selected_move.execute_action();
        // set root to my move; we expect to always find root, since we selected move from existing nodes
        assert!(mcts_ult_ttt.set_root(&game_data));
        // iterate until new input from codingame arrives
        let start = Instant::now();
        let mut instant_input_received = Instant::now();
        let mut input_received = false;
        number_of_iterations = 0;
        loop {
            match rx.try_recv() {
                Ok((opponent_row, opponent_col)) => {
                    eprintln!("time from opp perspective: {:?}", start.elapsed());
                    turn_counter += 1;
                    // codingame input received
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
                    instant_input_received = Instant::now();
                    input_received = true;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // expand mcts tree until new input is received
                    mcts_ult_ttt.iterate();
                    number_of_iterations += 1;
                    if start.elapsed() > time_out_codingame_input {
                        panic!("Timeout while waiting for codingame input");
                    }
                    // expand mcts tree for time_out_successive_turns after receiving input to make
                    // sure, that always some tree exists after action of opponent
                    if input_received
                        && instant_input_received.elapsed() > time_out_successive_turns
                    {
                        eprintln!(
                            "time from my perspective: {:?}",
                            instant_input_received.elapsed()
                        );
                        eprintln!("total time of iterations: {:?}", start.elapsed());
                        break;
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    panic!("Codingame input thread disconnected");
                }
            }
        }
        eprintln!("Iterations of successive turns: {}", number_of_iterations);
    }
}
