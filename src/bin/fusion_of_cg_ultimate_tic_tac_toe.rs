use crate::my_lib::my_map_point::MapPoint;
use crate::my_lib::my_map_two_dim::MyMap2D;
use crate::my_lib::my_monte_carlo_tree_search::MonteCarloGameData;
use crate::my_lib::my_monte_carlo_tree_search::MonteCarloGameDataUpdate;
use crate::my_lib::my_monte_carlo_tree_search::MonteCarloGameMode;
use crate::my_lib::my_monte_carlo_tree_search::MonteCarloPlayer;
use crate::my_lib::my_monte_carlo_tree_search::MonteCarloPlayerAction;
use crate::my_lib::my_monte_carlo_tree_search::MonteCarloTreeSearch;
use crate::my_lib::my_tic_tac_toe::TicTacToeGameData;
use crate::my_lib::my_tic_tac_toe::TicTacToeStatus;
use crate::my_lib::my_tic_tac_toe::X;
use crate::my_lib::my_tic_tac_toe::Y;
use std::cmp::Ordering;
use std::fmt::Write;
use std::io;
use std::time::Duration;
const U: usize = 9;
const V: usize = U;
struct IterUltTTT<'a> {
    ult_ttt_data: &'a UltTTT,
    player_action: UltTTTPlayerAction,
    next_action_square_is_specified: bool,
    iter_finished: bool,
}
impl<'a> IterUltTTT<'a> {
    fn new(ult_ttt_data: &'a UltTTT, player: MonteCarloPlayer, parent_game_turn: usize) -> Self {
        let mut result = IterUltTTT {
            ult_ttt_data,
            player_action: UltTTTPlayerAction::default(),
            next_action_square_is_specified: false,
            iter_finished: false,
        };
        if parent_game_turn == 0 && player == MonteCarloPlayer::Me {
            result.player_action.ult_ttt_big = MapPoint::<X, Y>::new(1, 1);
            result.player_action.ult_ttt_small = MapPoint::<X, Y>::new(0, 0);
            result.next_action_square_is_specified = true;
        } else if let Some(next_ult_ttt_big) = ult_ttt_data.next_action_square_is_specified {
            result.player_action.ult_ttt_big = next_ult_ttt_big;
            result.player_action.ult_ttt_small = ult_ttt_data
                .map
                .get(result.player_action.ult_ttt_big)
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
struct UltTTTPlayerAction {
    ult_ttt_big: MapPoint<X, Y>,
    ult_ttt_small: MapPoint<X, Y>,
}
impl UltTTTPlayerAction {
    fn from_ext(extern_coordinates: MapPoint<U, V>) -> UltTTTPlayerAction {
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
    fn to_ext(self) -> MapPoint<U, V> {
        MapPoint::<U, V>::new(
            self.ult_ttt_big.x() * X + self.ult_ttt_small.x(),
            self.ult_ttt_big.y() * Y + self.ult_ttt_small.y(),
        )
    }
    fn execute_action(&self) -> String {
        let mut action_commando_string = String::new();
        let action = self.to_ext();
        write!(action_commando_string, "{} {}", action.y(), action.x()).unwrap();
        println!("{}", action_commando_string);
        action_commando_string
    }
}
impl MonteCarloPlayerAction for UltTTTPlayerAction {
    fn downcast_self(player_action: &impl MonteCarloPlayerAction) -> &Self {
        match player_action.as_any().downcast_ref::<Self>() {
            Some(ult_ttt_pa) => ult_ttt_pa,
            None => panic!("player_action is not of type UltTTT_PlayerAction!"),
        }
    }
    fn iter_actions(
        game_data: &impl MonteCarloGameData,
        player: MonteCarloPlayer,
        parent_game_turn: usize,
    ) -> Box<dyn Iterator<Item = Self> + '_> {
        let game_data = UltTTT::downcast_self(game_data);
        Box::new(IterUltTTT::new(game_data, player, parent_game_turn))
    }
}
#[derive(Copy, Clone, PartialEq, Default)]
pub struct UltTTTGameDataUpdate {}
impl MonteCarloGameDataUpdate for UltTTTGameDataUpdate {
    fn downcast_self(_game_data_update: &impl MonteCarloGameDataUpdate) -> &Self {
        &UltTTTGameDataUpdate {}
    }
    fn iter_game_data_updates(
        _game_data: &impl MonteCarloGameData,
        _force_update: bool,
    ) -> Box<dyn Iterator<Item = Self> + '_> {
        Box::new(vec![].into_iter())
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
struct UltTTT {
    map: MyMap2D<TicTacToeGameData, X, Y>,
    status_map: TicTacToeGameData,
    status: TicTacToeStatus,
    next_action_square_is_specified: Option<MapPoint<X, Y>>,
}
impl UltTTT {
    fn new() -> Self {
        UltTTT {
            map: MyMap2D::new(),
            status_map: TicTacToeGameData::new(),
            status: TicTacToeStatus::Vacant,
            next_action_square_is_specified: None,
        }
    }
    fn set_last_opp_action(&mut self, opp_map_point: MapPoint<U, V>) -> TicTacToeStatus {
        let opp_action = UltTTTPlayerAction::from_ext(opp_map_point);
        self.execute_player_action(opp_action, MonteCarloPlayer::Opp)
    }
    fn execute_player_action(
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
            let my_squares = self.status_map.count_player_cells(MonteCarloPlayer::Me);
            let opp_squares = self.status_map.count_player_cells(MonteCarloPlayer::Opp);
            self.status = match my_squares.cmp(&opp_squares) {
                Ordering::Greater => TicTacToeStatus::Player(MonteCarloPlayer::Me),
                Ordering::Less => TicTacToeStatus::Player(MonteCarloPlayer::Opp),
                Ordering::Equal => TicTacToeStatus::Tie,
            };
        }
        self.next_action_square_is_specified =
            if self.status_map.get_cell_value(player_action.ult_ttt_small)
                == TicTacToeStatus::Vacant
            {
                Some(player_action.ult_ttt_small)
            } else {
                None
            };
        self.status
    }
}
impl MonteCarloGameData for UltTTT {
    fn downcast_self(game_data: &impl MonteCarloGameData) -> &Self {
        match game_data.as_any().downcast_ref::<Self>() {
            Some(ult_ttt) => ult_ttt,
            None => panic!("&game_data is not of type UltTTT!"),
        }
    }
    fn apply_my_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool {
        let my_action = *UltTTTPlayerAction::downcast_self(player_action);
        self.execute_player_action(my_action, MonteCarloPlayer::Me)
            .is_not_vacant()
            || self
                .status_map
                .get_cell_value(my_action.ult_ttt_big)
                .is_player()
    }
    fn apply_opp_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool {
        let opp_action = *UltTTTPlayerAction::downcast_self(player_action);
        self.execute_player_action(opp_action, MonteCarloPlayer::Opp)
            .is_not_vacant()
            || self
                .status_map
                .get_cell_value(opp_action.ult_ttt_big)
                .is_player()
    }
    fn simultaneous_player_actions_for_simultaneous_game_data_change(
        &mut self,
        _my_action: &impl MonteCarloPlayerAction,
        _opp_action: &impl MonteCarloPlayerAction,
    ) {
    }
    fn apply_game_data_update(
        &mut self,
        _game_data_update: &impl MonteCarloGameDataUpdate,
        _check_update_consistency: bool,
    ) -> bool {
        false
    }
    fn is_game_data_update_required(&self, _force_update: bool) -> bool {
        false
    }
    fn calc_heuristic(&self) -> f32 {
        self.status_map.calc_heuristic_() * 10.0
            + self
                .status_map
                .iter_map()
                .map(|(_, s)| match s {
                    TicTacToeStatus::Player(player) => match player {
                        MonteCarloPlayer::Me => 1.0,
                        MonteCarloPlayer::Opp => -1.0,
                    },
                    _ => 0.0,
                })
                .sum::<f32>()
    }
    fn check_game_ending(&self, _game_turn: usize) -> bool {
        self.status.is_not_vacant()
    }
    fn game_winner(&self, _game_turn: usize) -> Option<MonteCarloPlayer> {
        match self.status {
            TicTacToeStatus::Player(player) => Some(player),
            _ => None,
        }
    }
    fn check_consistency_of_game_data_during_init_root(
        &mut self,
        _current_game_state: &Self,
        _played_turns: usize,
    ) -> bool {
        true
    }
    fn check_consistency_of_game_data_update(
        &mut self,
        _current_game_state: &Self,
        _game_data_update: &impl MonteCarloGameDataUpdate,
        _played_turns: usize,
    ) -> bool {
        true
    }
    fn check_consistency_of_action_result(
        &mut self,
        _current_game_state: Self,
        _my_action: &impl MonteCarloPlayerAction,
        _opp_action: &impl MonteCarloPlayerAction,
        _played_turns: usize,
        _apply_player_actions_to_game_data: bool,
    ) -> bool {
        true
    }
}
macro_rules! parse_input {
    ($ x : expr_2021 , $ t : ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}
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
            if opponent_row >= 0 {
                starting_player = MonteCarloPlayer::Opp;
                turn_counter += 1;
                let opp_action =
                    MapPoint::<U, V>::new(opponent_col as usize, opponent_row as usize);
                game_data.set_last_opp_action(opp_action);
            }
        } else {
            let opp_action = MapPoint::<U, V>::new(opponent_col as usize, opponent_row as usize);
            game_data.set_last_opp_action(opp_action);
        }
        let start = mcts.init_root(&game_data, starting_player);
        mcts.expand_tree(start);
        let (_my_game_data, my_action) = mcts.choose_and_execute_actions();
        my_action.execute_action();
        turn_counter += 2;
    }
}
pub mod my_lib {
    pub mod my_map_point {
        use std::cmp::Ordering;
        #[derive(Debug, Eq, PartialEq, Copy, Clone, Default, Hash)]
        pub struct MapPoint<const X: usize, const Y: usize> {
            x: usize,
            y: usize,
        }
        impl<const X: usize, const Y: usize> PartialOrd for MapPoint<X, Y> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        impl<const X: usize, const Y: usize> Ord for MapPoint<X, Y> {
            fn cmp(&self, other: &Self) -> Ordering {
                match self.y.cmp(&other.y) {
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => self.x.cmp(&other.x),
                }
            }
        }
        impl<const X: usize, const Y: usize> From<(usize, usize)> for MapPoint<X, Y> {
            fn from(value: (usize, usize)) -> Self {
                MapPoint::<X, Y>::new(value.0, value.1)
            }
        }
        impl<const X: usize, const Y: usize> MapPoint<X, Y> {
            pub fn new(x: usize, y: usize) -> Self {
                if X == 0 {
                    panic!("line {}, minimum size of dimension X is 1", line!());
                }
                if Y == 0 {
                    panic!("line {}, minimum size of dimension Y is 1", line!());
                }
                let result = MapPoint { x, y };
                if !result.is_in_map() {
                    panic!("line {}, coordinates are out of range", line!());
                }
                result
            }
            pub fn x(&self) -> usize {
                self.x
            }
            pub fn y(&self) -> usize {
                self.y
            }
            pub fn is_in_map(&self) -> bool {
                self.x < X && self.y < Y
            }
            pub fn forward_x(&self) -> Option<MapPoint<X, Y>> {
                let mut result = *self;
                result.x += 1;
                if result.x == X {
                    result.y += 1;
                    if result.y == Y {
                        return None;
                    }
                    result.x = 0;
                }
                Some(result)
            }
        }
    }
    pub mod my_map_two_dim {
        use super::my_map_point::MapPoint;
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
        pub struct MyMap2D<T, const X: usize, const Y: usize> {
            items: [[T; X]; Y],
        }
        impl<T: Copy + Clone + Default, const X: usize, const Y: usize> MyMap2D<T, X, Y> {
            pub fn new() -> Self {
                if X == 0 {
                    panic!("line {}, minimum one column", line!());
                }
                if Y == 0 {
                    panic!("line {}, minimum one row", line!());
                }
                Self {
                    items: [[T::default(); X]; Y],
                }
            }
            pub fn init(init_element: T) -> Self {
                if X == 0 {
                    panic!("line {}, minimum one column", line!());
                }
                if Y == 0 {
                    panic!("line {}, minimum one row", line!());
                }
                Self {
                    items: [[init_element; X]; Y],
                }
            }
            pub fn get(&self, coordinates: MapPoint<X, Y>) -> &T {
                &self.items[coordinates.y()][coordinates.x()]
            }
            pub fn get_mut(&mut self, coordinates: MapPoint<X, Y>) -> &mut T {
                &mut self.items[coordinates.y()][coordinates.x()]
            }
            pub fn swap_value(&mut self, coordinates: MapPoint<X, Y>, value: T) -> T {
                let old_value = self.items[coordinates.y()][coordinates.x()];
                self.items[coordinates.y()][coordinates.x()] = value;
                old_value
            }
            pub fn get_row(&self, row: usize) -> &[T] {
                if row >= Y {
                    panic!("line {}, row out of range", line!());
                }
                &self.items[row][..]
            }
            pub fn iter(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
                self.items.iter().enumerate().flat_map(|(y, row)| {
                    row.iter()
                        .enumerate()
                        .map(move |(x, column)| (MapPoint::<X, Y>::new(x, y), column))
                })
            }
            pub fn iter_mut(&mut self) -> impl Iterator<Item = (MapPoint<X, Y>, &mut T)> {
                self.items.iter_mut().enumerate().flat_map(|(y, row)| {
                    row.iter_mut()
                        .enumerate()
                        .map(move |(x, column)| (MapPoint::<X, Y>::new(x, y), column))
                })
            }
            pub fn iter_row(&self, r: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
                self.get_row(r)
                    .iter()
                    .enumerate()
                    .map(move |(x, column)| (MapPoint::new(x, r), column))
            }
            pub fn iter_column(&self, c: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
                if c >= X {
                    panic!("line {}, column index is out of range", line!());
                }
                self.items.iter().enumerate().flat_map(move |(y, row)| {
                    row.iter()
                        .enumerate()
                        .filter(move |(x, _)| *x == c)
                        .map(move |(x, column)| (MapPoint::new(x, y), column))
                })
            }
        }
        impl<T: Copy + Clone + Default, const X: usize, const Y: usize> Default for MyMap2D<T, X, Y> {
            fn default() -> Self {
                Self::new()
            }
        }
    }
    pub mod my_monte_carlo_tree_search {
        mod misc_types {
            #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
            pub enum MonteCarloPlayer {
                Me,
                Opp,
            }
            impl MonteCarloPlayer {
                pub fn next_player(&self) -> Self {
                    match self {
                        MonteCarloPlayer::Me => MonteCarloPlayer::Opp,
                        MonteCarloPlayer::Opp => MonteCarloPlayer::Me,
                    }
                }
            }
            #[derive(Copy, Clone, PartialEq)]
            pub enum MonteCarloNodeType {
                GameDataUpdate,
                ActionResult,
            }
            #[derive(Copy, Clone, PartialEq)]
            pub enum MonteCarloGameMode {
                SameTurnParallel,
                ByTurns,
            }
        }
        mod node {
            use super::MonteCarloGameData;
            use super::MonteCarloGameDataUpdate;
            use super::MonteCarloGameMode;
            use super::MonteCarloNodeType;
            use super::MonteCarloPlayer;
            use super::MonteCarloPlayerAction;
            #[derive(PartialEq, Clone, Copy)]
            pub struct MonteCarloNode<
                G: MonteCarloGameData,
                A: MonteCarloPlayerAction,
                U: MonteCarloGameDataUpdate,
            > {
                pub game_data: G,
                pub player_action: A,
                pub game_data_update: U,
                pub node_type: MonteCarloNodeType,
                pub next_node: MonteCarloNodeType,
                pub player: MonteCarloPlayer,
                pub game_turn: usize,
                pub heuristic: f32,
                pub wins: f32,
                pub samples: f32,
                pub parent_samples: f32,
                pub exploitation_score: f32,
                pub exploration_score: f32,
                pub heuristic_score: f32,
                pub total_score: f32,
                pub game_end_node: bool,
            }
            impl<G: MonteCarloGameData, A: MonteCarloPlayerAction, U: MonteCarloGameDataUpdate>
                Default for MonteCarloNode<G, A, U>
            {
                fn default() -> Self {
                    Self::new()
                }
            }
            impl<G: MonteCarloGameData, A: MonteCarloPlayerAction, U: MonteCarloGameDataUpdate>
                MonteCarloNode<G, A, U>
            {
                pub fn new() -> Self {
                    MonteCarloNode {
                        game_data: G::default(),
                        player_action: A::default(),
                        game_data_update: U::default(),
                        node_type: MonteCarloNodeType::ActionResult,
                        next_node: MonteCarloNodeType::ActionResult,
                        player: MonteCarloPlayer::Me,
                        game_turn: 0,
                        heuristic: 0.0,
                        wins: 0.0,
                        samples: f32::NAN,
                        parent_samples: 0.0,
                        exploitation_score: 0.0,
                        exploration_score: 0.0,
                        heuristic_score: 0.0,
                        total_score: 0.0,
                        game_end_node: false,
                    }
                }
                pub fn new_player_action_child(&self, player_action: A) -> Self {
                    let mut new_child = Self::new();
                    new_child.player_action = player_action;
                    new_child.game_turn = self.game_turn;
                    new_child.player = self.player;
                    new_child
                }
                pub fn new_game_data_update_child(&self, game_data_update: U) -> Self {
                    let mut new_child = Self::new();
                    new_child.game_data_update = game_data_update;
                    new_child.game_turn = self.game_turn;
                    new_child.player = self.player;
                    new_child.node_type = MonteCarloNodeType::GameDataUpdate;
                    new_child
                }
                pub fn calc_heuristic(&mut self, use_heuristic_score: bool) {
                    if use_heuristic_score {
                        self.heuristic = self.game_data.calc_heuristic();
                    }
                }
                pub fn calc_node_score(&mut self, parent_samples: f32, weighting_factor: f32) {
                    if parent_samples != self.parent_samples {
                        self.update_exploration_score(parent_samples, weighting_factor);
                    }
                    self.total_score = match self.player {
                        MonteCarloPlayer::Me => {
                            self.exploitation_score + self.exploration_score - self.heuristic_score
                        }
                        MonteCarloPlayer::Opp => {
                            self.exploitation_score + self.exploration_score + self.heuristic_score
                        }
                    };
                }
                pub fn check_game_turn(&mut self, game_mode: MonteCarloGameMode) {
                    match game_mode {
                        MonteCarloGameMode::SameTurnParallel => {
                            if self.player == MonteCarloPlayer::Opp {
                                self.game_turn += 1;
                            }
                        }
                        MonteCarloGameMode::ByTurns => self.game_turn += 1,
                    }
                }
                pub fn set_next_node(&mut self, force_update: bool) {
                    if !self.game_end_node {
                        self.next_node =
                            if self.game_data.is_game_data_update_required(force_update) {
                                MonteCarloNodeType::GameDataUpdate
                            } else {
                                MonteCarloNodeType::ActionResult
                            };
                    }
                }
                pub fn apply_action(
                    &mut self,
                    parent_game_data: &G,
                    parent_action: &A,
                    game_mode: MonteCarloGameMode,
                    max_number_of_turns: usize,
                    use_heuristic_score: bool,
                ) {
                    self.game_data = *parent_game_data;
                    let mut score_event = self.apply_player_action();
                    self.player = self.player.next_player();
                    self.check_game_turn(game_mode);
                    match game_mode {
                        MonteCarloGameMode::SameTurnParallel => {
                            if self.player == MonteCarloPlayer::Me {
                                if self.check_game_ending(max_number_of_turns) {
                                    self.calc_heuristic(use_heuristic_score);
                                    return;
                                }
                                self.game_data
                                    .simultaneous_player_actions_for_simultaneous_game_data_change(
                                        parent_action,
                                        &self.player_action,
                                    );
                            }
                        }
                        MonteCarloGameMode::ByTurns => {
                            score_event =
                                self.check_game_ending(max_number_of_turns) || score_event;
                        }
                    }
                    if score_event {
                        self.calc_heuristic(use_heuristic_score);
                    }
                }
                pub fn apply_game_data_update(
                    &mut self,
                    parent_game_data: &G,
                    check_update_consistency: bool,
                ) -> bool {
                    self.game_data = *parent_game_data;
                    self.game_data
                        .apply_game_data_update(&self.game_data_update, check_update_consistency)
                }
                pub fn apply_player_action(&mut self) -> bool {
                    match self.player {
                        MonteCarloPlayer::Me => self.game_data.apply_my_action(&self.player_action),
                        MonteCarloPlayer::Opp => {
                            self.game_data.apply_opp_action(&self.player_action)
                        }
                    }
                }
                pub fn check_game_ending(&mut self, max_number_of_turns: usize) -> bool {
                    self.game_end_node = self.game_turn == max_number_of_turns
                        || self.game_data.check_game_ending(self.game_turn);
                    self.game_end_node
                }
                pub fn calc_simulation_score(&self) -> f32 {
                    match self.game_data.game_winner(self.game_turn) {
                        Some(player) => match player {
                            MonteCarloPlayer::Me => 1.0,
                            MonteCarloPlayer::Opp => 0.0,
                        },
                        None => 0.5,
                    }
                }
                pub fn score_simulation_result(
                    &mut self,
                    simulation_score: f32,
                    samples: f32,
                    use_heuristic_score: bool,
                ) {
                    self.wins += simulation_score;
                    self.samples += samples;
                    self.exploitation_score = match self.player {
                        MonteCarloPlayer::Me => 1.0 - self.wins / self.samples,
                        MonteCarloPlayer::Opp => self.wins / self.samples,
                    };
                    if use_heuristic_score {
                        self.heuristic_score = match self.player {
                            MonteCarloPlayer::Me => -self.heuristic / self.samples,
                            MonteCarloPlayer::Opp => self.heuristic / self.samples,
                        };
                    }
                }
                pub fn update_exploration_score(
                    &mut self,
                    parent_samples: f32,
                    weighting_factor: f32,
                ) {
                    self.parent_samples = parent_samples;
                    self.exploration_score =
                        weighting_factor * (self.parent_samples.log10() / self.samples).sqrt();
                }
                pub fn update_consistent_node_during_init_phase(
                    &mut self,
                    current_game_state: &G,
                    played_turns: usize,
                    force_update: bool,
                ) -> bool {
                    if !force_update
                        && !self
                            .game_data
                            .check_consistency_of_game_data_during_init_root(
                                current_game_state,
                                played_turns,
                            )
                    {
                        return false;
                    }
                    self.game_data == *current_game_state
                }
            }
        }
        mod traits {
            use super::MonteCarloPlayer;
            use std::any::Any;
            use std::hash::Hash;
            pub trait MonteCarloPlayerAction: Copy + Clone + PartialEq + Default + 'static {
                fn as_any(&self) -> &dyn Any {
                    self
                }
                fn downcast_self(player_action: &impl MonteCarloPlayerAction) -> &Self;
                fn iter_actions(
                    game_data: &impl MonteCarloGameData,
                    player: MonteCarloPlayer,
                    parent_game_turn: usize,
                ) -> Box<dyn Iterator<Item = Self> + '_>;
            }
            pub trait MonteCarloGameDataUpdate:
                Copy + Clone + PartialEq + Default + 'static
            {
                fn as_any(&self) -> &dyn Any {
                    self
                }
                fn downcast_self(game_data_update: &impl MonteCarloGameDataUpdate) -> &Self;
                fn iter_game_data_updates(
                    game_data: &impl MonteCarloGameData,
                    force_update: bool,
                ) -> Box<dyn Iterator<Item = Self> + '_>;
            }
            pub trait MonteCarloGameData:
                Copy + Clone + PartialEq + Eq + Hash + Default + 'static
            {
                fn as_any(&self) -> &dyn Any {
                    self
                }
                fn downcast_self(game_data: &impl MonteCarloGameData) -> &Self;
                fn apply_my_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool;
                fn apply_opp_action(&mut self, player_action: &impl MonteCarloPlayerAction)
                -> bool;
                fn simultaneous_player_actions_for_simultaneous_game_data_change(
                    &mut self,
                    my_action: &impl MonteCarloPlayerAction,
                    opp_action: &impl MonteCarloPlayerAction,
                );
                fn is_game_data_update_required(&self, force_update: bool) -> bool;
                fn apply_game_data_update(
                    &mut self,
                    game_data_update: &impl MonteCarloGameDataUpdate,
                    check_update_consistency: bool,
                ) -> bool;
                fn calc_heuristic(&self) -> f32;
                fn check_game_ending(&self, game_turn: usize) -> bool;
                fn game_winner(&self, game_turn: usize) -> Option<MonteCarloPlayer>;
                fn check_consistency_of_game_data_during_init_root(
                    &mut self,
                    current_game_state: &Self,
                    played_turns: usize,
                ) -> bool;
                fn check_consistency_of_game_data_update(
                    &mut self,
                    current_game_state: &Self,
                    game_data_update: &impl MonteCarloGameDataUpdate,
                    played_turns: usize,
                ) -> bool;
                fn check_consistency_of_action_result(
                    &mut self,
                    current_game_state: Self,
                    my_action: &impl MonteCarloPlayerAction,
                    opp_action: &impl MonteCarloPlayerAction,
                    played_turns: usize,
                    apply_player_actions_to_game_data: bool,
                ) -> bool;
            }
        }
        mod tree_search {
            use super::super::my_tree::TreeNode;
            use super::MonteCarloGameData;
            use super::MonteCarloGameDataUpdate;
            use super::MonteCarloGameMode;
            use super::MonteCarloNode;
            use super::MonteCarloNodeType;
            use super::MonteCarloPlayer;
            use super::MonteCarloPlayerAction;
            use rand::prelude::*;
            use std::cmp::Ordering;
            use std::collections::HashMap;
            use std::rc::Rc;
            use std::rc::Weak;
            use std::time::Duration;
            use std::time::Instant;
            pub struct MonteCarloTreeSearch<
                G: MonteCarloGameData,
                A: MonteCarloPlayerAction,
                U: MonteCarloGameDataUpdate,
            > {
                tree_root: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
                game_mode: MonteCarloGameMode,
                starting_player: MonteCarloPlayer,
                played_turns: usize,
                max_number_of_turns: usize,
                force_update: bool,
                first_turn: bool,
                time_out_first_turn: Duration,
                time_out_successive_turns: Duration,
                weighting_factor: f32,
                use_heuristic_score: bool,
                use_caching: bool,
                #[allow(clippy::type_complexity)]
                node_cache:
                    HashMap<(G, MonteCarloPlayer, usize), Weak<TreeNode<MonteCarloNode<G, A, U>>>>,
                cache_events: usize,
                debug: bool,
            }
            impl<G: MonteCarloGameData, A: MonteCarloPlayerAction, U: MonteCarloGameDataUpdate>
                MonteCarloTreeSearch<G, A, U>
            {
                #[allow(clippy::too_many_arguments)]
                pub fn new(
                    game_mode: MonteCarloGameMode,
                    max_number_of_turns: usize,
                    force_update: bool,
                    time_out_first_turn: Duration,
                    time_out_successive_turns: Duration,
                    weighting_factor: f32,
                    use_heuristic_score: bool,
                    use_caching: bool,
                    debug: bool,
                ) -> Self {
                    MonteCarloTreeSearch {
                        tree_root: TreeNode::seed_root(MonteCarloNode::<G, A, U>::new(), 0),
                        game_mode,
                        starting_player: MonteCarloPlayer::Me,
                        played_turns: 0,
                        max_number_of_turns,
                        force_update,
                        first_turn: true,
                        time_out_first_turn,
                        time_out_successive_turns,
                        weighting_factor,
                        use_heuristic_score,
                        use_caching,
                        node_cache: HashMap::new(),
                        cache_events: 0,
                        debug,
                    }
                }
                pub fn init_root(
                    &mut self,
                    game_data: &G,
                    starting_player: MonteCarloPlayer,
                ) -> Instant {
                    let start = Instant::now();
                    if self.first_turn {
                        self.starting_player = starting_player;
                        self.tree_root.get_mut_value().game_data = *game_data;
                        self.tree_root.get_mut_value().samples = 0.0;
                        if self.game_mode == MonteCarloGameMode::ByTurns
                            && self.starting_player == MonteCarloPlayer::Opp
                        {
                            self.played_turns = 1;
                            self.tree_root.get_mut_value().game_turn = 1;
                            self.tree_root.get_mut_value().player = MonteCarloPlayer::Me;
                        } else {
                            self.tree_root.get_mut_value().node_type =
                                MonteCarloNodeType::GameDataUpdate;
                            self.tree_root.get_mut_value().player = starting_player;
                        }
                    } else {
                        let (search_turn, end_level) = match self.game_mode {
                            MonteCarloGameMode::SameTurnParallel => (self.played_turns, Some(3)),
                            MonteCarloGameMode::ByTurns => (self.played_turns + 1, Some(2)),
                        };
                        match self
                            .tree_root
                            .iter_level_order_traversal_with_borders(1, end_level)
                            .find(|(n, _)| {
                                let mut n_value = n.get_mut_value();
                                n_value.game_turn == search_turn
                                    && n_value.next_node == MonteCarloNodeType::ActionResult
                                    && n_value.player == MonteCarloPlayer::Me
                                    && n_value.update_consistent_node_during_init_phase(
                                        game_data,
                                        self.played_turns,
                                        self.force_update,
                                    )
                            }) {
                            Some((new_root, _)) => {
                                self.tree_root = new_root;
                                if self.tree_root.get_value().samples.is_nan() {
                                    self.tree_root.get_mut_value().samples = 0.0;
                                }
                            }
                            None => {
                                if self.debug {
                                    eprintln!(
                                        "Current game state not found in tree. Reinitialize tree after {} played turns",
                                        self.played_turns
                                    );
                                }
                                self.tree_root =
                                    TreeNode::seed_root(MonteCarloNode::<G, A, U>::new(), 0);
                                self.tree_root.get_mut_value().game_data = *game_data;
                                self.tree_root.get_mut_value().samples = 0.0;
                                self.tree_root.get_mut_value().player = MonteCarloPlayer::Me;
                                self.tree_root.get_mut_value().game_turn = search_turn;
                            }
                        }
                    }
                    start
                }
                pub fn expand_tree(&mut self, start: Instant) {
                    let time_out = if self.first_turn {
                        self.first_turn = false;
                        self.time_out_first_turn
                    } else {
                        self.time_out_successive_turns
                    };
                    let current_cache_events = if self.use_caching {
                        self.node_cache.retain(|_, v| v.weak_count() > 0);
                        self.cache_events
                    } else {
                        0
                    };
                    let mut counter = 0;
                    while start.elapsed() < time_out && !self.one_cycle(&start, time_out) {
                        counter += 1;
                    }
                    if self.debug {
                        eprintln!("number of expand cycles: {}", counter);
                        if self.use_caching {
                            eprintln!(
                                "number of cache events (current expansion / total): {}/{}",
                                self.cache_events - current_cache_events,
                                self.cache_events
                            );
                        }
                    }
                }
                pub fn choose_and_execute_actions(&mut self) -> (G, A) {
                    let child = self
                        .tree_root
                        .iter_children()
                        .max_by(|x, y| {
                            match x
                                .get_value()
                                .exploitation_score
                                .partial_cmp(&y.get_value().exploitation_score)
                                .unwrap()
                            {
                                Ordering::Greater => Ordering::Greater,
                                Ordering::Less => Ordering::Less,
                                Ordering::Equal => x
                                    .get_value()
                                    .samples
                                    .partial_cmp(&y.get_value().samples)
                                    .unwrap(),
                            }
                        })
                        .unwrap();
                    self.played_turns = child.get_value().game_turn;
                    self.tree_root = child.clone();
                    let result = (child.get_value().game_data, child.get_value().player_action);
                    result
                }
                fn one_cycle(&mut self, start: &Instant, time_out: Duration) -> bool {
                    let mut start_node = self.tree_root.clone();
                    loop {
                        match self.selection(start, time_out, start_node) {
                            Some(selection_node) => match self.expansion(selection_node) {
                                Ok(child_node) => {
                                    if let Some(simulation_score) =
                                        self.simulation(child_node.clone(), start, time_out)
                                    {
                                        self.propagation(child_node, simulation_score)
                                    }
                                }
                                Err(parent_with_cached_child) => {
                                    start_node = parent_with_cached_child;
                                    continue;
                                }
                            },
                            None => return true,
                        }
                        break;
                    }
                    false
                }
                fn selection(
                    &self,
                    start: &Instant,
                    time_out: Duration,
                    mut selection_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
                ) -> Option<Rc<TreeNode<MonteCarloNode<G, A, U>>>> {
                    let mut rng = thread_rng();
                    while !selection_node.is_leave() {
                        if start.elapsed() >= time_out {
                            return None;
                        }
                        if self.remove_inconsistent_children(selection_node.clone()) {
                            return Some(selection_node);
                        }
                        if let Some(child_without_samples) = selection_node
                            .iter_children()
                            .filter(|c| c.get_value().samples.is_nan())
                            .choose(&mut rng)
                        {
                            return Some(child_without_samples);
                        }
                        selection_node.iter_children().for_each(|c| {
                            c.get_mut_value().calc_node_score(
                                selection_node.get_value().samples,
                                self.weighting_factor,
                            )
                        });
                        let selected_child = selection_node.iter_children().max_by(|a, b| {
                            a.get_value()
                                .total_score
                                .partial_cmp(&b.get_value().total_score)
                                .unwrap()
                        });
                        selection_node = match selected_child {
                            Some(child) => {
                                if self.force_update {
                                    child.clone()
                                } else {
                                    let node_type = child.get_value().node_type;
                                    match node_type {
                                        MonteCarloNodeType::ActionResult => {
                                            let child_action = child.get_value().player_action;
                                            let apply_player_actions_to_game_data = match self
                                                .game_mode
                                            {
                                                MonteCarloGameMode::SameTurnParallel => {
                                                    child.get_value().player == MonteCarloPlayer::Me
                                                }
                                                MonteCarloGameMode::ByTurns => true,
                                            };
                                            let child_game_data_changed = child
                                                .get_mut_value()
                                                .game_data
                                                .check_consistency_of_action_result(
                                                    selection_node.get_value().game_data,
                                                    &selection_node.get_value().player_action,
                                                    &child_action,
                                                    self.played_turns,
                                                    apply_player_actions_to_game_data,
                                                );
                                            if child_game_data_changed
                                                && child.get_value().next_node
                                                    == MonteCarloNodeType::GameDataUpdate
                                                && child.is_leave()
                                            {
                                                child
                                                    .get_mut_value()
                                                    .set_next_node(self.force_update);
                                            }
                                            child.clone()
                                        }
                                        MonteCarloNodeType::GameDataUpdate => child.clone(),
                                    }
                                }
                            }
                            None => panic!("selection should always find a child!"),
                        };
                    }
                    Some(selection_node)
                }
                #[allow(clippy::type_complexity)]
                fn expansion(
                    &mut self,
                    expansion_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
                ) -> Result<
                    Rc<TreeNode<MonteCarloNode<G, A, U>>>,
                    Rc<TreeNode<MonteCarloNode<G, A, U>>>,
                > {
                    if expansion_node.get_value().game_end_node
                        || expansion_node.get_value().samples.is_nan()
                    {
                        return Ok(expansion_node);
                    }
                    let mut found_cached_game_state = false;
                    let next_node = expansion_node.get_value().next_node;
                    match next_node {
                        MonteCarloNodeType::GameDataUpdate => {
                            for game_data_update in U::iter_game_data_updates(
                                &expansion_node.get_value().game_data,
                                self.force_update,
                            ) {
                                let mut new_game_data_update_node = expansion_node
                                    .get_value()
                                    .new_game_data_update_child(game_data_update);
                                if new_game_data_update_node.apply_game_data_update(
                                    &expansion_node.get_value().game_data,
                                    !self.force_update,
                                ) {
                                    new_game_data_update_node.set_next_node(self.force_update);
                                    expansion_node.add_child(new_game_data_update_node, 0);
                                }
                            }
                        }
                        MonteCarloNodeType::ActionResult => {
                            for player_action in A::iter_actions(
                                &expansion_node.get_value().game_data,
                                expansion_node.get_value().player,
                                expansion_node.get_value().game_turn,
                            ) {
                                let mut new_player_action_node = expansion_node
                                    .get_value()
                                    .new_player_action_child(player_action);
                                new_player_action_node.apply_action(
                                    &expansion_node.get_value().game_data,
                                    &expansion_node.get_value().player_action,
                                    self.game_mode,
                                    self.max_number_of_turns,
                                    self.use_heuristic_score,
                                );
                                new_player_action_node.set_next_node(self.force_update);
                                if self.use_caching {
                                    let cache_key = (
                                        new_player_action_node.game_data,
                                        new_player_action_node.player,
                                        new_player_action_node.game_turn,
                                    );
                                    if let Some(cached_child) = self.node_cache.get(&cache_key) {
                                        if let Some(child) = cached_child.upgrade() {
                                            expansion_node.link_child_to_parent(child);
                                            found_cached_game_state = true;
                                            self.cache_events += 1;
                                            continue;
                                        }
                                    }
                                    let child = expansion_node.add_child(new_player_action_node, 0);
                                    if self.game_mode == MonteCarloGameMode::ByTurns
                                        || (self.game_mode == MonteCarloGameMode::SameTurnParallel
                                            && new_player_action_node.player
                                                == MonteCarloPlayer::Me)
                                    {
                                        self.node_cache.insert(cache_key, Rc::downgrade(&child));
                                    }
                                } else {
                                    expansion_node.add_child(new_player_action_node, 0);
                                }
                            }
                        }
                    }
                    if found_cached_game_state {
                        return Err(expansion_node);
                    }
                    Ok(expansion_node.get_child(0).unwrap())
                }
                fn simulation(
                    &self,
                    simulation_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
                    start: &Instant,
                    time_out: Duration,
                ) -> Option<f32> {
                    if simulation_node.get_value().game_end_node {
                        Some(simulation_node.get_value().calc_simulation_score())
                    } else {
                        let mut rng = thread_rng();
                        let mut simulation = *simulation_node.get_value();
                        while !simulation.game_end_node {
                            if start.elapsed() >= time_out {
                                return None;
                            }
                            match simulation.next_node {
                                MonteCarloNodeType::GameDataUpdate => {
                                    let parent_game_data = simulation.game_data;
                                    let game_data_update = U::iter_game_data_updates(
                                        &simulation.game_data,
                                        self.force_update,
                                    )
                                    .choose(&mut rng)
                                    .unwrap();
                                    simulation =
                                        simulation.new_game_data_update_child(game_data_update);
                                    simulation.apply_game_data_update(&parent_game_data, false);
                                    simulation.set_next_node(self.force_update);
                                }
                                MonteCarloNodeType::ActionResult => {
                                    let parent_game_data = simulation.game_data;
                                    let parent_action = simulation.player_action;
                                    let player_action = A::iter_actions(
                                        &simulation.game_data,
                                        simulation.player,
                                        simulation.game_turn,
                                    )
                                    .choose(&mut rng)
                                    .unwrap();
                                    simulation = simulation.new_player_action_child(player_action);
                                    simulation.apply_action(
                                        &parent_game_data,
                                        &parent_action,
                                        self.game_mode,
                                        self.max_number_of_turns,
                                        self.use_heuristic_score,
                                    );
                                    simulation.set_next_node(self.force_update);
                                }
                            }
                        }
                        Some(simulation.calc_simulation_score())
                    }
                }
                fn propagation(
                    &self,
                    start_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
                    mut simulation_score: f32,
                ) {
                    if start_node.get_value().samples.is_nan() {
                        start_node.get_mut_value().samples = 0.0;
                    }
                    for nodes in start_node.iter_back_track() {
                        for node in nodes.iter() {
                            if node.get_value().next_node == MonteCarloNodeType::GameDataUpdate
                                && node.len_children() > 0
                            {
                                let num_children = node.len_children() as f32;
                                simulation_score /= num_children;
                            }
                            node.get_mut_value().score_simulation_result(
                                simulation_score,
                                1.0,
                                self.use_heuristic_score,
                            );
                        }
                    }
                }
                fn reverse_propagation(
                    &self,
                    start_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
                    mut wins: f32,
                    mut samples: f32,
                ) {
                    start_node.get_mut_value().score_simulation_result(
                        wins,
                        samples,
                        self.use_heuristic_score,
                    );
                    for nodes in start_node.iter_back_track().skip(1) {
                        for node in nodes.iter() {
                            if node.get_value().next_node == MonteCarloNodeType::GameDataUpdate {
                                let num_children = node.len_children() as f32;
                                wins /= num_children;
                                samples /= num_children;
                            }
                            node.get_mut_value().score_simulation_result(
                                wins,
                                samples,
                                self.use_heuristic_score,
                            );
                        }
                    }
                }
                fn remove_inconsistent_children(
                    &self,
                    selection_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
                ) -> bool {
                    if self.force_update
                        || selection_node.get_value().next_node == MonteCarloNodeType::ActionResult
                        || selection_node.len_children() == 1
                    {
                        return false;
                    }
                    let n_children = selection_node.len_children() as f32;
                    let mut index = 0;
                    let mut samples = 0.0;
                    let mut wins = 0.0;
                    let mut inconsistency_detected = false;
                    while index < selection_node.len_children() {
                        let child = selection_node.get_child(index).unwrap();
                        if !child.get_value().samples.is_nan() {
                            samples += child.get_value().samples;
                            wins += child.get_value().wins;
                            let child_game_data_update = child.get_value().game_data_update;
                            if child
                                .get_mut_value()
                                .game_data
                                .check_consistency_of_game_data_update(
                                    &selection_node.get_value().game_data,
                                    &child_game_data_update,
                                    self.played_turns,
                                )
                            {
                                index += 1;
                            } else {
                                selection_node.swap_remove_child(index);
                                inconsistency_detected = true;
                            }
                        } else {
                            index += 1;
                        }
                    }
                    if inconsistency_detected {
                        wins = -wins / n_children;
                        samples = -samples / n_children;
                        let consistent_child_index = selection_node
                            .iter_children()
                            .position(|c| !c.get_value().samples.is_nan());
                        match consistent_child_index {
                            Some(index) => {
                                wins += selection_node.get_child(index).unwrap().get_value().wins;
                                samples +=
                                    selection_node.get_child(index).unwrap().get_value().samples;
                                self.reverse_propagation(selection_node.clone(), wins, samples);
                                selection_node.split_off_children(index, true);
                                selection_node.split_off_children(1, false);
                            }
                            None => {
                                self.reverse_propagation(selection_node.clone(), wins, samples);
                                selection_node.clear_children(0);
                                selection_node
                                    .get_mut_value()
                                    .set_next_node(self.force_update);
                                return true;
                            }
                        }
                    }
                    false
                }
                pub fn node_data_of_root_children(&self) -> (usize, Vec<(A, usize, f32, f32)>) {
                    let children_data: Vec<(A, usize, f32, f32)> = self
                        .tree_root
                        .iter_children()
                        .map(|c| {
                            (
                                c.get_value().player_action,
                                c.iter_pre_order_traversal().count(),
                                c.get_value().wins,
                                c.get_value().samples,
                            )
                        })
                        .collect();
                    let total_nodes = children_data.iter().map(|(_, n, ..)| n).sum();
                    (total_nodes, children_data)
                }
            }
        }
        pub use misc_types::MonteCarloGameMode;
        pub use misc_types::MonteCarloNodeType;
        pub use misc_types::MonteCarloPlayer;
        pub use node::MonteCarloNode;
        pub use traits::MonteCarloGameData;
        pub use traits::MonteCarloGameDataUpdate;
        pub use traits::MonteCarloPlayerAction;
        pub use tree_search::MonteCarloTreeSearch;
    }
    pub mod my_tic_tac_toe {
        use super::my_map_point::MapPoint;
        use super::my_map_two_dim::MyMap2D;
        use super::my_monte_carlo_tree_search::MonteCarloPlayer;
        pub const X: usize = 3;
        pub const Y: usize = X;
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
        pub enum TicTacToeStatus {
            #[default]
            Vacant,
            Player(MonteCarloPlayer),
            Tie,
        }
        impl TicTacToeStatus {
            pub fn is_vacant(&self) -> bool {
                *self == Self::Vacant
            }
            pub fn is_not_vacant(&self) -> bool {
                *self != Self::Vacant
            }
            pub fn is_player(&self) -> bool {
                matches!(self, Self::Player(_))
            }
        }
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
        pub struct TicTacToeGameData {
            map: MyMap2D<TicTacToeStatus, X, Y>,
            status: TicTacToeStatus,
            num_me_cells: u8,
            num_opp_cells: u8,
            num_tie_cells: u8,
        }
        impl TicTacToeGameData {
            pub fn new() -> Self {
                TicTacToeGameData {
                    map: MyMap2D::init(TicTacToeStatus::Vacant),
                    status: TicTacToeStatus::Vacant,
                    num_me_cells: 0,
                    num_opp_cells: 0,
                    num_tie_cells: 0,
                }
            }
            fn check_status_for_one_line<'a>(
                &self,
                line: impl Iterator<Item = &'a TicTacToeStatus>,
            ) -> TicTacToeStatus {
                let mut winner = TicTacToeStatus::Tie;
                for (index, element) in line.enumerate() {
                    if index == 0 {
                        match element {
                            TicTacToeStatus::Player(player) => {
                                winner = TicTacToeStatus::Player(*player)
                            }
                            _ => return TicTacToeStatus::Tie,
                        }
                    } else if winner != *element {
                        return TicTacToeStatus::Tie;
                    }
                }
                winner
            }
            fn check_status(&mut self, cell: MapPoint<X, Y>, check_lines: bool) -> TicTacToeStatus {
                if check_lines {
                    if let TicTacToeStatus::Player(player) =
                        self.check_status_for_one_line(self.map.iter_row(cell.y()).map(|(_, v)| v))
                    {
                        self.status = TicTacToeStatus::Player(player);
                        return self.status;
                    }
                    if let TicTacToeStatus::Player(player) = self
                        .check_status_for_one_line(self.map.iter_column(cell.x()).map(|(_, v)| v))
                    {
                        self.status = TicTacToeStatus::Player(player);
                        return self.status;
                    }
                    if cell.x() == cell.y() {
                        if let TicTacToeStatus::Player(player) =
                            self.check_status_for_one_line(self.iter_diagonal_top_left())
                        {
                            self.status = TicTacToeStatus::Player(player);
                            return self.status;
                        }
                    }
                    if cell.x() + cell.y() == 2 {
                        if let TicTacToeStatus::Player(player) =
                            self.check_status_for_one_line(self.iter_diagonal_top_right())
                        {
                            self.status = TicTacToeStatus::Player(player);
                            return self.status;
                        }
                    }
                }
                if self.num_me_cells + self.num_opp_cells + self.num_tie_cells == 9 {
                    self.status = TicTacToeStatus::Tie;
                }
                self.status
            }
            fn iter_diagonal_top_left(&self) -> impl Iterator<Item = &'_ TicTacToeStatus> {
                [(0_usize, 0_usize), (1, 1), (2, 2)]
                    .into_iter()
                    .map(|p| self.map.get(p.into()))
            }
            fn iter_diagonal_top_right(&self) -> impl Iterator<Item = &'_ TicTacToeStatus> {
                [(2_usize, 0_usize), (1, 1), (0, 2)]
                    .into_iter()
                    .map(|p| self.map.get(p.into()))
            }
            fn calc_line_heuristic<'a>(
                &self,
                line: impl Iterator<Item = &'a TicTacToeStatus>,
            ) -> f32 {
                let mut count: u8 = 0;
                let mut line_owner: Option<MonteCarloPlayer> = None;
                for cell in line {
                    match cell {
                        TicTacToeStatus::Vacant => (),
                        TicTacToeStatus::Tie => return 0.0,
                        TicTacToeStatus::Player(player) => match line_owner {
                            Some(owner) => {
                                if *player == owner {
                                    count += 1;
                                } else {
                                    return 0.0;
                                }
                            }
                            None => {
                                line_owner = Some(*player);
                                count += 1;
                            }
                        },
                    }
                }
                let line_heuristic = match count {
                    1 => 1.0,
                    2 => 10.0,
                    _ => 100.0,
                };
                match line_owner {
                    Some(player) => match player {
                        MonteCarloPlayer::Me => line_heuristic,
                        MonteCarloPlayer::Opp => -line_heuristic,
                    },
                    None => 0.0,
                }
            }
            pub fn calc_heuristic_(&self) -> f32 {
                let mut heuristic = 0.0;
                for rc in 0..3 {
                    heuristic += self.calc_line_heuristic(self.map.iter_row(rc).map(|(_, v)| v));
                    heuristic += self.calc_line_heuristic(self.map.iter_column(rc).map(|(_, v)| v));
                }
                heuristic += self.calc_line_heuristic(self.iter_diagonal_top_left());
                heuristic += self.calc_line_heuristic(self.iter_diagonal_top_right());
                heuristic
            }
            pub fn set_player(
                &mut self,
                cell: MapPoint<X, Y>,
                player: MonteCarloPlayer,
            ) -> TicTacToeStatus {
                let check_lines = match player {
                    MonteCarloPlayer::Me => {
                        self.num_me_cells += 1;
                        self.num_me_cells > 2
                    }
                    MonteCarloPlayer::Opp => {
                        self.num_opp_cells += 1;
                        self.num_opp_cells > 2
                    }
                };
                if self
                    .map
                    .swap_value(cell, TicTacToeStatus::Player(player))
                    .is_not_vacant()
                {
                    panic!("Set player on not vacant cell.");
                }
                self.check_status(cell, check_lines)
            }
            pub fn set_tie(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
                self.num_tie_cells += 1;
                if self
                    .map
                    .swap_value(cell, TicTacToeStatus::Tie)
                    .is_not_vacant()
                {
                    panic!("Set tie on not vacant cell.");
                }
                self.check_status(cell, false)
            }
            pub fn set_all_to_status(&mut self) -> TicTacToeStatus {
                for (_, cell) in self.map.iter_mut() {
                    *cell = self.status;
                }
                self.status
            }
            pub fn get_cell_value(&self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
                *self.map.get(cell)
            }
            pub fn get_first_vacant_cell(&self) -> Option<(MapPoint<X, Y>, &TicTacToeStatus)> {
                self.map.iter().find(|(_, v)| v.is_vacant())
            }
            pub fn count_player_cells(&self, count_player: MonteCarloPlayer) -> usize {
                self.map
                    .iter()
                    .filter(|(_, v)| match v {
                        TicTacToeStatus::Player(player) => *player == count_player,
                        _ => false,
                    })
                    .count()
            }
            pub fn iter_map(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &TicTacToeStatus)> {
                self.map.iter()
            }
        }
    }
    pub mod my_tree {
        mod back_track {
            use super::TreeNode;
            use std::collections::HashSet;
            use std::rc::Rc;
            pub struct BackTrack<N> {
                current_nodes: Vec<Rc<TreeNode<N>>>,
                finished: bool,
            }
            impl<N: PartialEq> BackTrack<N> {
                pub fn new(node: Rc<TreeNode<N>>) -> Self {
                    BackTrack {
                        current_nodes: vec![node],
                        finished: false,
                    }
                }
            }
            impl<N: PartialEq> Iterator for BackTrack<N> {
                type Item = Vec<Rc<TreeNode<N>>>;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.finished {
                        return None;
                    }
                    let result = Some(self.current_nodes.clone());
                    let mut seen = HashSet::new();
                    self.current_nodes = self
                        .current_nodes
                        .iter()
                        .flat_map(|c| c.iter_parents())
                        .filter(|n| seen.insert(n.get_id()))
                        .collect();
                    self.finished = self.current_nodes.is_empty();
                    result
                }
            }
        }
        mod iter_children {
            use super::TreeNode;
            use std::rc::Rc;
            pub struct IterChildren<N> {
                node: Rc<TreeNode<N>>,
                len_children: usize,
                child_index: usize,
                finished: bool,
            }
            impl<N: PartialEq> IterChildren<N> {
                pub fn new(node: Rc<TreeNode<N>>) -> Self {
                    let len_children = node.len_children();
                    IterChildren {
                        node,
                        len_children,
                        child_index: 0,
                        finished: false,
                    }
                }
            }
            impl<N: PartialEq> Iterator for IterChildren<N> {
                type Item = Rc<TreeNode<N>>;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.finished {
                        return None;
                    }
                    match self.node.get_child(self.child_index) {
                        Some(node) => {
                            self.child_index += 1;
                            Some(node)
                        }
                        None => {
                            self.finished = true;
                            None
                        }
                    }
                }
                fn size_hint(&self) -> (usize, Option<usize>) {
                    (0, Some(self.len_children))
                }
            }
            impl<N: PartialEq + Copy + Clone> ExactSizeIterator for IterChildren<N> {
                fn len(&self) -> usize {
                    self.len_children
                }
            }
        }
        mod iter_parents {
            use super::TreeNode;
            use std::rc::Rc;
            pub struct IterParents<N> {
                node: Rc<TreeNode<N>>,
                len_parents: usize,
                parent_index: usize,
                finished: bool,
            }
            impl<N: PartialEq> IterParents<N> {
                pub fn new(node: Rc<TreeNode<N>>) -> Self {
                    let len_parents = node.len_parents();
                    IterParents {
                        node,
                        len_parents,
                        parent_index: 0,
                        finished: false,
                    }
                }
            }
            impl<N: PartialEq> Iterator for IterParents<N> {
                type Item = Rc<TreeNode<N>>;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.finished {
                        return None;
                    }
                    while self.parent_index < self.len_parents {
                        let parent = self.node.get_parent(self.parent_index);
                        self.parent_index += 1;
                        if parent.is_none() {
                            continue;
                        }
                        return parent;
                    }
                    self.finished = true;
                    None
                }
                fn size_hint(&self) -> (usize, Option<usize>) {
                    (0, Some(self.len_parents))
                }
            }
            impl<N: PartialEq + Copy + Clone> ExactSizeIterator for IterParents<N> {
                fn len(&self) -> usize {
                    self.len_parents
                }
            }
        }
        mod iter_self {
            use super::TreeNode;
            use std::rc::Rc;
            pub struct IterSelf<N> {
                node: Rc<TreeNode<N>>,
                finished: bool,
            }
            impl<N: PartialEq> IterSelf<N> {
                pub fn new(node: Rc<TreeNode<N>>) -> Self {
                    IterSelf {
                        node,
                        finished: false,
                    }
                }
            }
            impl<N: PartialEq> Iterator for IterSelf<N> {
                type Item = Rc<TreeNode<N>>;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.finished {
                        return None;
                    }
                    self.finished = true;
                    Some(self.node.clone())
                }
            }
        }
        mod level_order_traversal {
            use super::TreeNode;
            use std::rc::Rc;
            pub struct LevelOrderTraversal<N> {
                current_node: Rc<TreeNode<N>>,
                child_indices: Vec<usize>,
                parent_ids: Vec<usize>,
                vertical: bool,
                finished: bool,
                target_level: usize,
                end_level: Option<usize>,
                node_on_target_level: bool,
            }
            impl<N: PartialEq> LevelOrderTraversal<N> {
                pub fn new(
                    start_node: Rc<TreeNode<N>>,
                    start_level: usize,
                    end_level: Option<usize>,
                ) -> Self {
                    if let Some(level) = end_level {
                        if start_level > level {
                            panic!("end_level must be >= start_level.");
                        }
                    }
                    let vec_capacity = start_node.get_max_level();
                    let mut child_indices: Vec<usize> = Vec::with_capacity(vec_capacity);
                    child_indices.push(0);
                    LevelOrderTraversal {
                        current_node: start_node,
                        child_indices,
                        parent_ids: Vec::with_capacity(vec_capacity),
                        vertical: false,
                        finished: false,
                        target_level: start_level,
                        end_level,
                        node_on_target_level: false,
                    }
                }
                fn increment_target_level(&mut self) -> bool {
                    if let Some(level) = self.end_level {
                        if self.target_level == level {
                            self.finished = true;
                            return true;
                        }
                    }
                    self.target_level += 1;
                    false
                }
            }
            impl<N: PartialEq> Iterator for LevelOrderTraversal<N> {
                type Item = (Rc<TreeNode<N>>, usize);
                fn next(&mut self) -> Option<Self::Item> {
                    if self.finished {
                        return None;
                    }
                    loop {
                        if self.vertical {
                            match self.parent_ids.pop() {
                                Some(parent_id) => {
                                    self.child_indices.pop();
                                    let last_child_index = self.child_indices.len() - 1;
                                    self.child_indices[last_child_index] += 1;
                                    self.current_node =
                                        self.current_node.get_parent_by_id(parent_id).unwrap();
                                }
                                None => {
                                    if self.node_on_target_level {
                                        if self.increment_target_level() {
                                            return None;
                                        }
                                        self.node_on_target_level = false;
                                        assert_eq!(self.child_indices.len(), 1);
                                        self.child_indices[0] = 0;
                                    } else {
                                        self.finished = true;
                                        return None;
                                    }
                                }
                            }
                            self.vertical = false;
                        } else {
                            if self.child_indices.len() - 1 == self.target_level {
                                self.node_on_target_level = true;
                                self.vertical = true;
                                return self
                                    .current_node
                                    .get_self()
                                    .map(|n| (n, self.target_level));
                            }
                            let child_index = self.child_indices[self.child_indices.len() - 1];
                            match self.current_node.get_child(child_index) {
                                Some(node) => {
                                    self.parent_ids.push(self.current_node.get_id());
                                    self.current_node = node;
                                    self.child_indices.push(0);
                                }
                                None => self.vertical = true,
                            }
                        }
                    }
                }
            }
        }
        mod post_order_traversal {
            use super::TreeNode;
            use std::rc::Rc;
            pub struct PostOrderTraversal<N> {
                current_node: Rc<TreeNode<N>>,
                child_indices: Vec<usize>,
                parent_ids: Vec<usize>,
                vertical: bool,
                finished: bool,
            }
            impl<N: PartialEq> PostOrderTraversal<N> {
                pub fn new(start_node: Rc<TreeNode<N>>) -> Self {
                    let vec_capacity = start_node.get_max_level();
                    let mut child_indices: Vec<usize> = Vec::with_capacity(vec_capacity);
                    child_indices.push(0);
                    PostOrderTraversal {
                        current_node: start_node,
                        child_indices,
                        parent_ids: Vec::with_capacity(vec_capacity),
                        vertical: false,
                        finished: false,
                    }
                }
            }
            impl<N: PartialEq> Iterator for PostOrderTraversal<N> {
                type Item = Rc<TreeNode<N>>;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.finished {
                        return None;
                    }
                    loop {
                        if self.vertical {
                            let result = self.current_node.get_self();
                            match self.parent_ids.pop() {
                                Some(parent_id) => {
                                    self.current_node =
                                        self.current_node.get_parent_by_id(parent_id).unwrap();
                                    self.child_indices.pop();
                                    let last_child_index = self.child_indices.len() - 1;
                                    self.child_indices[last_child_index] += 1;
                                    self.vertical = false;
                                }
                                None => self.finished = true,
                            }
                            return result;
                        } else {
                            let child_index = self.child_indices[self.child_indices.len() - 1];
                            match self.current_node.get_child(child_index) {
                                Some(node) => {
                                    self.parent_ids.push(self.current_node.get_id());
                                    self.current_node = node;
                                    self.child_indices.push(0);
                                }
                                None => self.vertical = true,
                            }
                        }
                    }
                }
            }
        }
        mod pre_order_traversal {
            use super::TreeNode;
            use std::rc::Rc;
            pub struct PreOrderTraversal<N> {
                current_node: Rc<TreeNode<N>>,
                child_indices: Vec<usize>,
                parent_ids: Vec<usize>,
                vertical: bool,
                iter_finished: bool,
            }
            impl<N: PartialEq> PreOrderTraversal<N> {
                pub fn new(start_node: Rc<TreeNode<N>>) -> Self {
                    let vec_capacity = start_node.get_max_level();
                    let mut child_indices: Vec<usize> = Vec::with_capacity(vec_capacity);
                    child_indices.push(0);
                    PreOrderTraversal {
                        current_node: start_node,
                        child_indices,
                        parent_ids: Vec::with_capacity(vec_capacity),
                        vertical: false,
                        iter_finished: false,
                    }
                }
            }
            impl<N: PartialEq> Iterator for PreOrderTraversal<N> {
                type Item = Rc<TreeNode<N>>;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.iter_finished {
                        return None;
                    }
                    let result = Some(self.current_node.clone());
                    loop {
                        if self.vertical {
                            match self.parent_ids.pop() {
                                Some(parent_id) => {
                                    self.child_indices.pop();
                                    let last_child_index = self.child_indices.len() - 1;
                                    self.child_indices[last_child_index] += 1;
                                    self.current_node =
                                        self.current_node.get_parent_by_id(parent_id).unwrap();
                                    self.vertical = false;
                                }
                                None => {
                                    self.iter_finished = true;
                                    break;
                                }
                            }
                        } else {
                            let child_index = self.child_indices[self.child_indices.len() - 1];
                            match self.current_node.get_child(child_index) {
                                Some(node) => {
                                    self.parent_ids.push(self.current_node.get_id());
                                    self.current_node = node;
                                    self.child_indices.push(0);
                                    break;
                                }
                                None => self.vertical = true,
                            }
                        }
                    }
                    result
                }
            }
        }
        mod tree_node {
            use super::BackTrack;
            use super::IterChildren;
            use super::IterParents;
            use super::IterSelf;
            use super::LevelOrderTraversal;
            use super::PostOrderTraversal;
            use super::PreOrderTraversal;
            use super::unique_id::generate_unique_id;
            use std::cell::RefCell;
            use std::cmp::Ordering;
            use std::rc::Rc;
            use std::rc::Weak;
            pub struct TreeNode<N> {
                value: RefCell<N>,
                id: usize,
                level: usize,
                max_level: Rc<RefCell<usize>>,
                node: RefCell<Weak<TreeNode<N>>>,
                parents: RefCell<Vec<Weak<TreeNode<N>>>>,
                children: RefCell<Vec<Rc<TreeNode<N>>>>,
            }
            impl<N: PartialEq> TreeNode<N> {
                pub fn seed_root(value: N, children_capacity: usize) -> Rc<TreeNode<N>> {
                    TreeNode::new(value, Weak::new(), children_capacity)
                }
                fn new(
                    value: N,
                    parent: Weak<TreeNode<N>>,
                    children_capacity: usize,
                ) -> Rc<TreeNode<N>> {
                    let (level, max_level, parents) = match parent.upgrade() {
                        Some(p) => {
                            let new_level = p.get_level() + 1;
                            let mut current_max_level = (*p.max_level).borrow_mut();
                            *current_max_level = current_max_level.max(new_level);
                            (
                                new_level,
                                p.max_level.clone(),
                                RefCell::new(vec![parent.clone()]),
                            )
                        }
                        None => (
                            0,
                            Rc::new(RefCell::new(0_usize)),
                            RefCell::new(Vec::with_capacity(1)),
                        ),
                    };
                    let result = Rc::new(TreeNode {
                        value: RefCell::new(value),
                        id: generate_unique_id(),
                        level,
                        max_level,
                        node: RefCell::new(Weak::new()),
                        parents,
                        children: RefCell::new(Vec::with_capacity(children_capacity)),
                    });
                    let node = Rc::downgrade(&result);
                    *result.node.borrow_mut() = node;
                    result
                }
                pub fn add_child_to_parent(
                    &self,
                    child_value: N,
                    parent_value: &N,
                    children_capacity: usize,
                ) -> Option<Rc<TreeNode<N>>> {
                    self.get_root()
                        .get_node(parent_value)
                        .map(|parent| parent.add_child(child_value, children_capacity))
                }
                pub fn add_child(&self, value: N, children_capacity: usize) -> Rc<TreeNode<N>> {
                    match self.iter_children().find(|n| *n.value.borrow() == value) {
                        Some(node) => node,
                        None => {
                            let child =
                                TreeNode::new(value, self.node.borrow().clone(), children_capacity);
                            self.children.borrow_mut().push(child.clone());
                            child
                        }
                    }
                }
                pub fn insert_child_at_parent(
                    &self,
                    child_value: N,
                    parent_value: &N,
                    index: usize,
                    children_capacity: usize,
                ) -> Option<Rc<TreeNode<N>>> {
                    self.get_root()
                        .get_node(parent_value)
                        .map(|parent| parent.insert_child(child_value, index, children_capacity))
                }
                pub fn insert_child(
                    &self,
                    value: N,
                    index: usize,
                    children_capacity: usize,
                ) -> Rc<TreeNode<N>> {
                    match self.iter_children().find(|n| *n.value.borrow() == value) {
                        Some(node) => node,
                        None => {
                            let child =
                                TreeNode::new(value, self.node.borrow().clone(), children_capacity);
                            let number_of_children = self.children.borrow().len();
                            if index < number_of_children {
                                self.children.borrow_mut().insert(index, child.clone());
                            } else {
                                self.children.borrow_mut().push(child.clone());
                            }
                            child
                        }
                    }
                }
                pub fn add_unambiguous_child_to_parent(
                    &self,
                    child_value: N,
                    parent_value: &N,
                    children_capacity: usize,
                ) -> Option<Rc<TreeNode<N>>> {
                    let root = self.get_root();
                    if root.get_node(&child_value).is_some() {
                        return None;
                    }
                    root.get_node(parent_value)
                        .map(|parent| parent.add_child(child_value, children_capacity))
                }
                pub fn add_unambiguous_child(
                    &self,
                    value: N,
                    children_capacity: usize,
                ) -> Option<Rc<TreeNode<N>>> {
                    match self
                        .get_root()
                        .iter_pre_order_traversal()
                        .find(|n| *n.value.borrow() == value)
                    {
                        Some(_) => None,
                        None => {
                            let child =
                                TreeNode::new(value, self.node.borrow().clone(), children_capacity);
                            self.children.borrow_mut().push(child.clone());
                            Some(child)
                        }
                    }
                }
                pub fn insert_unambiguous_child_at_parent(
                    &self,
                    child_value: N,
                    parent_value: &N,
                    index: usize,
                    children_capacity: usize,
                ) -> Option<Rc<TreeNode<N>>> {
                    let root = self.get_root();
                    if root.get_node(&child_value).is_some() {
                        return None;
                    }
                    root.get_node(parent_value)
                        .map(|parent| parent.insert_child(child_value, index, children_capacity))
                }
                pub fn insert_unambiguous_child(
                    &self,
                    value: N,
                    index: usize,
                    children_capacity: usize,
                ) -> Option<Rc<TreeNode<N>>> {
                    match self
                        .get_root()
                        .iter_pre_order_traversal()
                        .find(|n| *n.value.borrow() == value)
                    {
                        Some(_) => None,
                        None => {
                            let child =
                                TreeNode::new(value, self.node.borrow().clone(), children_capacity);
                            let number_of_children = self.children.borrow().len();
                            if index < number_of_children {
                                self.children.borrow_mut().insert(index, child.clone());
                            } else {
                                self.children.borrow_mut().push(child.clone());
                            }
                            Some(child)
                        }
                    }
                }
                pub fn link_child_to_parent(
                    &self,
                    child: Rc<TreeNode<N>>,
                ) -> Option<Rc<TreeNode<N>>> {
                    if self.level + 1 != child.level {
                        return None;
                    }
                    if !child.iter_parents().any(|p| p.id == self.id)
                        && !self.iter_children().any(|c| c.id == child.id)
                    {
                        child.parents.borrow_mut().push(self.node.borrow().clone());
                        self.children.borrow_mut().push(child.clone());
                    }
                    Some(child)
                }
                pub fn swap_remove_child(&self, index: usize) -> Option<Rc<TreeNode<N>>> {
                    if index >= self.len_children() {
                        return None;
                    }
                    let result = self.children.borrow_mut().swap_remove(index);
                    Some(result)
                }
                pub fn split_off_children(&self, at: usize, keep_split_off: bool) {
                    let split_off = self.children.borrow_mut().split_off(at);
                    if keep_split_off {
                        *self.children.borrow_mut() = split_off;
                    }
                }
                pub fn reserve_children(&self, additional_children: usize) {
                    self.children.borrow_mut().reserve(additional_children);
                }
                pub fn clear_children(&self, children_capacity: usize) {
                    *self.children.borrow_mut() = Vec::with_capacity(children_capacity);
                }
                pub fn clear_parent(&self) -> Option<Rc<TreeNode<N>>> {
                    self.parents.borrow_mut().clear();
                    self.get_self()
                }
                pub fn get_value(&self) -> std::cell::Ref<'_, N> {
                    self.value.borrow()
                }
                pub fn get_mut_value(&self) -> std::cell::RefMut<'_, N> {
                    self.value.borrow_mut()
                }
                pub fn get_id(&self) -> usize {
                    self.id
                }
                pub fn get_level(&self) -> usize {
                    self.level
                }
                pub fn get_self(&self) -> Option<Rc<TreeNode<N>>> {
                    self.node.borrow().upgrade().as_ref().cloned()
                }
                pub fn get_child(&self, index: usize) -> Option<Rc<TreeNode<N>>> {
                    self.children.borrow().get(index).cloned()
                }
                pub fn get_child_by_id(&self, id: usize) -> Option<Rc<TreeNode<N>>> {
                    self.iter_children().find(|c| c.get_id() == id)
                }
                pub fn len_children(&self) -> usize {
                    self.children.borrow().len()
                }
                pub fn get_parent(&self, index: usize) -> Option<Rc<TreeNode<N>>> {
                    self.parents
                        .borrow()
                        .get(index)?
                        .upgrade()
                        .as_ref()
                        .cloned()
                }
                pub fn get_parent_by_id(&self, id: usize) -> Option<Rc<TreeNode<N>>> {
                    self.iter_parents().find(|c| c.get_id() == id)
                }
                pub fn len_parents(&self) -> usize {
                    self.parents.borrow().len()
                }
                pub fn get_node(&self, value: &N) -> Option<Rc<TreeNode<N>>> {
                    self.iter_pre_order_traversal()
                        .find(|n| *n.value.borrow() == *value)
                }
                pub fn get_root(&self) -> Rc<TreeNode<N>> {
                    self.iter_back_track().last().unwrap()[0].clone()
                }
                pub fn is_root(&self) -> bool {
                    self.len_parents() == 0
                        || self.parents.borrow().iter().all(|n| n.weak_count() == 0)
                }
                pub fn is_leave(&self) -> bool {
                    self.len_children() == 0
                }
                pub fn sort_children_by<F>(&self, compare: F)
                where
                    F: Fn(&N, &N) -> Ordering,
                {
                    self.children
                        .borrow_mut()
                        .sort_by(|a, b| compare(&a.value.borrow(), &b.value.borrow()));
                }
                pub fn get_min_level(&self) -> usize {
                    self.get_root().level
                }
                pub fn get_max_level(&self) -> usize {
                    *self.max_level.borrow()
                }
                pub fn iter_self(&self) -> impl Iterator<Item = Rc<TreeNode<N>>> + use<N> {
                    IterSelf::new(self.get_self().unwrap())
                }
                pub fn iter_children(&self) -> impl Iterator<Item = Rc<TreeNode<N>>> + use<N> {
                    IterChildren::new(self.get_self().unwrap())
                }
                pub fn iter_parents(&self) -> impl Iterator<Item = Rc<TreeNode<N>>> + use<N> {
                    IterParents::new(self.get_self().unwrap())
                }
                pub fn iter_back_track(
                    &self,
                ) -> impl Iterator<Item = Vec<Rc<TreeNode<N>>>> + use<N> {
                    BackTrack::new(self.get_self().unwrap())
                }
                pub fn iter_pre_order_traversal(
                    &self,
                ) -> impl Iterator<Item = Rc<TreeNode<N>>> + use<N> {
                    PreOrderTraversal::new(self.get_self().unwrap())
                }
                pub fn iter_post_order_traversal(
                    &self,
                ) -> impl Iterator<Item = Rc<TreeNode<N>>> + use<N> {
                    PostOrderTraversal::new(self.get_self().unwrap())
                }
                pub fn iter_level_order_traversal(
                    &self,
                ) -> impl Iterator<Item = (Rc<TreeNode<N>>, usize)> + use<N> {
                    LevelOrderTraversal::new(self.get_self().unwrap(), 0, None)
                }
                pub fn iter_level_order_traversal_with_borders(
                    &self,
                    start_level: usize,
                    end_level: Option<usize>,
                ) -> impl Iterator<Item = (Rc<TreeNode<N>>, usize)> + use<N> {
                    LevelOrderTraversal::new(self.get_self().unwrap(), start_level, end_level)
                }
            }
        }
        mod unique_id {
            use std::sync::atomic::AtomicUsize;
            use std::sync::atomic::Ordering as AtomicOrdering;
            static GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);
            pub fn generate_unique_id() -> usize {
                GLOBAL_COUNTER.fetch_add(1, AtomicOrdering::SeqCst)
            }
        }
        use back_track::BackTrack;
        use iter_children::IterChildren;
        use iter_parents::IterParents;
        use iter_self::IterSelf;
        use level_order_traversal::LevelOrderTraversal;
        use post_order_traversal::PostOrderTraversal;
        use pre_order_traversal::PreOrderTraversal;
        pub use tree_node::TreeNode;
    }
}
