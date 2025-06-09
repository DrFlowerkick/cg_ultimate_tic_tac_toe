// lib for ult_ttt game code

pub mod config;
pub use config::*;

pub mod caching;
pub use caching::*;

pub mod heuristic;
pub use heuristic::*;

pub mod utilities;

mod old_heuristic;

use my_lib::my_map_3x3::*;
use my_lib::my_mcts::{
    GamePlayer, HeuristicProgressiveWidening, MCTSGame, NoGameCache, ProgressiveWidening,
};
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

pub type HPWDefaultTTTWithGameCache = ProgressiveWidening<UltTTTMCTSGame, UltTTTMCTSConfig>;
pub type HPWDefaultTTTNoGameCache =
    HeuristicProgressiveWidening<UltTTTMCTSGame, UltTTTHeuristic, UltTTTMCTSConfig>;

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
        let status = new_state.map.get_cell(mv.status_index).get_status();
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

#[cfg(test)]
mod tests;
