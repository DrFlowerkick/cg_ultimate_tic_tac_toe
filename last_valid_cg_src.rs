//⏬my_map_point.rs
use std::cmp::Ordering;
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
struct MapPoint<const X: usize, const Y: usize> {
    // X: size of dimension x
    // Y: size of dimension Y
    // x and y are not public, because changing them without the provided functions can result in unwanted panics!
    x: usize,
    y: usize,
}
impl<const X: usize, const Y: usize> MapPoint<X, Y> {
    fn new(x: usize, y: usize) -> Self {
        if X == 0 {
            panic!("line {}, minimum size of dimension X is 1", line!());
        }
        if Y == 0 {
            panic!("line {}, minimum size of dimension Y is 1", line!());
        }
        let result = MapPoint { x, y, };
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
        // increments x, if x reaches row end, move to start of next row; if x reaches end of map, return None
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
    fn offset_pp(&self, offset: (usize, usize)) -> Option<MapPoint<X, Y>> {
        let result = MapPoint {
            x: self.x + offset.0,
            y: self.y + offset.1,
        };
        if result.is_in_map() {
            Some(result)
        } else {
            None
        }
    }
    fn offset_mm(&self, offset: (usize, usize)) -> Option<MapPoint<X, Y>> {
        if offset.0 > self.x || offset.1 > self.y {
            return None;
        }
        let result = MapPoint {
            x: self.x - offset.0,
            y: self.y - offset.1,
        };
        if result.is_in_map() {
            Some(result)
        } else {
            None
        }
    }
    fn neighbor(&self, orientation: Compass) -> Option<MapPoint<X, Y>> {
        match orientation {
            Compass::Center => Some(*self),
            Compass::N => self.offset_mm((0, 1)),
            Compass::NE => self.offset_mm((0, 1)).map_or(None, |n| n.offset_pp((1, 0))),
            Compass::E => self.offset_pp((1, 0)),
            Compass::SE => self.offset_pp((1, 1)),
            Compass::S => self.offset_pp((0, 1)),
            Compass::SW => self.offset_pp((0, 1)).map_or(None, |s| s.offset_mm((1, 0))),
            Compass::W => self.offset_mm((1, 0)),
            Compass::NW => self.offset_mm((1, 1)),
        }
    }
    fn iter_orientation(&self, orientation: Compass) -> impl Iterator<Item = MapPoint<X, Y>> {
        OrientationIter::new(*self, orientation)
    }
}
struct NeighborIter<const X: usize, const Y: usize> {
    include_center: bool,
    include_corners: bool,
    center_point: MapPoint<X, Y>,
    initial_orientation: Compass,
    current_orientation: Compass,
    rotation_direction: bool,
    finished: bool,
}
impl<const X: usize, const Y: usize>NeighborIter<X, Y> {
    fn rotate_orientation(&mut self) {
        if self.include_center {
            self.include_center = false;
        } else if self.rotation_direction {
            // rotate clockwise
            self.current_orientation = if self.include_corners {
                self.current_orientation.clockwise()
            } else {
                self.current_orientation.clockwise().clockwise()
            };
            self.finished = self.current_orientation == self.initial_orientation;
        } else {
            // rotate counterclockwise
            self.current_orientation = if self.include_corners {
                self.current_orientation.counterclockwise()
            } else {
                self.current_orientation.counterclockwise().counterclockwise()
            };
            self.finished = self.current_orientation == self.initial_orientation;
        }
    }
}
impl<const X: usize, const Y: usize> Iterator for NeighborIter<X, Y> {
    type Item = (MapPoint<X, Y>, Compass);
    fn next(&mut self) -> Option<Self::Item> {
        while !self.finished {
            let result = if self.include_center {
                Some((self.center_point, Compass::Center))
            } else {
                self.center_point.neighbor(self.current_orientation).map_or(None, |n| Some((n, self.current_orientation)))
            };
            match result {
                Some(map_point) => {
                    self.rotate_orientation();
                    return Some(map_point);
                },
                None => self.rotate_orientation(),
            }
        }
        None
    }
}
struct OrientationIter<const X: usize, const Y: usize> {
    current_point: MapPoint<X, Y>,
    orientation: Compass,
    finished: bool,
}
impl <const X: usize, const Y: usize>OrientationIter<X, Y> {
    fn new(start_point: MapPoint<X, Y>, orientation: Compass) -> Self {
        if orientation.is_center() {
            panic!("line {}, need direction", line!());
        }
        OrientationIter {
            current_point: start_point,
            orientation,
            finished: false,
        }
    }
}
impl<const X: usize, const Y: usize> Iterator for OrientationIter<X, Y> {
    type Item = MapPoint<X, Y>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None
        }
        let result = self.current_point;
        match self.current_point.neighbor(self.orientation) {
            Some(map_point) => self.current_point = map_point,
            None => self.finished = true,
        }
        Some(result)
    }
}
//⏫my_map_point.rs
//⏬my_map_two_dim.rs
// use MyMap2D if compilation time is suffice, because it is more efficient and has cleaner interface
#[derive(Copy, Clone, PartialEq)]
struct MyMap2D<T, const X: usize, const Y: usize, const N: usize> { // X: number of columns, Y: number of rows, N: number of elements in map: X * Y
    items: [[T; X] ; Y], //outer array rows, inner array columns -> first index chooses row (y), second index chooses column (x)
}
impl<T: Copy + Clone + Default, const X: usize, const Y: usize, const N: usize> MyMap2D<T, X, Y, N> {
    fn new() -> Self {
        if X == 0 {
            panic!("line {}, minimum one column", line!());
        }
        if Y == 0 {
            panic!("line {}, minimum one row", line!());
        }
        Self { items: [[T::default(); X] ; Y], }
    }
    fn init(init_element: T) -> Self {
        if X == 0 {
            panic!("line {}, minimum one column", line!());
        }
        if Y == 0 {
            panic!("line {}, minimum one row", line!());
        }
        Self { items: [[init_element; X] ; Y], }
    }
    fn get(&self, coordinates: MapPoint<X, Y>) -> &T {
        &self.items[coordinates.y()][coordinates.x()]
    }
    fn get_mut(&mut self, coordinates: MapPoint<X, Y>) -> &mut T {
        &mut self.items[coordinates.y()][coordinates.x()]
    }
    fn set(&mut self, coordinates: MapPoint<X, Y>, value: T) -> &T {
        self.items[coordinates.y()][coordinates.x()] = value;
        &self.items[coordinates.y()][coordinates.x()]
    }
    fn iter(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        self.items
            .iter()
            .enumerate()
            .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, column)| (MapPoint::<X, Y>::new(x, y), column)))
    }
    fn iter_row(&self, r: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        if r >= Y {
            panic!("line {}, row index is out of range", line!());
        }
        self.items
            .iter()
            .enumerate()
            .filter(move |(y, _)| *y == r)
            .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, column)| (MapPoint::new(x, y), column)))
    }
    fn iter_column(&self, c: usize) -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        if c >= X {
            panic!("line {}, column index is out of range", line!());
        }
        self.items
            .iter()
            .enumerate()
            .flat_map(move |(y, row)| row.iter().enumerate().filter(move |(x, _)| *x == c).map(move |(x, column)| (MapPoint::new(x, y), column)))
    }
    fn iter_diagonale_top_left(&self)  -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        MapPoint::<X, Y>::new(0, 0).iter_orientation(Compass::SE).map(move |p| (p, self.get(p)))
    }
    fn iter_diagonale_top_right(&self)  -> impl Iterator<Item = (MapPoint<X, Y>, &T)> {
        MapPoint::<X, Y>::new(X - 1, 0).iter_orientation(Compass::SW).map(move |p| (p, self.get(p)))
    }
}
impl<T: Copy + Clone + Default, const X: usize, const Y: usize, const N: usize> Default for MyMap2D<T, X, Y, N> {
    fn default() -> Self {
        Self::new()
    }
}
//⏫my_map_two_dim.rs
//⏬my_monte_carlo_tree_search.rs
use rand::prelude::*;
use rand::seq::IteratorRandom;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;
use std::any::Any;
#[derive(Copy, Clone, PartialEq, Debug)]
enum MonteCarloPlayer {
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
#[derive(Copy, Clone, PartialEq)]
enum MonteCarloNodeType {
    GameDataUpdate,
    ActionResult,
}
#[derive(Copy, Clone, PartialEq)]
// each game mode describes a different handling of player actions, see below
// normally each player has one action
// if multiple actions per player are possible, than starting_player does his actions, afterward the other player. this is true for every mode
enum MonteCarloGameMode {
    ByTurns, // each turn only one player acts, players switch at turn end
}
#[derive(Copy, Clone, PartialEq)]
enum MonteCarloNodeConsistency {
}
// Trait for actions players can take to interact with game data.
trait MonteCarloPlayerAction: Copy + Clone + PartialEq + Default + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn downcast_self(player_action: &impl MonteCarloPlayerAction) -> &Self;
    fn iter_actions(game_data: &impl MonteCarloGameData, player: MonteCarloPlayer, parent_game_turn: usize) -> Box<dyn Iterator<Item=Self> + '_>;
}
// Trait for updating game data after modifications through players. Normally there as some kind of random factor involved, e.g. drawing new ressources of several kind from a "bag".
trait MonteCarloGameDataUpdate: Copy + Clone + PartialEq + Default + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn downcast_self(game_data_update: &impl MonteCarloGameDataUpdate) -> &Self;
    fn iter_game_data_updates(game_data: &impl MonteCarloGameData, force_update: bool) -> Box<dyn Iterator<Item=Self> + '_>;
}
// trait for game data, which works with Monte Carlo Tree Search
trait MonteCarloGameData: Copy + Clone + PartialEq + Default + 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn downcast_self(game_data: &impl MonteCarloGameData) -> &Self;
    fn apply_my_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool; // true if score event, which results in change of heuristic
    fn apply_opp_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool; // true if score event, which results in change of heuristic
    fn simultaneous_player_actions_for_simultaneous_game_data_change(&mut self, my_action: &impl MonteCarloPlayerAction, opp_action: &impl MonteCarloPlayerAction);
    fn is_game_data_update_required(&self, force_update: bool) -> bool;
    fn apply_game_data_update(&mut self, game_data_update: &impl MonteCarloGameDataUpdate, check_update_consistency: bool) -> bool; // true if consistent 
    fn calc_heuristic(&self) -> f32;
    fn check_game_ending(&self, game_turn: usize) -> bool;
    fn game_winner(&self, game_turn: usize) -> Option<MonteCarloPlayer>; // None if tie
    fn check_consistency_of_game_data_during_init_root(&mut self, current_game_state: &Self, played_turns: usize) -> bool;
    fn check_consistency_of_game_data_update(&mut self, current_game_state: &Self, game_data_update: &impl MonteCarloGameDataUpdate, played_turns: usize) -> bool;
    fn check_consistency_of_action_result(&mut self, current_game_state: Self, my_action: &impl MonteCarloPlayerAction, opp_action: &impl MonteCarloPlayerAction, played_turns: usize, apply_player_actions_to_game_data: bool) -> bool;
}
// "G" is a trait object for a game data 
#[derive(PartialEq, Clone, Copy)]
struct MonteCarloNode<G: MonteCarloGameData, A: MonteCarloPlayerAction, U: MonteCarloGameDataUpdate> {
    game_data: G,
    player_action: A,
    game_data_update: U,
    node_type: MonteCarloNodeType,
    next_node: MonteCarloNodeType,
    player: MonteCarloPlayer,
    game_turn: usize,
    heuristic: f32,
    alpha: f32,
    beta: f32,
    wins: f32,
    samples: f32,
    parent_samples: f32,
    exploitation_score: f32, // exploitation_score is needed to choose best action and to choose node to exploit
    exploration_score: f32, // exploration_score is needed to identify nodes for exploration
    heuristic_score: f32,
    total_score: f32,
    pruned_node: bool,
    game_end_node: bool, // leave, at which the game ends
}
impl <G: MonteCarloGameData, A: MonteCarloPlayerAction, U: MonteCarloGameDataUpdate> MonteCarloNode<G, A, U> {
    fn new() -> Self {
        MonteCarloNode {
            game_data: G::default(),
            player_action: A::default(),
            game_data_update: U::default(),
            node_type: MonteCarloNodeType::ActionResult,
            next_node: MonteCarloNodeType::ActionResult,
            player: MonteCarloPlayer::Me,
            game_turn: 0,
            heuristic: 0.0,
            alpha: f32::INFINITY,
            beta: f32::NEG_INFINITY,
            wins: 0.0,
            samples: f32::NAN,
            parent_samples: 0.0,
            exploitation_score: 0.0,
            exploration_score: 0.0,
            heuristic_score: 0.0,
            total_score: 0.0,
            pruned_node: false,
            game_end_node: false,
        }
    }
    fn new_player_action_child(&self, player_action: A) -> Self {
        let mut new_child = Self::new();
        new_child.player_action = player_action;
        new_child.parent_samples = self.samples;
        new_child.game_turn = self.game_turn;
        new_child.player = self.player;
        new_child
    }
    fn new_game_data_update_child(&self, game_data_update: U) -> Self {
        let mut new_child = Self::new();
        new_child.game_data_update = game_data_update;
        new_child.parent_samples = self.samples;
        new_child.game_turn = self.game_turn;
        new_child.player = self.player;
        new_child.node_type = MonteCarloNodeType::GameDataUpdate;
        new_child
    }
    fn calc_heuristic(&mut self, use_heuristic_score: bool) {
        if use_heuristic_score {
            self.heuristic = self.game_data.calc_heuristic();
            match self.player {
                MonteCarloPlayer::Me => self.alpha = self.heuristic,
                MonteCarloPlayer::Opp => self.beta = self.heuristic,
            }
        }
    }
    fn calc_node_score(&mut self, parent_samples: f32, weighting_factor: f32) {
        if parent_samples != self.parent_samples {
            self.update_exploration_score(parent_samples, weighting_factor);
        }
        self.total_score = match self.player {
            MonteCarloPlayer::Me => self.exploitation_score + self.exploration_score - self.heuristic_score,
            MonteCarloPlayer::Opp => self.exploitation_score + self.exploration_score + self.heuristic_score,
        };
    }
    fn check_game_turn(&mut self, game_mode: MonteCarloGameMode) {
        match game_mode {
            MonteCarloGameMode::ByTurns => self.game_turn += 1,
        }
    }
    fn set_next_node(&mut self, force_update: bool) {
        if !self.game_end_node {
            self.next_node = if self.game_data.is_game_data_update_required(force_update) {
                MonteCarloNodeType::GameDataUpdate
            } else {
                MonteCarloNodeType::ActionResult
            };
        }
    }
    fn apply_action(&mut self, parent_game_data: &G, _parent_action: &A, game_mode: MonteCarloGameMode, max_number_of_turns: usize, use_heuristic_score: bool) -> bool {
        // transfer game_data of parent
        self.game_data = *parent_game_data;
        self.samples = 0.0;
        // score_event depends on player action (e.g. scoring points) or end of game
        let mut score_event = self.apply_player_action();
        self.player = self.player.next_player();
        self.check_game_turn(game_mode);
        match game_mode {
            MonteCarloGameMode::ByTurns => {
                score_event = self.check_game_ending(max_number_of_turns) || score_event;
            }
        }
        if score_event {
            self.calc_heuristic(use_heuristic_score);
        }
        score_event && use_heuristic_score
    }
    fn apply_game_data_update(&mut self, parent_game_data: &G, check_update_consistency: bool) -> bool {
        // transfer game_data of parent
        self.game_data = *parent_game_data;
        self.samples = 0.0;
        // apply update
        self.game_data.apply_game_data_update(&self.game_data_update, check_update_consistency)
    }
    fn apply_player_action(&mut self) -> bool {
        match self.player {
            MonteCarloPlayer::Me => self.game_data.apply_my_action(&self.player_action),
            MonteCarloPlayer::Opp => self.game_data.apply_opp_action(&self.player_action),
        }
    }
    fn check_game_ending(&mut self, max_number_of_turns: usize) -> bool {
        self.game_end_node = self.game_turn == max_number_of_turns || self.game_data.check_game_ending(self.game_turn);
        self.game_end_node
    }
    fn calc_playout_score(&self) -> f32 {
        match self.game_data.game_winner(self.game_turn) {
            Some(player) => match player {
                MonteCarloPlayer::Me => 1.0,
                MonteCarloPlayer::Opp => 0.0,
            },
            None => 0.5,
        }
    }
    fn score_playout_result(&mut self, playout_score: f32, samples: f32, use_heuristic_score: bool) {
        self.wins += playout_score;
        self.samples += samples;
        self.exploitation_score = match self.player {
            MonteCarloPlayer::Me => 1.0 - self.wins / self.samples,
            MonteCarloPlayer::Opp => self.wins / self.samples,
        };
        if use_heuristic_score {
            self.heuristic_score = match self.player {
                MonteCarloPlayer::Me => {
                    if self.alpha.is_finite() {
                        self.alpha / self.samples
                    } else {
                        0.0
                    }
                },
                MonteCarloPlayer::Opp => {
                    if self.beta.is_finite() {
                        self.beta / self.samples
                    } else {
                        0.0
                    }
                }
            };
        }
    }
    fn update_exploration_score(&mut self, parent_samples: f32, weighting_factor: f32) {
        self.parent_samples = parent_samples;
        self.exploration_score = weighting_factor * (self.parent_samples.log10()/self.samples).sqrt();
    }
    fn update_consistent_node_during_init_phase(&mut self, current_game_state: &G, played_turns: usize, force_update: bool) -> bool {
        if !force_update {
            if !self.game_data.check_consistency_of_game_data_during_init_root(current_game_state, played_turns) {
                return false
            }
        }
        self.game_data == *current_game_state
    }
}
struct MonteCarloTreeSearch<G: MonteCarloGameData, A: MonteCarloPlayerAction, U: MonteCarloGameDataUpdate> {
    tree_root: Rc<TreeNode<MonteCarloNode<G, A, U>>>,
    keep_root: Option<Rc<TreeNode<MonteCarloNode<G, A, U>>>>,
    root_level: usize,
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
    debug: bool,
}
impl <G: MonteCarloGameData, A: MonteCarloPlayerAction, U: MonteCarloGameDataUpdate> MonteCarloTreeSearch<G, A, U> {
    fn new(game_mode: MonteCarloGameMode, max_number_of_turns: usize, force_update: bool, time_out_first_turn: Duration, time_out_successive_turns: Duration, weighting_factor: f32, use_heuristic_score: bool, debug: bool, keep_root: bool) -> Self {
        let mut result = MonteCarloTreeSearch {
            tree_root: TreeNode::seed_root(MonteCarloNode::<G, A, U>::new(), 0),
            keep_root: None,
            root_level: 0,
            game_mode,
            starting_player: MonteCarloPlayer::Me,
            played_turns: 0,
            max_number_of_turns,
            force_update,
            first_turn: true,
            time_out_first_turn,
            time_out_successive_turns,
            weighting_factor, // try starting with 1.0 and find a way to applicate a better value
            use_heuristic_score,
            debug,
        };
        if keep_root {
            result.keep_root = Some(result.tree_root.clone());
        }
        result
    }
    fn init_root(&mut self, game_data: &G, starting_player: MonteCarloPlayer) -> Instant {
        let start = Instant::now();
        if self.first_turn {
            self.starting_player = starting_player;
            // init root with initial game data
            self.tree_root.get_mut_value().game_data = *game_data;
            self.tree_root.get_mut_value().samples = 0.0;
            if self.game_mode == MonteCarloGameMode::ByTurns && self.starting_player == MonteCarloPlayer::Opp {
                // if opp is starting player, than with turn wise actions opp player already played a turn
                self.played_turns = 1;
                self.tree_root.get_mut_value().game_turn = 1;
                self.tree_root.get_mut_value().player = MonteCarloPlayer::Me;
            } else {
                // no action made yet: tree_root represents initial game data
                self.tree_root.get_mut_value().node_type = MonteCarloNodeType::GameDataUpdate;
                self.tree_root.get_mut_value().player = starting_player;
            }
        } else {
            // search new root node and move tree_root to it
            // root node is one node before next possible node with starting player as node owner
            let (search_turn, end_level) = match self.game_mode {
                MonteCarloGameMode::ByTurns => (self.played_turns + 1, Some(2)),
            };
            match self.tree_root.iter_level_order_traversal_with_bordes(1, end_level).find(|(n, _)| {
                let mut n_value = n.get_mut_value();
                n_value.game_turn == search_turn &&
                n_value.next_node == MonteCarloNodeType::ActionResult &&
                n_value.player == MonteCarloPlayer::Me &&
                n_value.update_consistent_node_during_init_phase(game_data, self.played_turns, self.force_update)
            }) {
                Some((new_root, _)) => {
                    self.tree_root = new_root;
                    self.root_level = self.tree_root.get_level();
                },
                None => {
                    // create new tree_root, since no node with game_data has been found
                    if self.debug {
                        eprintln!("Current game state not found in tree. Reinit tree after {} played turns", self.played_turns);
                    }
                    if self.keep_root.is_some() {
                        panic!("quit since root has been reset.");
                    }
                    self.tree_root = TreeNode::seed_root(MonteCarloNode::<G, A, U>::new(), 0);
                    self.root_level = 0;
                    self.tree_root.get_mut_value().game_data = *game_data;
                    self.tree_root.get_mut_value().samples = 0.0;
                    self.tree_root.get_mut_value().player = MonteCarloPlayer::Me;
                    self.tree_root.get_mut_value().game_turn = search_turn;
                },
            }
        }
        start
    }
    fn expand_tree(&mut self, start: Instant) {
        let time_out = if self.first_turn {
            self.first_turn = false;
            self.time_out_first_turn
        } else {
            self.time_out_successive_turns
        };
        // loop until time out or no more nodes to cycle
        let mut counter = 0;
        while start.elapsed() < time_out && !self.one_cycle(&start, time_out) {
            counter += 1;
        }
        if self.debug {
            eprintln!("number of expand cycles: {}", counter);
        }
    }
    fn choose_and_execute_actions(&mut self) -> (impl MonteCarloGameData, impl MonteCarloPlayerAction) {
        // my best action is at max exploitation_score
        let child = self.tree_root.iter_children().max_by(|x, y| x.get_value().exploitation_score.partial_cmp(&y.get_value().exploitation_score).unwrap()).unwrap();
        self.played_turns = child.get_value().game_turn;
        self.tree_root = child.clone();
        self.root_level = self.tree_root.get_level();
        // return game_data and my action
        let result = (child.get_value().game_data, child.get_value().player_action);
        result
    }
    fn one_cycle(&self, start: &Instant, time_out: Duration) -> bool {
        let selection_node = self.selection(start, time_out);
        match selection_node {
            Some(selection_node) => {
                let child_node = self.expansion(selection_node);
                match self.playout(child_node.clone(), start, time_out) {
                    Some((playout_score, backtrack_heuristic)) => self.propagation(child_node, playout_score, backtrack_heuristic),
                    None => (),
                }
            },
            None => return true, // no more nodes to simulate in tree or time over
        }
        false
    }
    fn selection(&self, start: &Instant, time_out: Duration) -> Option<Rc<TreeNode<MonteCarloNode<G, A, U>>>> {
        let mut rng = thread_rng();
        // search for node to select
        let mut selection_node = self.tree_root.clone();
        while !selection_node.is_leave() {
            if start.elapsed() >= time_out {
                // return None, if selection cannot finish in time
                return None;
            }
            // remove inconsistent children, if next_node is GameDataUpdate
            // if consistent child is detected it will be updated
            // if all children removed, return selection_node
            if self.remove_inconsistent_children(selection_node.clone()) {
                return Some(selection_node);
            }
            // search children without samples
            match selection_node.iter_children().filter(|c| c.get_value().samples.is_nan()).choose(&mut rng) {
                Some(child_without_samples) => return Some(child_without_samples),
                None => (),
            }
            selection_node.iter_children().for_each(|c| c.get_mut_value().calc_node_score(selection_node.get_value().samples, self.weighting_factor));
            let selected_child = selection_node.iter_children().max_by(|a, b| a.get_value().total_score.partial_cmp(&b.get_value().total_score).unwrap());
            selection_node = match selected_child {
                Some(child) => {
                    if self.force_update {
                        child.clone()
                    } else {
                        let node_type = child.get_value().node_type;
                        match node_type {
                            MonteCarloNodeType::ActionResult => {
                                // update child with parent game state (if no update is needed, nothing happens)
                                let child_action = child.get_value().player_action;
                                let apply_player_actions_to_game_data = match self.game_mode {
                                    MonteCarloGameMode::ByTurns => true,
                                };
                                let child_game_data_changed = child.get_mut_value().game_data.check_consistency_of_action_result(selection_node.get_value().game_data, &selection_node.get_value().player_action, &child_action, self.played_turns, apply_player_actions_to_game_data);
                                if child_game_data_changed && child.get_value().next_node == MonteCarloNodeType::GameDataUpdate && child.is_leave() {
                                    child.get_mut_value().set_next_node(self.force_update);
                                }
                                child.clone()
                            },
                            MonteCarloNodeType::GameDataUpdate => child.clone(),
                        }
                    }
                },
                None => panic!("selection should alway find a child!"),
            };
        }
        Some(selection_node)
    }
    fn expansion(&self, expansion_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>) -> Rc<TreeNode<MonteCarloNode<G, A, U>>> {
        if expansion_node.get_value().game_end_node || (expansion_node.get_level() > self.root_level && expansion_node.get_value().samples.is_nan()) {
            return expansion_node;
        }
        let next_node = expansion_node.get_value().next_node;
        match next_node {
            MonteCarloNodeType::GameDataUpdate => {
                for game_data_update in U::iter_game_data_updates(&expansion_node.get_value().game_data, self.force_update) {
                    let new_game_data_update_node = expansion_node.get_value().new_game_data_update_child(game_data_update);
                    expansion_node.add_child(new_game_data_update_node, 0);
                }
            },
            MonteCarloNodeType::ActionResult => {
                for player_action in A::iter_actions(&expansion_node.get_value().game_data, expansion_node.get_value().player, expansion_node.get_value().game_turn) {
                    let new_player_action_node = expansion_node.get_value().new_player_action_child(player_action);
                    expansion_node.add_child(new_player_action_node, 0);
                }
            },
        }
        expansion_node.get_child(0).unwrap()
    }
    fn playout(&self, playout_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>, start: &Instant, time_out: Duration) -> Option<(f32, bool)> {
        if playout_node.get_value().game_end_node {
            Some((playout_node.get_value().calc_playout_score(), false))
        } else {
            let node_type = playout_node.get_value().node_type;
            let parent = playout_node.get_parent().unwrap();
            let backtrack_heuristic = match node_type {
                MonteCarloNodeType::GameDataUpdate => {
                    if !playout_node.get_mut_value().apply_game_data_update(&parent.get_value().game_data, !self.force_update) {
                        // node is inconsistent -> delete this node from parent and search for new child
                        let index = parent.iter_children().position(|c| *c.get_value() == *playout_node.get_value()).unwrap();
                        parent.swap_remove_child(index);
                        return None;
                    }
                    playout_node.get_mut_value().set_next_node(self.force_update);
                    false
                },
                MonteCarloNodeType::ActionResult => {
                    let parent_action = parent.get_value().player_action;
                    let backtrack_heuristic = playout_node.get_mut_value().apply_action(&parent.get_value().game_data, &parent_action, self.game_mode, self.max_number_of_turns, self.use_heuristic_score);
                    playout_node.get_mut_value().set_next_node(self.force_update);
                    backtrack_heuristic
                },
            };
            let mut rng = thread_rng();
            let mut playout = *playout_node.get_value();
            while !playout.game_end_node {
                if start.elapsed() >= time_out {
                    // return tie, if playout cannot finish in time
                    return None;
                }
                match playout.next_node {
                    MonteCarloNodeType::GameDataUpdate => {
                        // create new game game_data update
                        let parent_game_data = playout.game_data;
                        let game_data_update = U::iter_game_data_updates(&playout.game_data, self.force_update).choose(&mut rng).unwrap();
                        playout = playout.new_game_data_update_child(game_data_update);
                        playout.apply_game_data_update(&parent_game_data, false);
                        playout.set_next_node(self.force_update);
                    },
                    MonteCarloNodeType::ActionResult => {
                        // set random next action
                        let parent_game_data = playout.game_data;
                        let parent_action = playout.player_action;
                        let player_action = A::iter_actions(&playout.game_data, playout.player, playout.game_turn).choose(&mut rng).unwrap();
                        playout = playout.new_player_action_child(player_action);
                        playout.apply_action(&parent_game_data, &parent_action, self.game_mode, self.max_number_of_turns, self.use_heuristic_score);
                        playout.set_next_node(self.force_update);
                    },
                }
            }
            Some((playout.calc_playout_score(), backtrack_heuristic))
        }
    }
    fn propagation(&self, start_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>, mut playout_score: f32, backtrack_heuristic: bool) {
        // score playout result and calc new exploitation score for start_node
        start_node.get_mut_value().score_playout_result(playout_score, 1.0, self.use_heuristic_score);
        // backtrack playout_score and heuristic if score event
        for node in start_node.iter_back_track().skip(1).filter(|n| n.get_level() >= self.root_level) {
            // first backtrack heuristic, since heuristic is used by score_playout_result()
            if backtrack_heuristic {
                // ToDo: how to do this with MonteCarloNodeType::GameDataUpdate 
                let player = node.get_value().player;
                match player {
                    MonteCarloPlayer::Me => {
                        let max_beta = node.iter_children().map(|c| c.get_value().beta).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                        node.get_mut_value().alpha = max_beta;
                    },
                    MonteCarloPlayer::Opp => {
                        let min_alpha = node.iter_children().map(|c| c.get_value().alpha).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
                        node.get_mut_value().beta = min_alpha;
                    },
                }
            }
            // do score_playout_result()
            if node.get_value().next_node == MonteCarloNodeType::GameDataUpdate {
                let num_children = node.len_children() as f32;
                playout_score /= num_children;
            }
            // score playout result and calc new exploitation score
            node.get_mut_value().score_playout_result(playout_score, 1.0, self.use_heuristic_score);
        }
    }
    fn reverse_propagation(&self, start_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>, mut wins: f32, mut samples: f32) {
        // remove samples and wins of inconsistent children and calc new exploitation score for start_node
        start_node.get_mut_value().score_playout_result(wins, samples, self.use_heuristic_score);
        for node in start_node.iter_back_track().skip(1).filter(|n| n.get_level() >= self.root_level) {
            if node.get_value().next_node == MonteCarloNodeType::GameDataUpdate {
                let num_children = node.len_children() as f32;
                wins /= num_children;
                samples /= num_children;
            }
            // remove samples and wins of inconsistent children and calc new exploitation score
            node.get_mut_value().score_playout_result(wins, samples, self.use_heuristic_score);
        }
    }
    fn remove_inconsistent_children(&self, selection_node: Rc<TreeNode<MonteCarloNode<G, A, U>>>) -> bool {     
        if self.force_update || selection_node.get_value().next_node == MonteCarloNodeType::ActionResult || selection_node.len_children() == 1 {
            return false;
        }
        let n_children = selection_node.len_children() as f32;
        let mut index = 0;
        let mut samples = 0.0;
        let mut wins = 0.0;
        let mut inconsistency_detected = false;
        while index < selection_node.len_children() {
            let child = selection_node.get_child(index).unwrap();
            // find child with samples
            if !child.get_value().samples.is_nan() {
                samples += child.get_value().samples;
                wins += child.get_value().wins;
                let child_game_data_update = child.get_value().game_data_update;
                if child.get_mut_value().game_data.check_consistency_of_game_data_update(&selection_node.get_value().game_data, &child_game_data_update, self.played_turns) {
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
            // calc inconsistent playout results
            wins = -wins / n_children;
            samples = -samples / n_children;
            let consistent_child_index = selection_node.iter_children().position(|c| !c.get_value().samples.is_nan());
            match consistent_child_index {
                Some(index) => {
                    // If inconsistent children were removed and a child with samples remains, only
                    // this child can be consistent, while all other children are inconsistent.
                    // It's wins and samples are valid and thus not removed by reverse_propagation.
                    wins += selection_node.get_child(index).unwrap().get_value().wins;
                    samples += selection_node.get_child(index).unwrap().get_value().samples;
                    self.reverse_propagation(selection_node.clone(), wins, samples);
                    // remove all other children, since they are inconsistent
                    selection_node.split_off_children(index, true);
                    selection_node.split_off_children(1, false);
                },
                None => {
                    // no consistent child with samples left -> remove all children and reset next node
                    self.reverse_propagation(selection_node.clone(), wins, samples);
                    selection_node.clear_children(0);
                    selection_node.get_mut_value().set_next_node(self.force_update);
                    return true;
                }
            }
        }
        false
    }
}
//⏫my_monte_carlo_tree_search.rs
//⏬my_tic_tac_toe.rs
// This is an example for usage of monte carlo tree search lib
const X: usize = 3;
const Y: usize = X;
const N: usize = X * Y;
#[derive(Copy, Clone, PartialEq, Debug)]
enum TicTacToeStatus {
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
    fn is_player(&self) -> bool {
        match self {
            Self::Player(_) => true,
            _ => false,
        }
    }
}
impl Default for TicTacToeStatus {
    fn default() -> Self {
        TicTacToeStatus::Vacant
    }
}
#[derive(Copy, Clone, Default)]
struct TicTacToeGameData {
    map: MyMap2D<TicTacToeStatus, X, Y, N>,
    status: TicTacToeStatus,
}
impl PartialEq for TicTacToeGameData {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}
impl TicTacToeGameData {
    fn new() -> Self {
        TicTacToeGameData {
            map: MyMap2D::init(TicTacToeStatus::Vacant),
            status: TicTacToeStatus::Vacant,
        }
    }
    fn check_status_for_one_line<'a>(&self, line: impl Iterator<Item = &'a TicTacToeStatus>) -> TicTacToeStatus {
        let mut winner = TicTacToeStatus::Tie;
        for (index, element) in line.enumerate() {
            if index == 0 {
                match element {
                    TicTacToeStatus::Player(player) => winner = TicTacToeStatus::Player(*player),
                    _ => return TicTacToeStatus::Tie,
                }
            } else if winner != *element {
                return TicTacToeStatus::Tie
            }
        }
        winner
    }
    fn check_status(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        // check row with cell.y()
        match self.check_status_for_one_line(self.map.iter_row(cell.y()).map(|(_, v)| v)) {
            TicTacToeStatus::Player(player) => {
                self.status = TicTacToeStatus::Player(player);
                return self.status
            },
            _ => (),
        }
        // check col with cell.x()
        match self.check_status_for_one_line(self.map.iter_column(cell.x()).map(|(_, v)| v)) {
            TicTacToeStatus::Player(player) => {
                self.status = TicTacToeStatus::Player(player);
                return self.status
            },
            _ => (),
        }
        // check neg diag, if cell.x() == cell.y()
        if cell.x() == cell.y() {
            match self.check_status_for_one_line(self.map.iter_diagonale_top_left().map(|(_, v)| v)) {
                TicTacToeStatus::Player(player) => {
                    self.status = TicTacToeStatus::Player(player);
                    return self.status
                },
                _ => (),
            }
        }
        // check pos diag, if cell.x() + cell.y() == 2
        if cell.x() + cell.y() == 2 {
            match self.check_status_for_one_line(self.map.iter_diagonale_top_right().map(|(_, v)| v)) {
                TicTacToeStatus::Player(player) => {
                    self.status = TicTacToeStatus::Player(player);
                    return self.status
                },
                _ => (),
            }
        }
        // set to Tie, if no Vacant left
        if self.map.iter().find(|(_, v)| v.is_vacant()).is_none() {
            self.status = TicTacToeStatus::Tie;
        }
        self.status
    }
    fn calc_line_heuristic<'a>(&self, line: impl Iterator<Item = &'a TicTacToeStatus>) -> f32 {
        let mut count: u8 = 0;
        let mut line_owner: Option<MonteCarloPlayer> = None;
        for cell in line {
            match cell {
                TicTacToeStatus::Vacant => (),
                TicTacToeStatus::Tie => return 0.0,
                TicTacToeStatus::Player(player) => match line_owner {
                    Some(owner) => if *player == owner {
                        count += 1;
                    } else {
                        return 0.0;
                    },
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
            None => 0.0
        }
    }
    fn calc_heuristic_(&self) -> f32 {
        let mut heuristic = 0.0;
        for rc in 0..3 {
            heuristic += self.calc_line_heuristic(self.map.iter_row(rc).map(|(_, v)| v));
            heuristic += self.calc_line_heuristic(self.map.iter_column(rc).map(|(_, v)| v));
        }
        heuristic += self.calc_line_heuristic(self.map.iter_diagonale_top_left().map(|(_, v)| v));
        heuristic += self.calc_line_heuristic(self.map.iter_diagonale_top_right().map(|(_, v)| v));
        heuristic
    }
    fn set_player(&mut self, cell: MapPoint<X, Y>, player: MonteCarloPlayer) -> TicTacToeStatus {
        self.map.set(cell, TicTacToeStatus::Player(player));
        self.check_status(cell)
    }
    fn set_vacant(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        self.map.set(cell, TicTacToeStatus::Vacant);
        self.status = TicTacToeStatus::Vacant;
        self.status
    }
    fn set_tie(&mut self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        self.map.set(cell, TicTacToeStatus::Tie);
        self.check_status(cell)
    }
    fn get_cell_value(&self, cell: MapPoint<X, Y>) -> TicTacToeStatus {
        *self.map.get(cell)
    }
    fn get_first_vacant_cell(&self) -> Option<(MapPoint<X, Y>, &TicTacToeStatus)> {
        self.map.iter().find(|(_, v)| v.is_vacant())
    }
    fn count_player_cells(&self, count_player: MonteCarloPlayer) -> usize {
        self.map.iter().filter(|(_, v)| match v {
            TicTacToeStatus::Player(player) => *player == count_player,
            _ => false,
        }).count()
    }
    fn iter_map(&self) -> impl Iterator<Item = (MapPoint<X, Y>, &TicTacToeStatus)> {
        self.map.iter()
    }
}
//⏫my_tic_tac_toe.rs
//⏬my_compass.rs
#[derive(Clone, Copy, PartialEq)]
enum Compass {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
    Center,
}
impl Default for Compass {
    fn default() -> Self {
        Compass::N
    }
}
impl Compass {
    fn clockwise(&self) -> Self {
        match self {
            Compass::N => Compass::NE,
            Compass::NE => Compass::E,
            Compass::E => Compass::SE,
            Compass::SE => Compass::S,
            Compass::S => Compass::SW,
            Compass::SW => Compass::W,
            Compass::W => Compass::NW,
            Compass::NW => Compass::N,
            Compass::Center => Compass::Center,
        }
    }
    fn counterclockwise(&self) -> Self {
        match self {
            Compass::N => Compass::NW,
            Compass::NW => Compass::W,
            Compass::W => Compass::SW,
            Compass::SW => Compass::S,
            Compass::S => Compass::SE,
            Compass::SE => Compass::E,
            Compass::E => Compass::NE,
            Compass::NE => Compass::N,
            Compass::Center => Compass::Center,
        }
    }
    fn is_center(&self) -> bool {
        *self == Compass::Center
    }
}
//⏫my_compass.rs
//⏬my_tree.rs
use std::cell::RefCell;
use std::rc::Weak;
struct PreOrderTraversal<N> {
    next_node: Rc<TreeNode<N>>,
    child_indices: Vec<usize>, // vector of indices of children while traveling through tree
    vertical: bool, // false: children, true: parent
    iter_finished: bool,
}
impl<'a, N: PartialEq> PreOrderTraversal<N> {
}
impl<'a, N: PartialEq> Iterator for PreOrderTraversal<N> {
    type Item = Rc<TreeNode<N>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.iter_finished {
            return None;
        }
        loop {
            if self.vertical { // in direction of parent
                match self.next_node.get_parent() {
                    Some(node) => {
                        self.child_indices.pop();
                        if self.child_indices.len() == 0 {
                            break; // end of subtree, which started at given "root" node
                        }
                        let last_index = self.child_indices.len() - 1;
                        self.child_indices[last_index] += 1;
                        self.next_node = node;
                        self.vertical = false;
                    },
                    None => break, // end of tree
                } 
            } else { // in direction of children
                if self.child_indices.len() == 0 {
                    self.child_indices.push(0);
                    return Some(self.next_node.clone());
                }
                let child_index = self.child_indices[self.child_indices.len() - 1];
                match self.next_node.get_child(child_index) {
                    Some(node) => {
                        self.next_node = node;
                        self.child_indices.push(0);
                        return Some(self.next_node.clone());
                    },
                    None => self.vertical = true,
                }
            }
        }
        self.iter_finished = true;
        None
    }
}
struct PostOrderTraversal<N> {
    current_node: Rc<TreeNode<N>>,
    child_indices: Vec<usize>, // vector of indices of children while traveling through tree
    vertical: bool, // false: children, true: parent
    finished: bool, // true if iterator finished
}
impl<'a, N: PartialEq> PostOrderTraversal<N> {
}
impl<'a, N: PartialEq> Iterator for PostOrderTraversal<N> {
    type Item = Rc<TreeNode<N>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        loop {
            if self.vertical { // in direction of parent
                let last_index = self.child_indices.len() - 1;
                self.child_indices[last_index] += 1;
                self.vertical = false;
            } else { // in direction of child
                let child_index = self.child_indices[self.child_indices.len() - 1];
                match self.current_node.get_child(child_index) {
                    Some(node) => {
                        self.current_node = node;
                        self.child_indices.push(0);
                    },
                    None => {
                        let result = self.current_node.get_self();
                        match self.current_node.get_parent() {
                            Some(node) => {
                                self.vertical = true;
                                self.child_indices.pop();
                                self.finished = self.child_indices.len() == 0; // root of subtree, which started at given "root" node
                                self.current_node = node;
                            },
                            None => self.finished = true,
                        }
                        return result;
                    },
                }
            }
        }
    }
}
struct LevelOrderTraversal<N> {
    current_node: Rc<TreeNode<N>>,
    child_indices: Vec<usize>, // vector of indices of children while traveling through tree
    vertical: bool, // false: children, true: parent
    finished: bool, // true if iterator finished
    target_level: usize,
    end_level: Option<usize>,
    node_on_target_level: bool,
}
impl<'a, N: PartialEq> LevelOrderTraversal<N> {
    fn new(root: Rc<TreeNode<N>>, start_level: usize, end_level: Option<usize>) -> Self {
        let ci_capacity = match end_level {
            Some(level) => {
                if start_level > level {
                    panic!("end_level must be >= start_level.");
                }
                level + 1
            },
            None => 1,
        };
        let mut child_indices: Vec<usize> = Vec::with_capacity(ci_capacity);
        child_indices.push(0);
        LevelOrderTraversal {
            current_node: root,
            child_indices,
            vertical: false,
            finished: false,
            target_level: start_level,
            end_level,
            node_on_target_level: false,
        }
    }
    fn increment_target_level(&mut self) -> bool {
        match self.end_level {
            Some(level) => if self.target_level == level {
                self.finished = true;
                return true;
            },
            None => (),
        }
        self.target_level += 1;
        false
    }
}
impl<'a, N: PartialEq> Iterator for LevelOrderTraversal<N> {
    type Item = (Rc<TreeNode<N>>, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        loop {
            if self.child_indices.len() - 1 == self.target_level {
                let result = self.current_node.get_self().map(|n| (n, self.target_level));
                if self.target_level == 0 {
                    if self.increment_target_level() {
                        return None;
                    }
                } else {
                    match self.current_node.get_parent() {
                        Some(node) => {
                            self.node_on_target_level = true;
                            self.vertical = true;
                            self.child_indices.pop();
                            self.current_node = node;
                        },
                        None => (),
                    }
                }
                return result;
            }
            if self.vertical { // in direction of parent
                let last_index = self.child_indices.len() - 1;
                self.child_indices[last_index] += 1;
                self.vertical = false;
            } else { // in direction of child
                let child_index = self.child_indices[self.child_indices.len() - 1];
                match self.current_node.get_child(child_index) {
                    Some(node) => {
                        self.current_node = node;
                        self.child_indices.push(0);
                    },
                    None => {
                        if self.child_indices.len() == 1 { // root of sub tree
                            if self.node_on_target_level {
                                if self.increment_target_level() {
                                    return None;
                                }
                                self.node_on_target_level = false;
                                self.child_indices[0] = 0; // reset index
                            } else {
                                // no more childs of root to search for target_level
                                self.finished = true;
                                return None;
                            }
                        } else {
                            match self.current_node.get_parent() {
                                Some(node) => {
                                    self.vertical = true;
                                    self.child_indices.pop();
                                    self.current_node = node;
                                },
                                None => (),
                            }
                        }
                    },
                }
            }
        }
    }
}
struct BackTrack<N> {
    current_node: Rc<TreeNode<N>>,
    finished: bool, // true if iterator finished
}
impl<'a, N: PartialEq> BackTrack<N> {
    fn new(node: Rc<TreeNode<N>>) -> Self {
        BackTrack {
            current_node: node,
            finished: false,
        }
    }
}
impl<'a, N: PartialEq> Iterator for BackTrack<N> {
    type Item = Rc<TreeNode<N>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        let result = self.current_node.get_self();
        match self.current_node.get_parent() {
            Some(node) => self.current_node = node,
            None => self.finished = true,
        }
        result
    }
}
struct IterChildren<N> {
    node: Rc<TreeNode<N>>,
    len_children: usize,
    child_index: usize,
    finished: bool, // true if iterator finished
}
impl<'a, N: PartialEq> IterChildren<N> {
    fn new(node: Rc<TreeNode<N>>) -> Self {
        let len_children = node.len_children();
        IterChildren {
            node,
            len_children,
            child_index: 0,
            finished: false,
        }
    }
}
impl<'a, N: PartialEq> Iterator for IterChildren<N> {
    type Item = Rc<TreeNode<N>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        match self.node.get_child(self.child_index) {
            Some(node) => {
                self.child_index += 1;
                Some(node)
            },
            None => {
                self.finished = true;
                None
            },
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len_children))
    }
}
impl<'a, N: PartialEq + Copy + Clone> ExactSizeIterator for IterChildren<N> {
    fn len(&self) -> usize {
        self.len_children
    }
}
struct IterSelf<N> {
    node: Rc<TreeNode<N>>,
    finished: bool, // true if iterator finished
}
impl<'a, N: PartialEq> IterSelf<N> {
}
impl<'a, N: PartialEq> Iterator for IterSelf<N> {
    type Item = Rc<TreeNode<N>>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None; // iterator finished
        }
        self.finished = true;
        Some(self.node.clone())
    }
}
struct TreeNode<N> {
    value: RefCell<N>,
    level: usize,
    node: RefCell<Weak<TreeNode<N>>>,
    parent: RefCell<Weak<TreeNode<N>>>,
    children: RefCell<Vec<Rc<TreeNode<N>>>>,
}
impl<N: PartialEq> TreeNode<N> {
    fn seed_root(value: N, children_capacity: usize)  -> Rc<TreeNode<N>> {
        TreeNode::new(value, 0, children_capacity)
    }
    fn new(value: N, level: usize, children_capacity: usize) -> Rc<TreeNode<N>> {
        let result = Rc::new(TreeNode {
            value: RefCell::new(value),
            level,
            node: RefCell::new(Weak::new()), // weak reference on itself!
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::with_capacity(children_capacity)),
        });
        let node = Rc::downgrade(&result);
        *result.node.borrow_mut() = node;
        result
    }
    fn add_child(&self, value: N, children_capacity: usize) -> Rc<TreeNode<N>> { 
        match self.iter_children().find(|n| *n.value.borrow() == value) {
            Some(node) => node,
            None => {
                let child = TreeNode::new(value, self.level + 1, children_capacity);
                *child.parent.borrow_mut() = self.node.borrow().clone();
                self.children.borrow_mut().push(child.clone());
                child
            }
        }
    }
    fn swap_remove_child(&self, index: usize) -> Option<Rc<TreeNode<N>>> {
        if index >= self.len_children() {
            return None;
        }
        let result = self.children.borrow_mut().swap_remove(index);
        Some(result)
    }
    fn split_off_children(&self, at: usize, keep_split_off: bool) {
        let split_off = self.children.borrow_mut().split_off(at);
        if keep_split_off {
            *self.children.borrow_mut() = split_off;
        }
    }
    fn clear_children(&self, children_capacity: usize) {
        *self.children.borrow_mut() = Vec::with_capacity(children_capacity);
    }
    fn get_value(&self) -> std::cell::Ref<'_, N> {
        self.value.borrow()
    }
    fn get_mut_value(&self) -> std::cell::RefMut<'_, N> {
        self.value.borrow_mut()
    }
    fn get_level(&self) -> usize {
        self.level
    }
    fn get_self(&self) -> Option<Rc<TreeNode<N>>> {
        match self.node.borrow().upgrade() {
            Some(ref node) => Some(node.clone()),
            None => None,
        }
    }
    fn get_child(&self, index: usize) -> Option<Rc<TreeNode<N>>> {
        match self.children.borrow().get(index) {
            Some(ref node) => Some((*node).clone()),
            None => None,
        }
    }
    fn len_children(&self) -> usize {
        self.children.borrow().len()
    }
    fn get_parent(&self) -> Option<Rc<TreeNode<N>>> {
        match self.parent.borrow().upgrade() {
            Some(ref node) => Some(node.clone()),
            None => None,
        }
    }
    fn is_leave(&self) -> bool {
        self.len_children() == 0
    }
    fn iter_children(&self) -> impl Iterator<Item = Rc<TreeNode<N>>> {
        IterChildren::new(self.get_self().unwrap())
    }
    fn iter_back_track(&self) -> impl Iterator<Item = Rc<TreeNode<N>>> {
        BackTrack::new(self.get_self().unwrap())
    }
    // second return value is level of node relative to start node, from which iter_level_order_traversal() was called
    fn iter_level_order_traversal_with_bordes(&self, start_level: usize, end_level: Option<usize>) -> impl Iterator<Item = (Rc<TreeNode<N>>, usize)> {
        LevelOrderTraversal::new(self.get_self().unwrap(), start_level, end_level)
    }
}
//⏫my_tree.rs
//⏬main.rs
use std::io;
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
            // if Me is startplayer, only choose cells from big middle cell
            result.player_action.ult_ttt_big = MapPoint::<X, Y>::new(1, 1);
            result.player_action.ult_ttt_small = MapPoint::<X, Y>::new(0, 0);
            result.next_action_square_is_specified = true;
        } else if ult_ttt_data.next_action_square_is_specified {
            result.player_action.ult_ttt_big = ult_ttt_data.player_action.ult_ttt_small;
            result.player_action.ult_ttt_small = ult_ttt_data.map.get(result.player_action.ult_ttt_big).get_first_vacant_cell().unwrap().0;
            result.next_action_square_is_specified = true;
        } else {
            match result.ult_ttt_data.status_map.get_first_vacant_cell() {
                Some((new_iter_ttt_big, _)) => {
                    result.player_action.ult_ttt_big = new_iter_ttt_big;
                    result.player_action.ult_ttt_small = ult_ttt_data.map.get(new_iter_ttt_big).get_first_vacant_cell().unwrap().0;
                },
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
        self.player_action.id += 1;
        let mut searching_new_iter_ttt_big = true;
        while searching_new_iter_ttt_big {
            if self.ult_ttt_data.status_map.get_cell_value(self.player_action.ult_ttt_big).is_vacant() {
                let mut searching_new_iter_ttt_small = true;
                while searching_new_iter_ttt_small {
                    match self.player_action.ult_ttt_small.forward_x() {
                        Some(new_iter_ttt_small) => {
                            self.player_action.ult_ttt_small = new_iter_ttt_small;
                            if self.ult_ttt_data.map.get(self.player_action.ult_ttt_big).get_cell_value(self.player_action.ult_ttt_small).is_vacant() {
                                return Some(result);
                            }
                        },
                        None => {
                            if self.next_action_square_is_specified {
                                self.iter_finished = true;
                                return Some(result);
                            }
                            self.player_action.ult_ttt_small = MapPoint::<X, Y>::new(0, 0);
                            searching_new_iter_ttt_small = false;
                        },
                    }
                }
            }
            match self.player_action.ult_ttt_big.forward_x() {
                Some(new_iter_ttt_big) => {
                    self.player_action.ult_ttt_big = new_iter_ttt_big;
                    if self.ult_ttt_data.status_map.get_cell_value(self.player_action.ult_ttt_big).is_vacant() &&
                       self.ult_ttt_data.map.get(self.player_action.ult_ttt_big).get_cell_value(self.player_action.ult_ttt_small).is_vacant() {
                        return Some(result);
                    }
                },
                None => {
                    self.iter_finished = true;
                    searching_new_iter_ttt_big = false;
                }
            }
        }
        Some(result)
    }
}
#[derive(Copy, Clone, Default)]
struct UltTTTPlayerAction {
    id: usize,
    ult_ttt_big: MapPoint<X, Y>,
    ult_ttt_small: MapPoint<X, Y>,
}
impl PartialEq for UltTTTPlayerAction {
    fn eq(&self, other: &Self) -> bool {
        self.ult_ttt_big == other.ult_ttt_big &&
        self.ult_ttt_small == other.ult_ttt_small
    }
}
impl UltTTTPlayerAction {
    fn from_ext(&mut self, extern_coordinates: MapPoint<U, V>) {
        self.ult_ttt_big = MapPoint::<X, Y>::new(extern_coordinates.x()/X, extern_coordinates.y()/Y);
        self.ult_ttt_small = MapPoint::<X, Y>::new(extern_coordinates.x()%X, extern_coordinates.y()%Y);
        self.id = 0;
    }
    fn to_ext(&self) -> MapPoint<U, V> {
        MapPoint::<U, V>::new(self.ult_ttt_big.x()*X + self.ult_ttt_small.x(), self.ult_ttt_big.y()*Y + self.ult_ttt_small.y())
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
    fn iter_actions(game_data: &impl MonteCarloGameData, player: MonteCarloPlayer, parent_game_turn: usize) -> Box<dyn Iterator<Item=Self> + '_> {
        let game_data = UltTTT::downcast_self(game_data);
        Box::new(IterUltTTT::new(game_data, player, parent_game_turn))
    }
}
#[derive(Copy, Clone, PartialEq, Default)]
struct UltTTTGameDataUpdate {}
impl MonteCarloGameDataUpdate for UltTTTGameDataUpdate {
    fn downcast_self(_game_data_update: &impl MonteCarloGameDataUpdate) -> &Self {
        &UltTTTGameDataUpdate{}
    }
    fn iter_game_data_updates(_game_data: &impl MonteCarloGameData, _force_update: bool) -> Box<dyn Iterator<Item=Self> + '_> {
        Box::new(vec![].into_iter())
    }
}
#[derive(Copy, Clone, Default)]
struct UltTTT {
    map: MyMap2D<TicTacToeGameData, X, Y, N>,
    status_map: TicTacToeGameData,
    status: TicTacToeStatus,
    player_action: UltTTTPlayerAction,
    next_action_square_is_specified: bool,
}
impl PartialEq for UltTTT {
    fn eq(&self, other: &Self) -> bool {
        self.player_action == other.player_action
    }
}
impl UltTTT {
    fn new() -> Self {
        UltTTT {
            map: MyMap2D::new(),
            status_map: TicTacToeGameData::new(),
            status: TicTacToeStatus::Vacant,
            player_action: UltTTTPlayerAction::default(),
            next_action_square_is_specified: false,
        }
    }
    fn set_last_opp_action(&mut self, opp_action: MapPoint<U, V>) -> TicTacToeStatus {
        self.player_action.from_ext(opp_action);
        self.execute_player_action(MonteCarloPlayer::Opp)
    }
    fn execute_player_action(&mut self, player: MonteCarloPlayer) -> TicTacToeStatus {
        let status = self.map.get_mut(self.player_action.ult_ttt_big).set_player(self.player_action.ult_ttt_small, player);
        self.status = match status {
            TicTacToeStatus::Vacant => self.status_map.set_vacant(self.player_action.ult_ttt_big),
            TicTacToeStatus::Player(winner) => self.status_map.set_player(self.player_action.ult_ttt_big, winner),
            TicTacToeStatus::Tie => self.status_map.set_tie(self.player_action.ult_ttt_big),
        };
        if self.status == TicTacToeStatus::Tie {
            // game finished without direct winner
            // count for each player number of won sqares; most squares won wins game
            let my_squares = self.status_map.count_player_cells(MonteCarloPlayer::Me);
            let opp_squares = self.status_map.count_player_cells(MonteCarloPlayer::Opp);
            self.status = match my_squares.cmp(&opp_squares) {
                Ordering::Greater => TicTacToeStatus::Player(MonteCarloPlayer::Me),
                Ordering::Less => TicTacToeStatus::Player(MonteCarloPlayer::Opp),
                Ordering::Equal => TicTacToeStatus::Tie,
            };
        }
        self.next_action_square_is_specified = self.status_map.get_cell_value(self.player_action.ult_ttt_small) == TicTacToeStatus::Vacant;
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
        self.player_action = *UltTTTPlayerAction::downcast_self(player_action);
        // game end or cell won/lost/tie
        self.execute_player_action(MonteCarloPlayer::Me).is_not_vacant() || self.status_map.get_cell_value(self.player_action.ult_ttt_big).is_player()
    }
    fn apply_opp_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool {
        self.player_action = *UltTTTPlayerAction::downcast_self(player_action);
        // game end or cell won/lost/tie
        self.execute_player_action(MonteCarloPlayer::Opp).is_not_vacant() || self.status_map.get_cell_value(self.player_action.ult_ttt_big).is_player()
    }
    fn simultaneous_player_actions_for_simultaneous_game_data_change(&mut self, _my_action: &impl MonteCarloPlayerAction, _opp_action: &impl MonteCarloPlayerAction) {
        // no random game_data updates for TicTacToe
    }
    fn apply_game_data_update(&mut self, _game_data_update: &impl MonteCarloGameDataUpdate, _check_update_consistency: bool) -> bool {
        false
    }
    fn is_game_data_update_required(&self, _force_update: bool) -> bool {
        false
    }
    fn calc_heuristic(&self) -> f32 {
        self.status_map.calc_heuristic_() * 10.0 + self.status_map.iter_map().map(|(_, s)| match s {
            TicTacToeStatus::Player(player) => match player {
                MonteCarloPlayer::Me => 1.0,
                MonteCarloPlayer::Opp => -1.0,
            },
            _ => 0.0,
        }).sum::<f32>()
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
    fn check_consistency_of_game_data_during_init_root(&mut self, _current_game_state: &Self, _played_turns: usize) -> bool {
        //dummy
        true
    }
    fn check_consistency_of_game_data_update(&mut self, _current_game_state: &Self, _game_data_update: &impl MonteCarloGameDataUpdate, _played_turns: usize) -> bool {
        //dummy
        true
    }
    fn check_consistency_of_action_result(&mut self, _current_game_state: Self, _my_action: &impl MonteCarloPlayerAction, _opp_action: &impl MonteCarloPlayerAction, _played_turns: usize, _apply_player_actions_to_game_data: bool) -> bool {
        //dummy
        true
    }
}
macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
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
    let debug = true;
    let keep_root = false;
    let mut mcts:MonteCarloTreeSearch<UltTTT, UltTTTPlayerAction, UltTTTGameDataUpdate> = MonteCarloTreeSearch::new(game_mode, max_number_of_turns, force_update, time_out_first_turn, time_out_successive_turns, weighting_factor, use_heuristic_score, debug, keep_root);
    // game loop
    loop {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(" ").collect::<Vec<_>>();
        let opponent_row = parse_input!(inputs[0], i32);
        let opponent_col = parse_input!(inputs[1], i32);
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let valid_action_count = parse_input!(input_line, i32);
        for _i in 0..valid_action_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let _row = parse_input!(inputs[0], i32);
            let _col = parse_input!(inputs[1], i32);
        }
        if turn_counter == 1 {
            // check startplayer
            if opponent_row >= 0 {
                starting_player = MonteCarloPlayer::Opp;
                turn_counter += 1;
                let opp_action = MapPoint::<U, V>::new(opponent_col as usize, opponent_row as usize);
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
        let (my_game_data, my_action) = mcts.choose_and_execute_actions();
        game_data = *UltTTT::downcast_self(&my_game_data);
        let my_action = UltTTTPlayerAction::downcast_self(&my_action);
        my_action.execute_action();
        turn_counter += 2;
    }
}
//⏫main.rs