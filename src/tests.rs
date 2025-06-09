// tests of UltTTT

use std::time::{Duration, Instant};

use super::old_heuristic::OldUltTTTHeuristic;
use super::*;
use my_lib::my_mcts::{
    CachedUTC, DefaultSimulationPolicy, DynamicC, ExpandAll, HeuristicCutoff, MCTSAlgo,
    NoHeuristic, NoTranspositionTable, PlainMCTS, PlainTTHashMap,
};

pub type HPWDefaultTTTOldHeuristic =
    HeuristicProgressiveWidening<UltTTTMCTSGame, OldUltTTTHeuristic, UltTTTMCTSConfig>;

const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(995);
const TIME_OUT_SUCCESSIVE_TURNS: Duration = Duration::from_millis(95);

#[test]
fn test_mcts_ult_ttt_no_game_cache() {
    let mut wins = 0.0;
    let number_of_matches = 10;
    for i in 0..number_of_matches {
        eprintln!("________match {}________", i + 1);
        let mut first_mcts_ult_ttt: PlainMCTS<
            UltTTTMCTSGame,
            UltTTTHeuristic,
            UltTTTMCTSConfig,
            CachedUTC,
            PlainTTHashMap<UltTTT>,
            DynamicC,
            HPWDefaultTTTNoGameCache,
            HeuristicCutoff,
        > = PlainMCTS::new(
            UltTTTMCTSConfig::default(),
            UltTTTHeuristicConfig::default(),
        );
        let mut first_ult_ttt_game_data = UltTTT::new();
        first_ult_ttt_game_data.set_current_player(TicTacToeStatus::Me);
        let mut first_time_out = TIME_OUT_FIRST_TURN;
        let mut second_mcts_ult_ttt: PlainMCTS<
            UltTTTMCTSGame,
            NoHeuristic,
            UltTTTMCTSConfig,
            CachedUTC,
            NoTranspositionTable,
            DynamicC,
            ExpandAll,
            DefaultSimulationPolicy,
        > = PlainMCTS::new(UltTTTMCTSConfig::default(), NoHeuristic {});
        let mut second_ult_ttt_game_data = UltTTT::new();
        second_ult_ttt_game_data.set_current_player(TicTacToeStatus::Opp);
        let mut second_time_out = TIME_OUT_FIRST_TURN;

        let mut first = true;

        while UltTTTMCTSGame::evaluate(&first_ult_ttt_game_data, &mut first_mcts_ult_ttt.game_cache)
            .is_none()
        {
            if first {
                let start = Instant::now();
                first_mcts_ult_ttt.set_root(&first_ult_ttt_game_data);
                let mut number_of_iterations = 0;
                while start.elapsed() < first_time_out {
                    first_mcts_ult_ttt.iterate();
                    number_of_iterations += 1;
                }
                first_time_out = TIME_OUT_SUCCESSIVE_TURNS;
                let selected_move = *first_mcts_ult_ttt.select_move();
                let (x_ttt, y_ttt) = <(u8, u8)>::from(selected_move);
                eprintln!(
                    "first : ({}, {}) (number_of_iterations: {})",
                    x_ttt, y_ttt, number_of_iterations
                );
                first_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                    &first_ult_ttt_game_data,
                    &selected_move,
                    &mut first_mcts_ult_ttt.game_cache,
                );
                second_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                    &second_ult_ttt_game_data,
                    &selected_move,
                    &mut second_mcts_ult_ttt.game_cache,
                );
                first = false;
            } else {
                let start = Instant::now();
                second_mcts_ult_ttt.set_root(&second_ult_ttt_game_data);
                let mut number_of_iterations = 0;
                while start.elapsed() < second_time_out {
                    second_mcts_ult_ttt.iterate();
                    number_of_iterations += 1;
                }
                second_time_out = TIME_OUT_SUCCESSIVE_TURNS;
                let selected_move = *second_mcts_ult_ttt.select_move();
                let (x_ttt, y_ttt) = <(u8, u8)>::from(selected_move);
                eprintln!(
                    "second: ({}, {}) (number_of_iterations: {})",
                    x_ttt, y_ttt, number_of_iterations
                );
                second_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                    &second_ult_ttt_game_data,
                    &selected_move,
                    &mut second_mcts_ult_ttt.game_cache,
                );
                first_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                    &first_ult_ttt_game_data,
                    &selected_move,
                    &mut first_mcts_ult_ttt.game_cache,
                );
                first = true;
            }
        }
        eprintln!("Game ended");
        eprintln!("{}", first_ult_ttt_game_data);
        match first_ult_ttt_game_data.status_map.get_status() {
            TicTacToeStatus::Me => {
                eprintln!("first winner");
            }
            TicTacToeStatus::Opp => {
                eprintln!("second winner");
            }
            TicTacToeStatus::Tie => eprintln!("tie"),
            TicTacToeStatus::Vacant => {
                eprintln!("vacant: Game ended without winner!?");
                assert!(false, "vacant: Game ended without winner!?");
            }
        }
        wins +=
            UltTTTMCTSGame::evaluate(&first_ult_ttt_game_data, &mut first_mcts_ult_ttt.game_cache)
                .unwrap();
    }
    println!("{} wins out of {} matches.", wins, number_of_matches);
    //assert_eq!(wins, 25.0);
}

