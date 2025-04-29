use my_lib::my_map_point::*;
use my_lib::my_monte_carlo_tree_search::*;
use std::io;
use std::time::{Duration, Instant};

use cg_ultimate_tic_tac_toe::{UltTTT, UltTTTMCTSGame, UltTTTPlayerAction, U, V};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

/**
 * Auto-generated code below aims at helping you parse
 * the standard input according to the problem statement.
 **/
fn main() {
    let max_number_of_turns = 81;
    let weighting_factor = 1.4;
    let time_out_first_turn = Duration::from_millis(995);
    let time_out_successive_turns = Duration::from_millis(95);
    let mut first_turn = true;
    let mut game_data = UltTTT::new();
    let mut mcts_ult_ttt: TurnBasedMCTS<UltTTTMCTSGame> = TurnBasedMCTS::new(weighting_factor);
    // game loop
    loop {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        let opponent_row = parse_input!(inputs[0], i32);
        let opponent_col = parse_input!(inputs[1], i32);
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

        let time_out = if first_turn {
            // check starting_player
            if opponent_row >= 0 {
                game_data.set_current_player(MonteCarloPlayer::Opp);
                let opp_action =
                    MapPoint::<U, V>::new(opponent_col as usize, opponent_row as usize);
                game_data = UltTTTMCTSGame::apply_move(
                    &game_data,
                    &UltTTTPlayerAction::from_ext(opp_action),
                );
            }
            time_out_first_turn
        } else {
            // update opp action
            let opp_action = MapPoint::<U, V>::new(opponent_col as usize, opponent_row as usize);
            game_data =
                UltTTTMCTSGame::apply_move(&game_data, &UltTTTPlayerAction::from_ext(opp_action));
            time_out_successive_turns
        };

        // Write an action using println!("message...");
        // To debug: eprintln!("Debug message...");

        let start = Instant::now();
        mcts_ult_ttt.set_root(&game_data);
        let mut number_of_iterations = 0;
        while start.elapsed() < time_out {
            mcts_ult_ttt.iterate();
            number_of_iterations += 1;
        }
        let selected_move = mcts_ult_ttt.select_move();
        game_data = UltTTTMCTSGame::apply_move(&game_data, selected_move);
        selected_move.execute_action();

        eprintln!("Iterations: {}", number_of_iterations);

        first_turn = false;
        assert!(game_data.game_turn <= max_number_of_turns);
    }
}
