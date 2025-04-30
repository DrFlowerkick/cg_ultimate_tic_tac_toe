// lib for ult_ttt game code

use my_lib::my_map_point::*;
use my_lib::my_map_two_dim::*;
use my_lib::my_monte_carlo_tree_search::*;
use my_lib::my_tic_tac_toe::*;

use std::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Write;

pub const U: usize = 9;
pub const V: usize = U;

pub struct IterUltTTT<'a> {
    ult_ttt_data: &'a UltTTT,
    player_action: UltTTTPlayerAction,
    next_action_square_is_specified: bool,
    iter_finished: bool,
}

impl<'a> IterUltTTT<'a> {
    pub fn new(
        ult_ttt_data: &'a UltTTT,
        player: MonteCarloPlayer,
        parent_game_turn: usize,
    ) -> Self {
        let mut result = IterUltTTT {
            ult_ttt_data,
            player_action: UltTTTPlayerAction::default(),
            next_action_square_is_specified: false,
            iter_finished: false,
        };
        if parent_game_turn == 0 && player == MonteCarloPlayer::Me {
            // if Me is start_player, only choose cells from big middle cell
            result.player_action.ult_ttt_big = MapPoint::<X, Y>::new(1, 1);
            result.player_action.ult_ttt_small = MapPoint::<X, Y>::new(0, 0);
            result.next_action_square_is_specified = true;
        } else if let Some(next_ult_ttt_big) = ult_ttt_data.next_action_square_is_specified {
            result.player_action.ult_ttt_big = next_ult_ttt_big;
            result.player_action.ult_ttt_small = ult_ttt_data
                .map
                .get(next_ult_ttt_big)
                .get_first_vacant_cell()
                .unwrap()
                .0;
            result.next_action_square_is_specified = true;
        } else {
            match result.ult_ttt_data.status_map.get_first_vacant_cell() {
                Some((new_iter_ttt_big, _)) => {
                    result.player_action.ult_ttt_big = new_iter_ttt_big;
                    result.player_action.ult_ttt_small = ult_ttt_data
                        .map
                        .get(new_iter_ttt_big)
                        .get_first_vacant_cell()
                        .unwrap()
                        .0;
                }
                None => result.iter_finished = true,
            };
        }
        result
    }
}

impl<'a> Iterator for IterUltTTT<'a> {
    type Item = UltTTTPlayerAction;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter_finished {
            return None;
        }
        let result = self.player_action;
        let mut searching_new_iter_ttt_big = true;
        while searching_new_iter_ttt_big {
            if self
                .ult_ttt_data
                .status_map
                .get_cell_value(self.player_action.ult_ttt_big)
                .is_vacant()
            {
                let mut searching_new_iter_ttt_small = true;
                while searching_new_iter_ttt_small {
                    match self.player_action.ult_ttt_small.forward_x() {
                        Some(new_iter_ttt_small) => {
                            self.player_action.ult_ttt_small = new_iter_ttt_small;
                            if self
                                .ult_ttt_data
                                .map
                                .get(self.player_action.ult_ttt_big)
                                .get_cell_value(self.player_action.ult_ttt_small)
                                .is_vacant()
                            {
                                return Some(result);
                            }
                        }
                        None => {
                            if self.next_action_square_is_specified {
                                self.iter_finished = true;
                                return Some(result);
                            }
                            self.player_action.ult_ttt_small = MapPoint::<X, Y>::new(0, 0);
                            searching_new_iter_ttt_small = false;
                        }
                    }
                }
            }
            match self.player_action.ult_ttt_big.forward_x() {
                Some(new_iter_ttt_big) => {
                    self.player_action.ult_ttt_big = new_iter_ttt_big;
                    if self
                        .ult_ttt_data
                        .status_map
                        .get_cell_value(self.player_action.ult_ttt_big)
                        .is_vacant()
                        && self
                            .ult_ttt_data
                            .map
                            .get(self.player_action.ult_ttt_big)
                            .get_cell_value(self.player_action.ult_ttt_small)
                            .is_vacant()
                    {
                        return Some(result);
                    }
                }
                None => {
                    self.iter_finished = true;
                    searching_new_iter_ttt_big = false;
                }
            }
        }
        Some(result)
    }
}

#[derive(Copy, Clone, PartialEq, Default)]
pub struct UltTTTPlayerAction {
    pub ult_ttt_big: MapPoint<X, Y>,
    pub ult_ttt_small: MapPoint<X, Y>,
}

impl UltTTTPlayerAction {
    pub fn from_ext(extern_coordinates: MapPoint<U, V>) -> UltTTTPlayerAction {
        UltTTTPlayerAction {
            ult_ttt_big: MapPoint::<X, Y>::new(
                extern_coordinates.x() / X,
                extern_coordinates.y() / Y,
            ),
            ult_ttt_small: MapPoint::<X, Y>::new(
                extern_coordinates.x() % X,
                extern_coordinates.y() % Y,
            ),
        }
    }

