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
    pub fn new(ult_ttt_data: &'a UltTTT) -> Self {
        let mut result = IterUltTTT {
            ult_ttt_data,
            player_action: UltTTTPlayerAction::default(),
            next_action_square_is_specified: false,
            iter_finished: false,
        };
        match ult_ttt_data.next_action_constraint {
            NextActionConstraint::Init => {
                // Init is only possible, if me is starting player
                result.player_action.ult_ttt_big = MapPoint::<X, Y>::new(1, 1);
                result.player_action.ult_ttt_small = MapPoint::<X, Y>::new(0, 0);
                result.next_action_square_is_specified = true;
            }
            NextActionConstraint::None => {
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
            NextActionConstraint::MiniBoard(next_ult_ttt_big) => {
                result.player_action.ult_ttt_big = next_ult_ttt_big;
                result.player_action.ult_ttt_small = ult_ttt_data
                    .map
                    .get(next_ult_ttt_big)
                    .get_first_vacant_cell()
                    .unwrap()
                    .0;
                result.next_action_square_is_specified = true;
            }
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

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum NextActionConstraint {
    #[default]
    Init,
    None,
    MiniBoard(MapPoint<X, Y>),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct UltTTT {
    map: MyMap2D<TicTacToeGameData, X, Y>,
    status_map: TicTacToeGameData,
    next_action_constraint: NextActionConstraint,
    current_player: TwoPlayer,
}

impl UltTTT {
    pub fn new() -> Self {
        UltTTT {
            map: MyMap2D::new(),
            status_map: TicTacToeGameData::new(),
            next_action_constraint: NextActionConstraint::Init,
            current_player: TwoPlayer::Me,
        }
    }
    pub fn set_current_player(&mut self, player: TwoPlayer) {
        self.current_player = player;
    }
    pub fn next_player(&mut self) {
        self.current_player = self.current_player.next_player();
    }
    fn get_cell_value(&self, cell: UltTTTPlayerAction) -> TicTacToeStatus {
        self.map
            .get(cell.ult_ttt_big)
            .get_cell_value(cell.ult_ttt_small)
    }
    pub fn execute_player_action(&mut self, player_action: UltTTTPlayerAction, player: TwoPlayer) {
        self.map
            .get_mut(player_action.ult_ttt_big)
            .apply_player_move(player_action.ult_ttt_small, player);
        match self.map.get(player_action.ult_ttt_big).get_status() {
            TicTacToeStatus::Vacant => (),
            TicTacToeStatus::Player(winner) => {
                self.status_map
                    .apply_player_move(player_action.ult_ttt_big, winner);
            }
            TicTacToeStatus::Tie => self.status_map.set_tie(player_action.ult_ttt_big),
        };

        // player_action.ult_ttt_small points to next TicTacToe for next player to set new value.
        // if this TicTacToe status is not vacant (meaning there are no more cells to set), player can choose from all free cells
        self.next_action_constraint = if self
            .status_map
            .get_cell_value(player_action.ult_ttt_small)
            .is_vacant()
        {
            NextActionConstraint::MiniBoard(player_action.ult_ttt_small)
        } else {
            NextActionConstraint::None
        };
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

pub struct UltTTTMCTSGame {}

impl MCTSGame for UltTTTMCTSGame {
    type State = UltTTT;
    type Move = UltTTTPlayerAction;
    type Player = TwoPlayer;
    type Cache = NoGameCache<UltTTT, UltTTTPlayerAction>;

    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a> {
        Box::new(IterUltTTT::new(state))
    }

    fn apply_move(
        state: &Self::State,
        mv: &Self::Move,
        _game_cache: &mut Self::Cache,
    ) -> Self::State {
        let mut new_state = *state;
        // apply the move for current player
        new_state.execute_player_action(*mv, state.current_player);
        // set the next player
        new_state.next_player();
        new_state
    }

    fn evaluate(state: &Self::State, _game_cache: &mut Self::Cache) -> Option<f32> {
        let mut status = state.status_map.get_status();
        if status == TicTacToeStatus::Tie {
            // game finished without direct winner
            // count for each player number of won squares; most squares won wins game
            let my_squares = state.status_map.count_player_cells(TwoPlayer::Me);
            let opp_squares = state.status_map.count_player_cells(TwoPlayer::Opp);
            status = match my_squares.cmp(&opp_squares) {
                Ordering::Greater => TicTacToeStatus::Player(TwoPlayer::Me),
                Ordering::Less => TicTacToeStatus::Player(TwoPlayer::Opp),
                Ordering::Equal => TicTacToeStatus::Tie,
            };
        }
        match status {
            TicTacToeStatus::Player(TwoPlayer::Me) => Some(1.0),
            TicTacToeStatus::Player(TwoPlayer::Opp) => Some(0.0),
            TicTacToeStatus::Tie => Some(0.5),
            TicTacToeStatus::Vacant => None,
        }
    }
    fn current_player(state: &Self::State) -> TwoPlayer {
        state.current_player
    }
    fn perspective_player() -> Self::Player {
        TwoPlayer::Me
    }
}

pub struct UltTTTHeuristic {}

impl Heuristic<UltTTTMCTSGame> for UltTTTHeuristic {
    type Cache = NoHeuristicCache;

    fn evaluate_state(
        state: &<UltTTTMCTSGame as MCTSGame>::State,
        _game_cache: &mut <UltTTTMCTSGame as MCTSGame>::Cache,
        _heuristic_cache: &mut Self::Cache,
    ) -> f32 {
        match state.status_map.get_status() {
            TicTacToeStatus::Player(TwoPlayer::Me) => 1.0,
            TicTacToeStatus::Player(TwoPlayer::Opp) => 0.0,
            TicTacToeStatus::Tie => 0.5,
            TicTacToeStatus::Vacant => {
                // meta progress: wins on status_map
                let my_wins = state.status_map.count_player_cells(TwoPlayer::Me) as f32;
                let opp_wins = state.status_map.count_player_cells(TwoPlayer::Opp) as f32;
                // mini board threats
                let mut my_threats = 0.0;
                let mut opp_threats = 0.0;
                for (ult_ttt_big, _) in state.status_map.iter_map().filter(|(_, c)| c.is_vacant()) {
                    let (my, opp) = state.map.get(ult_ttt_big).get_threats();
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
        }
    }

    fn evaluate_move(
        _state: &<UltTTTMCTSGame as MCTSGame>::State,
        _mv: &<UltTTTMCTSGame as MCTSGame>::Move,
        _game_cache: &mut <UltTTTMCTSGame as MCTSGame>::Cache,
        _heuristic_cache: &mut Self::Cache,
    ) -> f32 {
        0.0
    }
}

pub type UltTTTSimulationPolicy = HeuristicCutoff<20>;

#[cfg(test)]
mod tests {
    type PWDefaultTTT = PWDefault<UltTTTMCTSGame>;
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
                PWDefaultTTT,
                UltTTTHeuristic,
                UltTTTSimulationPolicy,
            > = PlainMCTS::new(WEIGHTING_FACTOR);
            let mut first_ult_ttt_game_data = UltTTT::new();
            first_ult_ttt_game_data.set_current_player(TwoPlayer::Me);
            let mut first_time_out = TIME_OUT_FIRST_TURN;
            let mut second_mcts_ult_ttt: PlainMCTS<
                UltTTTMCTSGame,
                StaticC,
                NoUTCCache,
                ExpandAllTTT,
                NoHeuristic,
                DefaultSimulationPolicy,
            > = PlainMCTS::new(WEIGHTING_FACTOR);
            let mut second_ult_ttt_game_data = UltTTT::new();
            second_ult_ttt_game_data.set_current_player(TwoPlayer::Opp);
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
                    eprintln!(
                        "first : {} (number_of_iterations: {})",
                        selected_move.to_ext(),
                        number_of_iterations
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
                    eprintln!(
                        "second: {} (number_of_iterations: {})",
                        selected_move.to_ext(),
                        number_of_iterations
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
                TicTacToeStatus::Player(TwoPlayer::Me) => {
                    eprintln!("first winner");
                }
                TicTacToeStatus::Player(TwoPlayer::Opp) => {
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
