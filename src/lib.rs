// lib for ult_ttt game code

use my_lib::my_map_3x3::*;
use my_lib::my_monte_carlo_tree_search::*;
use my_lib::my_tic_tac_toe::*;

use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Display;
use std::fmt::Write;

#[derive(Copy, Clone, PartialEq, Default)]
pub struct UltTTTMove {
    pub status_index: CellIndex3x3,
    pub mini_board_index: CellIndex3x3,
}

impl TryFrom<(u8, u8)> for UltTTTMove {
    type Error = ();
    fn try_from(cg_coordinates: (u8, u8)) -> Result<Self, Self::Error> {
        let x_status = cg_coordinates.0 / 3;
        let y_status = cg_coordinates.1 / 3;
        let x_mini_board = cg_coordinates.0 % 3;
        let y_mini_board = cg_coordinates.1 % 3;

        Ok(UltTTTMove {
            status_index: CellIndex3x3::try_from((x_status, y_status))?,
            mini_board_index: CellIndex3x3::try_from((x_mini_board, y_mini_board))?,
        })
    }
}

impl From<UltTTTMove> for (u8, u8) {
    fn from(player_move: UltTTTMove) -> (u8, u8) {
        let (x_status, y_status) = player_move.status_index.into();
        let (x_mini_board, y_mini_board) = player_move.mini_board_index.into();
        let x = x_status * 3 + x_mini_board;
        let y = y_status * 3 + y_mini_board;
        (x, y)
    }
}