    pub fn to_ext(self) -> MapPoint<U, V> {
        MapPoint::<U, V>::new(
            self.ult_ttt_big.x() * X + self.ult_ttt_small.x(),
            self.ult_ttt_big.y() * Y + self.ult_ttt_small.y(),
        )
    }
    pub fn execute_action(&self) -> String {
        let mut action_commando_string = String::new();
        let action = self.to_ext();
        write!(action_commando_string, "{} {}", action.y(), action.x()).unwrap();
        println!("{}", action_commando_string);
        action_commando_string
    }
}

#[derive(Copy, Clone, PartialEq, Default)]
pub struct UltTTTGameDataUpdate {}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct UltTTT {
    map: MyMap2D<TicTacToeGameData, X, Y>,
    pub status_map: TicTacToeGameData,
    pub status: TicTacToeStatus,
    next_action_square_is_specified: Option<MapPoint<X, Y>>,
    // required for new MCTS traits
    pub current_player: MonteCarloPlayer,
    // since new MCTS traits do not require game_turn, we need to store it here
    pub game_turn: usize,
}

impl UltTTT {
    pub fn new() -> Self {
        UltTTT {
            map: MyMap2D::new(),
            status_map: TicTacToeGameData::new(),
            status: TicTacToeStatus::Vacant,
            next_action_square_is_specified: None,
            current_player: MonteCarloPlayer::Me,
            game_turn: 0,
        }
    }
    pub fn set_current_player(&mut self, player: MonteCarloPlayer) {
        self.current_player = player;
    }
    pub fn next_player(&mut self) {
        self.current_player = self.current_player.next_player();
    }
    pub fn increment_game_turn(&mut self) {
        self.game_turn += 1;
    }
    fn get_cell_value(&self, cell: UltTTTPlayerAction) -> TicTacToeStatus {
        self.map
            .get(cell.ult_ttt_big)
            .get_cell_value(cell.ult_ttt_small)
    }
    pub fn execute_player_action(
        &mut self,
        player_action: UltTTTPlayerAction,
        player: MonteCarloPlayer,
    ) -> TicTacToeStatus {
        let cell_status = self
            .map
            .get_mut(player_action.ult_ttt_big)
            .set_player(player_action.ult_ttt_small, player);
        self.status = match cell_status {
            TicTacToeStatus::Vacant => self.status,
            TicTacToeStatus::Player(winner) => {
                self.map
                    .get_mut(player_action.ult_ttt_big)
                    .set_all_to_status();
                self.status_map
                    .set_player(player_action.ult_ttt_big, winner)
            }
            TicTacToeStatus::Tie => self.status_map.set_tie(player_action.ult_ttt_big),
        };
        if self.status == TicTacToeStatus::Tie {
            // game finished without direct winner
            // count for each player number of won squares; most squares won wins game
            let my_squares = self.status_map.count_player_cells(MonteCarloPlayer::Me);
            let opp_squares = self.status_map.count_player_cells(MonteCarloPlayer::Opp);
            self.status = match my_squares.cmp(&opp_squares) {
                Ordering::Greater => TicTacToeStatus::Player(MonteCarloPlayer::Me),
                Ordering::Less => TicTacToeStatus::Player(MonteCarloPlayer::Opp),
                Ordering::Equal => TicTacToeStatus::Tie,
            };
        } else {
            // player_action.ult_ttt_small points to next TicTacToe for next player to set new value.
            // if this TicTacToe status is not vacant (meaning there are no more cells to set), player can choose from all free cells
            self.next_action_square_is_specified = if self
                .status_map
                .get_cell_value(player_action.ult_ttt_small)
                .is_vacant()
            {
                Some(player_action.ult_ttt_small)
            } else {
                None
            };
        }
        self.status
    }
}

impl Display for UltTTT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const HEAD: &str = "┌─┬─┬─┐┌─┬─┬─┐┌─┬─┬─┐";
        const MIDDLE: &str = "├─┼─┼─┤├─┼─┼─┤├─┼─┼─┤";
        const FOOTER: &str = "└─┴─┴─┘└─┴─┴─┘└─┴─┴─┘";
        for v in 0..V {
            if v % Y == 0 {
                writeln!(f, "{}", HEAD)?;
            }
            for u in 0..U {
                if u % X == 0 {
                    write!(f, "│")?;
                }
                let cell_to_print = UltTTTPlayerAction::from_ext((u, v).into());
                write!(f, "{}│", self.get_cell_value(cell_to_print))?;
                if u == U - 1 {
                    writeln!(f)?;
                }
            }
            if v % Y < Y - 1 {
                writeln!(f, "{}", MIDDLE)?;
            } else {
                write!(f, "{}", FOOTER)?;
                if v < V - 1 {
                    writeln!(f)?;
                }
            }
        }
        Ok(())
    }
}

