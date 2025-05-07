// lib for ult_ttt game code

use my_lib::my_map_3x3::*;
use my_lib::my_monte_carlo_tree_search::*;
use my_lib::my_tic_tac_toe::*;

use std::cmp::Ordering;
use std::collections::HashMap;
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
}

impl UltTTT {
    pub fn new() -> Self {
        UltTTT {
            map: MyMap3x3::new(),
            status_map: TicTacToeGameData::new(),
            next_action_constraint: NextActionConstraint::Init,
            current_player: TicTacToeStatus::Me,
        }
    }
    pub fn set_current_player(&mut self, player: TicTacToeStatus) {
        self.current_player = player;
    }
    pub fn next_player(&mut self) {
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

pub struct UltTTTMCTSGame<GC: GameCache<UltTTT, UltTTTMove>> {
    phantom: std::marker::PhantomData<GC>,
}

impl<GC: GameCache<UltTTT, UltTTTMove> + UltTTTGameCacheTrait> MCTSGame for UltTTTMCTSGame<GC> {
    type State = UltTTT;
    type Move = UltTTTMove;
    type Player = TicTacToeStatus;
    type Cache = GC;

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
        game_cache: &mut Self::Cache,
    ) -> Self::State {
        let mut new_state = *state;
        // apply the move for current player
        new_state
            .map
            .get_cell_mut(mv.status_index)
            .set_cell_value(mv.mini_board_index, state.current_player);
        let status = game_cache.cache_tic_tac_toe_status_increment(
            new_state.map.get_cell(mv.status_index),
            mv.mini_board_index,
        );
        if !status.is_vacant() {
            // depending on cache it may be useful to set all cells to status;
            // e.g. if we cache nodes, identical states of status map may result from different moves in mini boards,
            // If we do net set all cells, this may still result in different nodes in the tree, which would
            // reduce cache effectiveness.
            // at the current state we do not need this
            /*self.map
            .get_cell_mut(player_move.status_index)
            .set_all_cells(status);*/
            new_state.status_map.set_cell_value(mv.status_index, status);
            game_cache.cache_tic_tac_toe_status_increment(&new_state.status_map, mv.status_index);
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

    fn evaluate(state: &Self::State, game_cache: &mut Self::Cache) -> Option<f32> {
        let mut status = game_cache.cache_tic_tac_toe_status(&state.status_map);
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
    fn perspective_player() -> Self::Player {
        TicTacToeStatus::Me
    }
}

pub trait UltTTTGameCacheTrait {
    fn cache_tic_tac_toe_status(&mut self, ttt: &TicTacToeGameData) -> TicTacToeStatus;
    fn cache_tic_tac_toe_status_increment(
        &mut self,
        ttt: &TicTacToeGameData,
        cell: CellIndex3x3,
    ) -> TicTacToeStatus;
}

pub struct UltTTTGameCache {
    pub tic_tac_toe_cache: HashMap<TicTacToeGameData, TicTacToeStatus>,
    pub cache_usage: usize,
}

impl UltTTTGameCacheTrait for UltTTTGameCache {
    fn cache_tic_tac_toe_status(&mut self, ttt: &TicTacToeGameData) -> TicTacToeStatus {
        if let Some(cached_status) = self.tic_tac_toe_cache.get(ttt) {
            self.cache_usage += 1;
            *cached_status
        } else {
            let status = ttt.get_status();
            self.tic_tac_toe_cache.insert(*ttt, status);
            status
        }
    }
    fn cache_tic_tac_toe_status_increment(
        &mut self,
        ttt: &TicTacToeGameData,
        cell: CellIndex3x3,
    ) -> TicTacToeStatus {
        if let Some(cached_status) = self.tic_tac_toe_cache.get(ttt) {
            self.cache_usage += 1;
            *cached_status
        } else {
            let status = ttt.get_status_increment(&cell);
            self.tic_tac_toe_cache.insert(*ttt, status);
            status
        }
    }
}

impl GameCache<UltTTT, UltTTTMove> for UltTTTGameCache {
    fn new() -> Self {
        UltTTTGameCache {
            tic_tac_toe_cache: HashMap::new(),
            cache_usage: 0,
        }
    }
}

impl UltTTTGameCacheTrait for NoGameCache<UltTTT, UltTTTMove> {
    fn cache_tic_tac_toe_status(&mut self, ttt: &TicTacToeGameData) -> TicTacToeStatus {
        ttt.get_status()
    }
    fn cache_tic_tac_toe_status_increment(
        &mut self,
        ttt: &TicTacToeGameData,
        cell: CellIndex3x3,
    ) -> TicTacToeStatus {
        ttt.get_status_increment(&cell)
    }
}

pub struct UltTTTHeuristic<HC: HeuristicCache<UltTTT, UltTTTMove>> {
    phantom: std::marker::PhantomData<HC>,
}

impl<
        GC: GameCache<UltTTT, UltTTTMove> + UltTTTGameCacheTrait,
        HC: HeuristicCache<UltTTT, UltTTTMove>,
    > Heuristic<UltTTTMCTSGame<GC>> for UltTTTHeuristic<HC>
{
    type Cache = HC;

    fn evaluate_state(
        state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
        game_cache: &mut <UltTTTMCTSGame<GC> as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
    ) -> f32 {
        if let Some(cache_value) = heuristic_cache.get_intermediate_score(state) {
            return cache_value;
        }
        let heuristic = match UltTTTMCTSGame::evaluate(state, game_cache) {
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
                    let (my, opp) = state.map.get_cell(status_index).get_threats();
                    my_threats += my as f32;
                    opp_threats += opp as f32;
                }
                // calculate heuristic value
                let progress = state.status_map.count_non_vacant_cells() as f32 / 9.0;
                let meta_weight = 0.3 + 0.4 * progress;
                let threat_weight = 1.0 - meta_weight;

                let final_score = 0.5
                    + meta_weight * (my_wins - opp_wins) / 9.0
                    + threat_weight * (my_threats - opp_threats) / 20.0;

                final_score.clamp(0.0, 1.0)
            }
        };
        heuristic_cache.insert_intermediate_score(state, heuristic);
        heuristic
    }

    fn evaluate_move(
        _state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
        _mv: &<UltTTTMCTSGame<GC> as MCTSGame>::Move,
        _game_cache: &mut <UltTTTMCTSGame<GC> as MCTSGame>::Cache,
        _heuristic_cache: &mut Self::Cache,
    ) -> f32 {
        0.0
    }
}

use std::cell::RefCell;
pub struct UltTTTHeuristicCache<GC: GameCache<UltTTT, UltTTTMove> + UltTTTGameCacheTrait> {
    pub heuristic_cache: HashMap<UltTTT, f32>,
    pub cache_usage: RefCell<usize>,
    phantom: std::marker::PhantomData<GC>,
}

impl<GC: GameCache<UltTTT, UltTTTMove> + UltTTTGameCacheTrait> HeuristicCache<UltTTT, UltTTTMove>
    for UltTTTHeuristicCache<GC>
{
    fn new() -> Self {
        UltTTTHeuristicCache {
            heuristic_cache: HashMap::new(),
            cache_usage: RefCell::new(0),
            phantom: std::marker::PhantomData,
        }
    }
    fn get_intermediate_score(
        &self,
        state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
    ) -> Option<f32> {
        match self.heuristic_cache.get(state) {
            Some(&value) => {
                *self.cache_usage.borrow_mut() += 1;
                Some(value)
            }
            None => None,
        }
    }
    fn insert_intermediate_score(
        &mut self,
        state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
        value: f32,
    ) {
        self.heuristic_cache.insert(*state, value);
    }
}

pub type UltTTTSimulationPolicy = HeuristicCutoff<20>;

#[cfg(test)]
mod tests {
    type CachedUltTTTMCTSGame = UltTTTMCTSGame<UltTTTGameCache>;
    type NoCachedUltTTTMCTSGame = UltTTTMCTSGame<NoGameCache<UltTTT, UltTTTMove>>;
    type PWDefaultTTT = PWDefault<CachedUltTTTMCTSGame>;
    type ExpandAllTTT = ExpandAll<NoCachedUltTTTMCTSGame>;
    type CachedUltTTTMCTSHeuristic = UltTTTHeuristic<UltTTTHeuristicCache<UltTTTGameCache>>;
    type NoCachedUltTTTMCTSHeuristic = UltTTTHeuristic<NoHeuristicCache<UltTTT, UltTTTMove>>;
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
                CachedUltTTTMCTSGame,
                DynamicC,
                CachedUTC,
                PWDefaultTTT,
                CachedUltTTTMCTSHeuristic,
                UltTTTSimulationPolicy,
            > = PlainMCTS::new(WEIGHTING_FACTOR);
            let mut first_ult_ttt_game_data = UltTTT::new();
            first_ult_ttt_game_data.set_current_player(TicTacToeStatus::Me);
            let mut first_time_out = TIME_OUT_FIRST_TURN;
            let mut second_mcts_ult_ttt: PlainMCTS<
                NoCachedUltTTTMCTSGame,
                DynamicC,
                CachedUTC,
                PWDefault<NoCachedUltTTTMCTSGame>,
                NoCachedUltTTTMCTSHeuristic,
                UltTTTSimulationPolicy,
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
            eprintln!(
                "Game cache usage: {}, cache size: {}",
                first_mcts_ult_ttt.game_cache.cache_usage,
                first_mcts_ult_ttt.game_cache.tic_tac_toe_cache.len()
            );
            eprintln!(
                "Heuristic cache usage: {}, cache size: {}",
                first_mcts_ult_ttt.heuristic_cache.cache_usage.borrow(),
                first_mcts_ult_ttt.heuristic_cache.heuristic_cache.len()
            );
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