impl UltTTTMove {
    pub fn valid_move(
        status_index: CellIndex3x3,
        mini_board_index: CellIndex3x3,
        cell_value: &TicTacToeStatus,
    ) -> Option<UltTTTMove> {
        match cell_value {
            TicTacToeStatus::Vacant => Some(UltTTTMove {
                status_index,
                mini_board_index,
            }),
            _ => None,
        }
    }
    pub fn execute_action(&self) -> String {
        let mut action_commando_string = String::new();
        let action = <(u8, u8)>::from(*self);
        write!(action_commando_string, "{} {}", action.1, action.0).unwrap();
        println!("{}", action_commando_string);
        action_commando_string
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum NextActionConstraint {
    #[default]
    Init,
    None,
    MiniBoard(CellIndex3x3),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct UltTTT {
    map: MyMap3x3<TicTacToeGameData>,
    status_map: TicTacToeGameData,
    next_action_constraint: NextActionConstraint,
    current_player: TicTacToeStatus,
    last_player: TicTacToeStatus,
}

impl UltTTT {
    pub fn new() -> Self {
        UltTTT {
            map: MyMap3x3::new(),
            status_map: TicTacToeGameData::new(),
            next_action_constraint: NextActionConstraint::Init,
            current_player: TicTacToeStatus::Me,
            last_player: TicTacToeStatus::Me,
        }
    }
    pub fn set_current_player(&mut self, player: TicTacToeStatus) {
        self.current_player = player;
    }
    pub fn next_player(&mut self) {
        self.last_player = self.current_player;
        self.current_player = self.current_player.next();
    }
    fn get_cell_value(&self, cell: UltTTTMove) -> TicTacToeStatus {
        self.map
            .get_cell(cell.status_index)
            .get_cell_value(cell.mini_board_index)
    }
}

impl Display for UltTTT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const HEAD: &str = "┌─┬─┬─┐┌─┬─┬─┐┌─┬─┬─┐";
        const MIDDLE: &str = "├─┼─┼─┤├─┼─┼─┤├─┼─┼─┤";
        const FOOTER: &str = "└─┴─┴─┘└─┴─┴─┘└─┴─┴─┘";
        for v in 0..9 {
            if v % 3 == 0 {
                writeln!(f, "{}", HEAD)?;
            }
            for u in 0..9 {
                if u % 3 == 0 {
                    write!(f, "│")?;
                }
                let cell_to_print = UltTTTMove::try_from((u, v)).unwrap();
                write!(f, "{}│", self.get_cell_value(cell_to_print))?;
                if u == 8 {
                    writeln!(f)?;
                }
            }
            if v % 3 < 2 {
                writeln!(f, "{}", MIDDLE)?;
            } else {
                write!(f, "{}", FOOTER)?;
                if v < 8 {
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}

pub struct UltTTTMCTSGame {}

impl MCTSGame for UltTTTMCTSGame {
    type State = UltTTT;
    type Move = UltTTTMove;
    type Player = TicTacToeStatus;
    type Cache = NoGameCache<UltTTT, UltTTTMove>;

    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a> {
        match state.next_action_constraint {
            NextActionConstraint::Init => Box::new(
                state
                    .map
                    .get_cell(CellIndex3x3::MM)
                    .iter_map()
                    .filter_map(|(i, c)| UltTTTMove::valid_move(CellIndex3x3::MM, i, c)),
            ),
            NextActionConstraint::MiniBoard(constraint) => Box::new(
                state
                    .map
                    .get_cell(constraint)
                    .iter_map()
                    .filter_map(move |(i, c)| UltTTTMove::valid_move(constraint, i, c)),
            ),
            NextActionConstraint::None => Box::new(
                state
                    .status_map
                    .iter_map()
                    .filter_map(|(i, c)| match c {
                        TicTacToeStatus::Vacant => Some(i),
                        _ => None,
                    })
                    .flat_map(move |status_cell| {
                        state
                            .map
                            .get_cell(status_cell)
                            .iter_map()
                            .filter_map(move |(i, c)| UltTTTMove::valid_move(status_cell, i, c))
                    }),
            ),
        }
    }

    fn apply_move(
        state: &Self::State,
        mv: &Self::Move,
        _game_cache: &mut Self::Cache,
    ) -> Self::State {
        let mut new_state = *state;
        // apply the move for current player
        new_state
            .map
            .get_cell_mut(mv.status_index)
            .set_cell_value(mv.mini_board_index, state.current_player);
        let status = new_state
            .map
            .get_cell(mv.status_index)
            .get_status_increment(&mv.mini_board_index);
        if !status.is_vacant() {
            new_state.status_map.set_cell_value(mv.status_index, status);
        }

        // player_move.mini_board_index points to next TicTacToe for next player to set new value.
        // if this TicTacToe status is not vacant (meaning there are no more cells to set), player can choose from all free cells
        new_state.next_action_constraint = if new_state
            .status_map
            .get_cell_value(mv.mini_board_index)
            .is_vacant()
        {
            NextActionConstraint::MiniBoard(mv.mini_board_index)
        } else {
            NextActionConstraint::None
        };
        // set the next player
        new_state.next_player();
        new_state
    }

    fn evaluate(state: &Self::State, _game_cache: &mut Self::Cache) -> Option<f32> {
        let mut status = state.status_map.get_status();
        if status == TicTacToeStatus::Tie {
            // game finished without direct winner
            // count for each player number of won squares; most squares won wins game
            let my_squares = state.status_map.count_me_cells();
            let opp_squares = state.status_map.count_opp_cells();
            status = match my_squares.cmp(&opp_squares) {
                Ordering::Greater => TicTacToeStatus::Me,
                Ordering::Less => TicTacToeStatus::Opp,
                Ordering::Equal => TicTacToeStatus::Tie,
            };
        }
        status.evaluate()
    }
    fn current_player(state: &Self::State) -> TicTacToeStatus {
        state.current_player
    }
    fn last_player(state: &Self::State) -> Self::Player {
        state.last_player
    }
    fn perspective_player() -> Self::Player {
        TicTacToeStatus::Me
    }
}

pub struct UltTTTHeuristic {}

impl Heuristic<UltTTTMCTSGame> for UltTTTHeuristic {
    type Cache = NoHeuristicCache<UltTTT, UltTTTMove>;

    fn evaluate_state(
        state: &<UltTTTMCTSGame as MCTSGame>::State,
        game_cache: &mut <UltTTTMCTSGame as MCTSGame>::Cache,
        _heuristic_cache: &mut Self::Cache,
        perspective_player: Option<<UltTTTMCTSGame as MCTSGame>::Player>,
    ) -> f32 {
        let player = match perspective_player {
            Some(player) => player,
            None => state.last_player,
        };
        let score = match UltTTTMCTSGame::evaluate(state, game_cache) {
            Some(value) => value,
            None => {
                // meta progress: wins on status_map
                let my_wins = state.status_map.count_me_cells() as f32;
                let opp_wins = state.status_map.count_opp_cells() as f32;
                // mini board threats
                let mut my_threats = 0.0;
                let mut opp_threats = 0.0;
                for (status_index, _) in state.status_map.iter_map().filter(|(_, c)| c.is_vacant())
                {
                    let cell_weight = status_index.cell_weight();
                    let (my_meta_factor, opp_meta_factor) =
                        state.status_map.get_meta_cell_factors(status_index);
                    let (my, opp) = state.map.get_cell(status_index).get_threats();
                    my_threats += my_meta_factor * cell_weight * my as f32;
                    opp_threats += opp_meta_factor * cell_weight * opp as f32;
                }
                // calculate heuristic value
                let progress = state.status_map.count_non_vacant_cells() as f32 / 9.0;
                let meta_weight = 0.3 + 0.4 * progress;
                let threat_weight = 1.0 - meta_weight;
                let max_threat_score = 1.0_f32.max(my_threats + opp_threats);

                let final_score = 0.5
                    + meta_weight * (my_wins - opp_wins) / 9.0
                    + threat_weight * (my_threats - opp_threats) / max_threat_score;

                final_score.clamp(0.0, 1.0)
            }
        };

        match player {
            TicTacToeStatus::Me => score,
            TicTacToeStatus::Opp => 1.0 - score,
            _ => unreachable!("Player is alway Me or Opp"),
        }
    }

    fn evaluate_move(
        state: &<UltTTTMCTSGame as MCTSGame>::State,
        mv: &<UltTTTMCTSGame as MCTSGame>::Move,
        game_cache: &mut <UltTTTMCTSGame as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
    ) -> f32 {
        let new_state = UltTTTMCTSGame::apply_move(state, mv, game_cache);
        UltTTTHeuristic::evaluate_state(&new_state, game_cache, heuristic_cache, None)
    }
}

pub type UltTTTSimulationPolicy = HeuristicCutoff<20>;

#[cfg(test)]
mod tests {
    type PWDefaultTTT = PWDefault<UltTTTMCTSGame>;
    type HPWDefaultTTT = HPWDefault<UltTTTMCTSGame>;
    type ExpandAllTTT = ExpandAll<UltTTTMCTSGame>;
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn test_new_mcts_traits_with_ult_ttt() {
        const WEIGHTING_FACTOR: f32 = 1.4;
        const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(995);
        const TIME_OUT_SUCCESSIVE_TURNS: Duration = Duration::from_millis(95);

        let mut wins = 0.0;
        let number_of_matches = 10;
        for i in 0..number_of_matches {
            eprintln!("________match {}________", i + 1);
            let mut first_mcts_ult_ttt: PlainMCTS<
                UltTTTMCTSGame,
                DynamicC,
                CachedUTC,
                HPWDefaultTTT,
                UltTTTHeuristic,
                UltTTTSimulationPolicy,
            > = PlainMCTS::new(WEIGHTING_FACTOR);
            let mut first_ult_ttt_game_data = UltTTT::new();
            first_ult_ttt_game_data.set_current_player(TicTacToeStatus::Me);
            let mut first_time_out = TIME_OUT_FIRST_TURN;
            let mut second_mcts_ult_ttt: PlainMCTS<
                UltTTTMCTSGame,
                DynamicC,
                CachedUTC,
                ExpandAllTTT,
                NoHeuristic,
                DefaultSimulationPolicy,
            > = PlainMCTS::new(WEIGHTING_FACTOR);
            let mut second_ult_ttt_game_data = UltTTT::new();
            second_ult_ttt_game_data.set_current_player(TicTacToeStatus::Opp);
            let mut second_time_out = TIME_OUT_FIRST_TURN;

            let mut first = true;

            while UltTTTMCTSGame::evaluate(
                &first_ult_ttt_game_data,
                &mut first_mcts_ult_ttt.game_cache,
            )
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
            wins += UltTTTMCTSGame::evaluate(
                &first_ult_ttt_game_data,
                &mut first_mcts_ult_ttt.game_cache,
            )
            .unwrap();
        }
        println!("{} wins out of {} matches.", wins, number_of_matches);
        //assert_eq!(wins, 25.0);
    }
}