#[test]
fn test_mcts_ult_ttt_new_vs_old_heuristic() {
    let mut wins = 0.0;
    let number_of_matches = 100;
    for i in 0..number_of_matches {
        eprintln!("________match {}________", i + 1);
        let mut first_mcts_ult_ttt: PlainMCTS<
            UltTTTMCTSGame,
            UltTTTHeuristic,
            UltTTTMCTSConfig,
            CachedUTC,
            PlainTTHashMap<UltTTT>,
            DynamicC,
            HPWDefaultTTTNoGameCache,
            HeuristicCutoff,
        > = PlainMCTS::new(
            UltTTTMCTSConfig::new_optimized(),
            UltTTTHeuristicConfig::new_optimized(),
        );
        let mut first_ult_ttt_game_data = UltTTT::new();
        first_ult_ttt_game_data.set_current_player(TicTacToeStatus::Me);
        let mut first_time_out = TIME_OUT_FIRST_TURN;
        let mut second_mcts_ult_ttt: PlainMCTS<
            UltTTTMCTSGame,
            OldUltTTTHeuristic,
            UltTTTMCTSConfig,
            CachedUTC,
            NoTranspositionTable,
            DynamicC,
            HPWDefaultTTTOldHeuristic,
            HeuristicCutoff,
        > = PlainMCTS::new(
            UltTTTMCTSConfig::optimized(),
            UltTTTHeuristicConfig::optimized(),
        );
        let mut second_ult_ttt_game_data = UltTTT::new();
        second_ult_ttt_game_data.set_current_player(TicTacToeStatus::Opp);
        let mut second_time_out = TIME_OUT_FIRST_TURN;

        let mut first = i % 2 == 0;

        while UltTTTMCTSGame::evaluate(&first_ult_ttt_game_data, &mut first_mcts_ult_ttt.game_cache)
            .is_none()
        {
            if first {
                let start = Instant::now();
                first_mcts_ult_ttt.set_root(&first_ult_ttt_game_data);
                while start.elapsed() < first_time_out {
                    first_mcts_ult_ttt.iterate();
                }
                first_time_out = TIME_OUT_SUCCESSIVE_TURNS;
                let selected_move = *first_mcts_ult_ttt.select_move();
                first_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                    &first_ult_ttt_game_data,
                    &selected_move,
                    &mut first_mcts_ult_ttt.game_cache,
                );
                second_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                    &second_ult_ttt_game_data,
                    &selected_move,
                    &mut second_mcts_ult_ttt.game_cache,
                );
                first = false;
            } else {
                let start = Instant::now();
                second_mcts_ult_ttt.set_root(&second_ult_ttt_game_data);
                while start.elapsed() < second_time_out {
                    second_mcts_ult_ttt.iterate();
                }
                second_time_out = TIME_OUT_SUCCESSIVE_TURNS;
                let selected_move = *second_mcts_ult_ttt.select_move();
                second_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                    &second_ult_ttt_game_data,
                    &selected_move,
                    &mut second_mcts_ult_ttt.game_cache,
                );
                first_ult_ttt_game_data = UltTTTMCTSGame::apply_move(
                    &first_ult_ttt_game_data,
                    &selected_move,
                    &mut first_mcts_ult_ttt.game_cache,
                );
                first = true;
            }
        }
        eprint!("Game ended: ");
        match first_ult_ttt_game_data.status_map.get_status() {
            TicTacToeStatus::Me => {
                eprintln!("first winner");
            }
            TicTacToeStatus::Opp => {
                eprintln!("second winner");
            }
            TicTacToeStatus::Tie => eprintln!("tie"),
            TicTacToeStatus::Vacant => {
                eprintln!("vacant: Game ended without winner!?");
                assert!(false, "vacant: Game ended without winner!?");
            }
        }
        wins +=
            UltTTTMCTSGame::evaluate(&first_ult_ttt_game_data, &mut first_mcts_ult_ttt.game_cache)
                .unwrap();
    }
    println!(
        "New heuristic wins {} out of {} matches.",
        wins, number_of_matches
    );
    //assert_eq!(wins, 25.0);
}