// solving UltTTT with new MCTS traits

// solving TicTacToe with new MCTS traits
pub struct UltTTTMCTSGame {}

impl MCTSGame for UltTTTMCTSGame {
    type State = UltTTT;
    type Move = UltTTTPlayerAction;
    type Player = MonteCarloPlayer;

    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a> {
        Box::new(IterUltTTT::new(
            state,
            state.current_player,
            state.game_turn,
        ))
    }

    fn apply_move(state: &Self::State, mv: &Self::Move) -> Self::State {
        let mut new_state = *state;
        // apply the move for current player
        new_state.execute_player_action(*mv, state.current_player);
        // set the next player
        new_state.next_player();
        // increment game turn
        new_state.increment_game_turn();
        new_state
    }

    fn evaluate(state: &Self::State) -> f32 {
        match state.status {
            TicTacToeStatus::Player(MonteCarloPlayer::Me) => 1.0,
            TicTacToeStatus::Player(MonteCarloPlayer::Opp) => 0.0,
            TicTacToeStatus::Tie => 0.5,
            TicTacToeStatus::Vacant => f32::NAN,
        }
    }

    fn is_terminal(state: &Self::State) -> bool {
        state.status.is_not_vacant()
    }
    fn current_player(state: &Self::State) -> MonteCarloPlayer {
        state.current_player
    }
    fn perspective_player() -> Self::Player {
        MonteCarloPlayer::Me
    }
}

#[cfg(test)]
mod tests {
    use my_lib::my_monte_carlo_tree_search::{
        DynamicC, MCTSAlgo, NoCache, StaticC, TurnBasedMCTS, WithCache,
    };
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
            let mut first_mcts_ult_ttt: TurnBasedMCTS<UltTTTMCTSGame, DynamicC, WithCache> =
                TurnBasedMCTS::new(WEIGHTING_FACTOR);
            let mut first_ult_ttt_game_data = UltTTT::new();
            first_ult_ttt_game_data.set_current_player(MonteCarloPlayer::Me);
            let mut first_time_out = TIME_OUT_FIRST_TURN;
            let mut second_mcts_ult_ttt: TurnBasedMCTS<UltTTTMCTSGame, StaticC, NoCache> =
                TurnBasedMCTS::new(WEIGHTING_FACTOR);
            let mut second_ult_ttt_game_data = UltTTT::new();
            second_ult_ttt_game_data.set_current_player(MonteCarloPlayer::Opp);
            let mut second_time_out = TIME_OUT_FIRST_TURN;

            let mut first = true;

            while !UltTTTMCTSGame::is_terminal(&first_ult_ttt_game_data) {
                if first {
                    let start = Instant::now();
                    first_mcts_ult_ttt.set_root(&first_ult_ttt_game_data);
                    let mut number_of_iterations = 0;
                    while start.elapsed() < first_time_out {
                        first_mcts_ult_ttt.iterate();
                        number_of_iterations += 1;
                    }
                    first_time_out = TIME_OUT_SUCCESSIVE_TURNS;
                    let selected_move = first_mcts_ult_ttt.select_move();
                    eprintln!(
                        "first : {} (number_of_iterations: {})",
                        selected_move.to_ext(),
                        number_of_iterations
                    );
                    first_ult_ttt_game_data =
                        UltTTTMCTSGame::apply_move(&first_ult_ttt_game_data, selected_move);
                    second_ult_ttt_game_data =
                        UltTTTMCTSGame::apply_move(&second_ult_ttt_game_data, selected_move);
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
                    let selected_move = second_mcts_ult_ttt.select_move();
                    eprintln!(
                        "second: {} (number_of_iterations: {})",
                        selected_move.to_ext(),
                        number_of_iterations
                    );
                    second_ult_ttt_game_data =
                        UltTTTMCTSGame::apply_move(&second_ult_ttt_game_data, selected_move);
                    first_ult_ttt_game_data =
                        UltTTTMCTSGame::apply_move(&first_ult_ttt_game_data, selected_move);
                    first = true;
                }
            }
            eprintln!("Game ended");
            eprintln!("{}", first_ult_ttt_game_data);
            match first_ult_ttt_game_data.status {
                TicTacToeStatus::Player(MonteCarloPlayer::Me) => {
                    eprintln!("first winner");
                }
                TicTacToeStatus::Player(MonteCarloPlayer::Opp) => {
                    eprintln!("second winner");
                }
                TicTacToeStatus::Tie => eprintln!("tie"),
                TicTacToeStatus::Vacant => {
                    eprintln!("vacant: Game ended without winner!?");
                    assert!(false, "vacant: Game ended without winner!?");
                }
            }
            wins += UltTTTMCTSGame::evaluate(&first_ult_ttt_game_data);
        }
        println!("{} wins out of {} matches.", wins, number_of_matches);
        //assert_eq!(wins, 25.0);
    }
}
