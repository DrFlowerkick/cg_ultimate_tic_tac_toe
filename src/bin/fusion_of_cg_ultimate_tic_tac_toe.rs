use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;
macro_rules! parse_input {
    ($ x : expr , $ t : ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}
fn main() {
    let max_number_of_turns = 81;
    let weighting_factor = 1.4;
    let time_out_first_turn = Duration::from_millis(990);
    let time_out_successive_turns = Duration::from_millis(90);
    let time_out_codingame_input = Duration::from_millis(2000);
    let mut game_data = UltTTT::new();
    let mut mcts_ult_ttt: TurnBasedMCTS<UltTTTMCTSGame, DynamicC, WithCache> =
        TurnBasedMCTS::new(weighting_factor);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        let opponent_row = parse_input!(inputs[0], i32);
        let opponent_col = parse_input!(inputs[1], i32);
        if tx.send((opponent_row, opponent_col)).is_err() {
            break;
        }
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
    });
    let (opponent_row, opponent_col) = rx.recv().expect("Failed to receive initial input");
    if opponent_row >= 0 {
        game_data.set_current_player(MonteCarloPlayer::Opp);
        let opp_action = MapPoint::<U, V>::new(opponent_col as usize, opponent_row as usize);
        game_data =
            UltTTTMCTSGame::apply_move(&game_data, &UltTTTPlayerAction::from_ext(opp_action));
    }
    let mut time_out = time_out_first_turn;
    loop {
        mcts_ult_ttt.set_root(&game_data);
        let start = Instant::now();
        let mut number_of_iterations = 0;
        while start.elapsed() < time_out {
            mcts_ult_ttt.iterate();
            number_of_iterations += 1;
        }
        eprintln!("Iterations: {}", number_of_iterations);
        time_out = time_out_successive_turns;
        let selected_move = mcts_ult_ttt.select_move();
        game_data = UltTTTMCTSGame::apply_move(&game_data, selected_move);
        selected_move.execute_action();
        mcts_ult_ttt.set_root(&game_data);
        let start = Instant::now();
        number_of_iterations = 0;
        loop {
            match rx.try_recv() {
                Ok((opponent_row, opponent_col)) => {
                    let opp_action =
                        MapPoint::<U, V>::new(opponent_col as usize, opponent_row as usize);
                    game_data = UltTTTMCTSGame::apply_move(
                        &game_data,
                        &UltTTTPlayerAction::from_ext(opp_action),
                    );
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    mcts_ult_ttt.iterate();
                    number_of_iterations += 1;
                    if start.elapsed() > time_out_codingame_input {
                        panic!("Timeout while waiting for codingame input");
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    panic!("Codingame input thread disconnected");
                }
            }
        }
        eprintln!("Pre-Fill Iterations: {}", number_of_iterations);
        assert!(game_data.game_turn <= max_number_of_turns);
    }
}
use std::fmt::Write;
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
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
struct UltTTT {
    map: MyMap2D<TicTacToeGameData, X, Y>,
    status_map: TicTacToeGameData,
    status: TicTacToeStatus,
    next_action_square_is_specified: Option<MapPoint<X, Y>>,
    current_player: MonteCarloPlayer,
    game_turn: usize,
}
impl UltTTT {
    fn new() -> Self {
        UltTTT {
            map: MyMap2D::new(),
            status_map: TicTacToeGameData::new(),
            status: TicTacToeStatus::Vacant,
            next_action_square_is_specified: None,
            current_player: MonteCarloPlayer::Me,
            game_turn: 0,
        }
    }
    fn set_current_player(&mut self, player: MonteCarloPlayer) {
        self.current_player = player;
    }
    fn next_player(&mut self) {
        self.current_player = self.current_player.next_player();
    }
    fn increment_game_turn(&mut self) {
        self.game_turn += 1;
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
        } else {
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
struct UltTTTMCTSGame {}
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
        new_state.execute_player_action(*mv, state.current_player);
        new_state.next_player();
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
use std::cmp::Ordering;
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default, Hash)]
struct MapPoint<const X: usize, const Y: usize> {
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
    fn new(x: usize, y: usize) -> Self {
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
    fn x(&self) -> usize {
        self.x
    }
    fn y(&self) -> usize {
        self.y
    }
    fn is_in_map(&self) -> bool {
        self.x < X && self.y < Y
    }
    fn forward_x(&self) -> Option<MapPoint<X, Y>> {
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
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct MyMap2D<T, const X: usize, const Y: usize> {
    items: [[T; X]; Y],
}
impl<T: Copy + Clone + Default, const X: usize, const Y: usize> MyMap2D<T, X, Y> {
    fn new() -> Self {
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
    fn init(init_element: T) -> Self {
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
    fn get(&self, coordinates: MapPoint<X, Y>) -> &T {
        &self.items[coordinates.y()][coordinates.x()]
    }
    fn get_mut(&mut self, coordinates: MapPoint<X, Y>) -> &mut T {
        &mut self.items[coordinates.y()][coordinates.x()]
    }
    fn swap_value(&mut self, coordinates: MapPoint<X, Y>, value: T) -> T {
        let old_value = self.items[coordinates.y()][coordinates.x()];
        self.items[coordinates.y()][coordinates.x()] = value;
        old_value
    }
    fn get_row(&self, row: usize) -> &[T] {
        if row >= Y {
            panic!("line {}, row out of range", line!());
        }
        &self.items[row][..]
    }
    fn iter(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        self.items.iter().enumerate().flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .map(move |(x, column)| (MapPoint::<X, Y>::new(x, y), column))
        })
    }
    fn iter_row(&self, r: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        self.get_row(r)
            .iter()
            .enumerate()
            .map(move |(x, column)| (MapPoint::new(x, r), column))
    }
    fn iter_column(&self, c: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
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
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum MonteCarloPlayer {
    #[default]
    Me,
    Opp,
}
impl MonteCarloPlayer {
    fn next_player(&self) -> Self {
        match self {
            MonteCarloPlayer::Me => MonteCarloPlayer::Opp,
            MonteCarloPlayer::Opp => MonteCarloPlayer::Me,
        }
    }
}
impl MCTSPlayer for MonteCarloPlayer {
    fn next(&self) -> Self {
        match self {
            MonteCarloPlayer::Me => MonteCarloPlayer::Opp,
            MonteCarloPlayer::Opp => MonteCarloPlayer::Me,
        }
    }
}
struct StaticC {}
impl<G: MCTSGame> UCTPolicy<G> for StaticC {}
struct DynamicC {}
impl<G: MCTSGame> UCTPolicy<G> for DynamicC {
    fn exploration_score(visits: usize, parent_visits: usize, c: f32) -> f32 {
        let dynamic_c = c / (1.0 + (visits as f32).sqrt());
        dynamic_c * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}
struct WithCache {
    exploitation: f32,
    exploration: f32,
    last_parent_visits: usize,
}
impl<G: MCTSGame, P: UCTPolicy<G>> MCTSCache<G, P> for WithCache {
    fn new() -> Self {
        WithCache {
            exploitation: 0.0,
            exploration: 0.0,
            last_parent_visits: 0,
        }
    }
    fn update_exploitation(
        &mut self,
        visits: usize,
        acc_value: f32,
        current_player: G::Player,
        perspective_player: G::Player,
    ) {
        self.exploitation =
            P::exploitation_score(acc_value, visits, current_player, perspective_player);
    }
    fn get_exploitation(&self, _v: usize, _a: f32, _c: G::Player, _p: G::Player) -> f32 {
        self.exploitation
    }
    fn update_exploration(&mut self, visits: usize, parent_visits: usize, base_c: f32) {
        if self.last_parent_visits != parent_visits {
            self.exploration = P::exploration_score(visits, parent_visits, base_c);
            self.last_parent_visits = parent_visits;
        }
    }
    fn get_exploration(&self, _v: usize, _p: usize, _b: f32) -> f32 {
        self.exploration
    }
}
trait MCTSPlayer: PartialEq {
    fn next(&self) -> Self;
}
trait MCTSGame {
    type State: Clone + PartialEq;
    type Move;
    type Player: MCTSPlayer;
    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a>;
    fn apply_move(state: &Self::State, mv: &Self::Move) -> Self::State;
    fn is_terminal(state: &Self::State) -> bool;
    fn evaluate(state: &Self::State) -> f32;
    fn current_player(state: &Self::State) -> Self::Player;
    fn perspective_player() -> Self::Player;
}
trait MCTSNode<G: MCTSGame> {
    fn get_state(&self) -> &G::State;
    fn get_move(&self) -> Option<&G::Move> {
        None
    }
    fn get_visits(&self) -> usize;
    fn get_accumulated_value(&self) -> f32;
    fn add_simulation_result(&mut self, result: f32);
    fn increment_visits(&mut self);
}
trait MCTSAlgo<G: MCTSGame> {
    fn iterate(&mut self);
    fn set_root(&mut self, state: &G::State) -> bool;
    fn select_move(&self) -> &G::Move;
}
trait UCTPolicy<G: MCTSGame> {
    fn exploitation_score(
        accumulated_value: f32,
        visits: usize,
        current_player: G::Player,
        perspective_player: G::Player,
    ) -> f32 {
        let raw = accumulated_value / visits as f32;
        if current_player == perspective_player {
            1.0 - raw
        } else {
            raw
        }
    }
    fn exploration_score(visits: usize, parent_visits: usize, base_c: f32) -> f32 {
        base_c * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}
trait MCTSCache<G: MCTSGame, P: UCTPolicy<G>> {
    fn new() -> Self;
    fn update_exploitation(
        &mut self,
        visits: usize,
        acc_value: f32,
        current_player: G::Player,
        perspective_player: G::Player,
    );
    fn get_exploitation(
        &self,
        visits: usize,
        acc_value: f32,
        current_player: G::Player,
        perspective_player: G::Player,
    ) -> f32;
    fn update_exploration(&mut self, visits: usize, parent_visits: usize, base_c: f32);
    fn get_exploration(&self, visits: usize, parent_visits: usize, base_c: f32) -> f32;
}
use rand::prelude::IteratorRandom;
struct TurnBasedNode<G: MCTSGame, P: UCTPolicy<G>, C: MCTSCache<G, P>> {
    state: G::State,
    visits: usize,
    accumulated_value: f32,
    mv: Option<G::Move>,
    children: Vec<usize>,
    cache: C,
    phantom: std::marker::PhantomData<P>,
}
impl<G: MCTSGame, P: UCTPolicy<G>, C: MCTSCache<G, P>> MCTSNode<G> for TurnBasedNode<G, P, C> {
    fn get_state(&self) -> &G::State {
        &self.state
    }
    fn get_move(&self) -> Option<&G::Move> {
        self.mv.as_ref()
    }
    fn get_visits(&self) -> usize {
        self.visits
    }
    fn get_accumulated_value(&self) -> f32 {
        self.accumulated_value
    }
    fn add_simulation_result(&mut self, result: f32) {
        self.accumulated_value += result;
        self.cache.update_exploitation(
            self.visits,
            self.accumulated_value,
            G::current_player(&self.state),
            G::perspective_player(),
        );
    }
    fn increment_visits(&mut self) {
        self.visits += 1;
    }
}
impl<G: MCTSGame, P: UCTPolicy<G>, C: MCTSCache<G, P>> TurnBasedNode<G, P, C> {
    fn root_node(state: G::State) -> Self {
        TurnBasedNode {
            state,
            visits: 0,
            accumulated_value: 0.0,
            mv: None,
            children: vec![],
            cache: C::new(),
            phantom: std::marker::PhantomData,
        }
    }
    fn new(state: G::State, mv: G::Move) -> Self {
        TurnBasedNode {
            state,
            visits: 0,
            accumulated_value: 0.0,
            mv: Some(mv),
            children: vec![],
            cache: C::new(),
            phantom: std::marker::PhantomData,
        }
    }
    fn add_child(&mut self, child_index: usize) {
        self.children.push(child_index);
    }
    fn get_children(&self) -> &Vec<usize> {
        &self.children
    }
    fn calc_utc(&mut self, parent_visits: usize, c: f32, perspective_player: G::Player) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let exploitation = self.cache.get_exploitation(
            self.visits,
            self.accumulated_value,
            G::current_player(&self.state),
            perspective_player,
        );
        self.cache.update_exploration(self.visits, parent_visits, c);
        let exploration = self.cache.get_exploration(self.visits, parent_visits, c);
        exploitation + exploration
    }
}
struct TurnBasedMCTS<G: MCTSGame, P: UCTPolicy<G>, C: MCTSCache<G, P>> {
    nodes: Vec<TurnBasedNode<G, P, C>>,
    root_index: usize,
    exploration_constant: f32,
}
impl<G: MCTSGame, P: UCTPolicy<G>, C: MCTSCache<G, P>> TurnBasedMCTS<G, P, C> {
    fn new(exploration_constant: f32) -> Self {
        Self {
            nodes: vec![],
            root_index: 0,
            exploration_constant,
        }
    }
}
impl<G: MCTSGame, P: UCTPolicy<G>, C: MCTSCache<G, P>> MCTSAlgo<G> for TurnBasedMCTS<G, P, C> {
    fn iterate(&mut self) {
        let mut path = vec![self.root_index];
        let mut current_index = self.root_index;
        while !self.nodes[current_index].get_children().is_empty() {
            let parent_visits = self.nodes[current_index].get_visits();
            let mut best_child_index = 0;
            let mut best_utc = f32::NEG_INFINITY;
            for vec_index in 0..self.nodes[current_index].get_children().len() {
                let child_index = self.nodes[current_index].get_children()[vec_index];
                let utc = self.nodes[child_index].calc_utc(
                    parent_visits,
                    self.exploration_constant,
                    G::perspective_player(),
                );
                if utc > best_utc {
                    best_utc = utc;
                    best_child_index = child_index;
                }
            }
            path.push(best_child_index);
            current_index = best_child_index;
        }
        let current_index = if G::is_terminal(self.nodes[current_index].get_state())
            || self.nodes[current_index].get_visits() == 0
        {
            current_index
        } else {
            let current_state = self.nodes[current_index].get_state().clone();
            for mv in G::available_moves(&current_state) {
                let new_state = G::apply_move(&current_state, &mv);
                let new_node = TurnBasedNode::new(new_state, mv);
                self.nodes.push(new_node);
                let child_index = self.nodes.len() - 1;
                self.nodes[current_index].add_child(child_index);
            }
            let child_index = *self.nodes[current_index]
                .get_children()
                .first()
                .expect("No children found");
            path.push(child_index);
            child_index
        };
        let mut current_state = self.nodes[current_index].get_state().clone();
        while !G::is_terminal(&current_state) {
            let random_move = G::available_moves(&current_state)
                .choose(&mut rand::thread_rng())
                .expect("No available moves");
            current_state = G::apply_move(&current_state, &random_move);
        }
        let simulation_result = G::evaluate(&current_state);
        for &node_index in path.iter().rev() {
            self.nodes[node_index].increment_visits();
            self.nodes[node_index].add_simulation_result(simulation_result);
        }
    }
    fn set_root(&mut self, state: &G::State) -> bool {
        if !self.nodes.is_empty() {
            if let Some(new_root) = self.nodes[self.root_index]
                .get_children()
                .iter()
                .flat_map(|&my_move_nodes| self.nodes[my_move_nodes].get_children())
                .find(|&&opponent_move_nodes| self.nodes[opponent_move_nodes].get_state() == state)
            {
                self.root_index = *new_root;
                return true;
            }
        }
        self.nodes.clear();
        self.nodes.push(TurnBasedNode::root_node(state.clone()));
        self.root_index = 0;
        false
    }
    fn select_move(&self) -> &G::Move {
        let move_index = self.nodes[self.root_index]
            .get_children()
            .iter()
            .max_by_key(|&&child_index| self.nodes[child_index].get_visits())
            .expect("could not find move_index");
        self.nodes[*move_index]
            .get_move()
            .expect("node did not contain move")
    }
}
const X: usize = 3;
const Y: usize = X;
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum TicTacToeStatus {
    #[default]
    Vacant,
    Player(MonteCarloPlayer),
    Tie,
}
impl TicTacToeStatus {
    fn is_vacant(&self) -> bool {
        *self == Self::Vacant
    }
    fn is_not_vacant(&self) -> bool {
        *self != Self::Vacant
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
struct TicTacToeGameData {
    map: MyMap2D<TicTacToeStatus, X, Y>,
    status: TicTacToeStatus,
    num_me_cells: u8,
    num_opp_cells: u8,
    num_tie_cells: u8,
    current_player: MonteCarloPlayer,
}
impl TicTacToeGameData {
    fn new() -> Self {
        TicTacToeGameData {
            map: MyMap2D::init(TicTacToeStatus::Vacant),
            status: TicTacToeStatus::Vacant,
            num_me_cells: 0,
            num_opp_cells: 0,
            num_tie_cells: 0,
            current_player: MonteCarloPlayer::Me,
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
                    TicTacToeStatus::Player(player) => winner = TicTacToeStatus::Player(*player),
                    _ => return TicTacToeStatus::Tie,
                }
            } else if winner != *element {
                return TicTacToeStatus::Tie;
            }
        }
        winner
    }
    fn check_status(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        let check_lines = match self.map.get(cell) {
            TicTacToeStatus::Vacant => false,
            TicTacToeStatus::Player(MonteCarloPlayer::Me) => self.num_me_cells > 2,
            TicTacToeStatus::Player(MonteCarloPlayer::Opp) => self.num_opp_cells > 2,
            TicTacToeStatus::Tie => false,
        };
        if check_lines {
            if let TicTacToeStatus::Player(player) =
                self.check_status_for_one_line(self.map.iter_row(cell.y()).map(|(_, v)| v))
            {
                self.status = TicTacToeStatus::Player(player);
                return self.status;
            }
            if let TicTacToeStatus::Player(player) =
                self.check_status_for_one_line(self.map.iter_column(cell.x()).map(|(_, v)| v))
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
            .iter()
            .map(move |p| self.map.get((*p).into()))
    }
    fn iter_diagonal_top_right(&self) -> impl Iterator<Item = &'_ TicTacToeStatus> {
        [(2_usize, 0_usize), (1, 1), (0, 2)]
            .iter()
            .map(move |p| self.map.get((*p).into()))
    }
    fn set_player(&mut self, cell: MapPoint<X, Y>, player: MonteCarloPlayer) -> TicTacToeStatus {
        match player {
            MonteCarloPlayer::Me => {
                self.num_me_cells += 1;
            }
            MonteCarloPlayer::Opp => {
                self.num_opp_cells += 1;
            }
        }
        if self
            .map
            .swap_value(cell, TicTacToeStatus::Player(player))
            .is_not_vacant()
        {
            dbg!(self.map.get(cell));
            panic!("Set player on not vacant cell.");
        }
        self.check_status(cell)
    }
    fn set_tie(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        self.num_tie_cells += 1;
        if self
            .map
            .swap_value(cell, TicTacToeStatus::Tie)
            .is_not_vacant()
        {
            panic!("Set tie on not vacant cell.");
        }
        self.check_status(cell)
    }
    fn set_all_to_status(&mut self) -> TicTacToeStatus {
        self.map = MyMap2D::init(self.status);
        self.status
    }
    fn get_cell_value(&self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        *self.map.get(cell)
    }
    fn get_first_vacant_cell(&self) -> Option<(MapPoint<X, Y>, &TicTacToeStatus)> {
        self.map.iter().find(|(_, v)| v.is_vacant())
    }
    fn count_player_cells(&self, count_player: MonteCarloPlayer) -> usize {
        self.map
            .iter()
            .filter(|(_, v)| match v {
                TicTacToeStatus::Player(player) => *player == count_player,
                _ => false,
            })
            .count()
    }
}
