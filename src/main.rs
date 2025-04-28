use my_lib::my_map_point::*;
use my_lib::my_monte_carlo_tree_search::*;
use std::io;
use std::time::Duration;

use cg_ultimate_tic_tac_toe::{UltTTT, UltTTTGameDataUpdate, UltTTTPlayerAction, U, V};

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
    let mut turn_counter: usize = 1;
    let mut starting_player = MonteCarloPlayer::Me;
    let mut game_data = UltTTT::new();
    let game_mode = MonteCarloGameMode::ByTurns;
    let max_number_of_turns = 81;
    let force_update = true;
    let time_out_first_turn = Duration::from_millis(995);
    let time_out_successive_turns = Duration::from_millis(95);
    let weighting_factor = 1.4;
    let use_heuristic_score = false;
    let use_caching = false;
    let debug = true;
    let mut mcts: MonteCarloTreeSearch<UltTTT, UltTTTPlayerAction, UltTTTGameDataUpdate> =
        MonteCarloTreeSearch::new(
            game_mode,
            max_number_of_turns,
            force_update,
            time_out_first_turn,
            time_out_successive_turns,
            weighting_factor,
            use_heuristic_score,
            use_caching,
            debug,
        );
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

        if turn_counter == 1 {
            // check starting_player
            if opponent_row >= 0 {
                starting_player = MonteCarloPlayer::Opp;
                turn_counter += 1;
                let opp_action =
                    MapPoint::<U, V>::new(opponent_col as usize, opponent_row as usize);
                game_data.set_last_opp_action(opp_action);
            }
        } else {
            // update opp action
            let opp_action = MapPoint::<U, V>::new(opponent_col as usize, opponent_row as usize);
            game_data.set_last_opp_action(opp_action);
        }

        // Write an action using println!("message...");
        // To debug: eprintln!("Debug message...");

        let start = mcts.init_root(&game_data, starting_player);
        mcts.expand_tree(start);
        let (_my_game_data, my_action) = mcts.choose_and_execute_actions();
        my_action.execute_action();

        turn_counter += 2;
    }
}
