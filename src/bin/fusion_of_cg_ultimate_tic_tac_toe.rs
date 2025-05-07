use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;
type PWDefaultTTT = PWDefault<UltTTTMCTSGame>;
macro_rules! parse_input {
    ($ x : expr , $ t : ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}
fn main() {
    let weighting_factor = 1.4;
    let time_out_first_turn = Duration::from_millis(990);
    let time_out_successive_turns = Duration::from_millis(90);
    let time_out_codingame_input = Duration::from_millis(2000);
    let mut game_data = UltTTT::new();
    let mut mcts_ult_ttt: PlainMCTS<
        UltTTTMCTSGame,
        DynamicC,
        CachedUTC,
        PWDefaultTTT,
        UltTTTHeuristic,
        UltTTTSimulationPolicy,
    > = PlainMCTS::new(weighting_factor);
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
        game_data.set_current_player(TicTacToeStatus::Opp);
        let opp_action = (opponent_col as u8, opponent_row as u8);
        game_data = UltTTTMCTSGame::apply_move(
            &game_data,
            &UltTTTMove::from(opp_action),
            &mut mcts_ult_ttt.game_cache,
        );
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
        let selected_move = *mcts_ult_ttt.select_move();
        game_data =
            UltTTTMCTSGame::apply_move(&game_data, &selected_move, &mut mcts_ult_ttt.game_cache);
        selected_move.execute_action();
        mcts_ult_ttt.set_root(&game_data);
        let start = Instant::now();
        number_of_iterations = 0;
        loop {
            match rx.try_recv() {
                Ok((opponent_row, opponent_col)) => {
                    let opp_action = (opponent_col as u8, opponent_row as u8);
                    game_data = UltTTTMCTSGame::apply_move(
                        &game_data,
                        &UltTTTMove::from(opp_action),
                        &mut mcts_ult_ttt.game_cache,
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
    }
}
use std::cmp::Ordering;
use std::fmt::Write;
#[derive(Copy, Clone, PartialEq, Default)]
struct UltTTTMove {
    status_index: CellIndex3x3,
    mini_board_index: CellIndex3x3,
}
impl From<(u8, u8)> for UltTTTMove {
    fn from(cg_coordinates: (u8, u8)) -> UltTTTMove {
        let x_status = cg_coordinates.0 / 3;
        let y_status = cg_coordinates.1 / 3;
        let x_mini_board = cg_coordinates.0 % 3;
        let y_mini_board = cg_coordinates.1 % 3;
        UltTTTMove {
            status_index: CellIndex3x3::try_from((x_status, y_status)).unwrap(),
            mini_board_index: CellIndex3x3::try_from((x_mini_board, y_mini_board)).unwrap(),
        }
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
    fn valid_move(
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
    fn execute_action(&self) -> String {
        let mut action_commando_string = String::new();
        let action = <(u8, u8)>::from(*self);
        write!(action_commando_string, "{} {}", action.1, action.0).unwrap();
        println!("{}", action_commando_string);
        action_commando_string
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
enum NextActionConstraint {
    #[default]
    Init,
    None,
    MiniBoard(CellIndex3x3),
}
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
struct UltTTT {
    map: MyMap3x3<TicTacToeGameData>,
    status_map: TicTacToeGameData,
    next_action_constraint: NextActionConstraint,
    current_player: TicTacToeStatus,
}
impl UltTTT {
    fn new() -> Self {
        UltTTT {
            map: MyMap3x3::new(),
            status_map: TicTacToeGameData::new(),
            next_action_constraint: NextActionConstraint::Init,
            current_player: TicTacToeStatus::Me,
        }
    }
    fn set_current_player(&mut self, player: TicTacToeStatus) {
        self.current_player = player;
    }
    fn next_player(&mut self) {
        self.current_player = self.current_player.next();
    }
    fn execute_player_move(
        &mut self,
        player_move: UltTTTMove,
        player: TicTacToeStatus,
        _game_cache: &mut NoGameCache<UltTTT, UltTTTMove>,
    ) {
        self.map
            .get_cell_mut(player_move.status_index)
            .set_cell_value(player_move.mini_board_index, player);
        let status = self
            .map
            .get_cell(player_move.status_index)
            .get_status_increment(&player_move.mini_board_index);
        if !status.is_vacant() {
            self.status_map
                .set_cell_value(player_move.status_index, status);
        }
        self.next_action_constraint = if self
            .status_map
            .get_cell_value(player_move.mini_board_index)
            .is_vacant()
        {
            NextActionConstraint::MiniBoard(player_move.mini_board_index)
        } else {
            NextActionConstraint::None
        };
    }
}
struct UltTTTMCTSGame {}
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
        game_cache: &mut Self::Cache,
    ) -> Self::State {
        let mut new_state = *state;
        new_state.execute_player_move(*mv, state.current_player, game_cache);
        new_state.next_player();
        new_state
    }
    fn evaluate(state: &Self::State, _game_cache: &mut Self::Cache) -> Option<f32> {
        let mut status = state.status_map.get_status();
        if status == TicTacToeStatus::Tie {
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
struct UltTTTHeuristic {}
impl Heuristic<UltTTTMCTSGame> for UltTTTHeuristic {
    type Cache = NoHeuristicCache<UltTTT, UltTTTMove>;
    fn evaluate_state(
        state: &<UltTTTMCTSGame as MCTSGame>::State,
        game_cache: &mut <UltTTTMCTSGame as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
    ) -> f32 {
        if let Some(cache_value) = heuristic_cache.get_intermediate_score(state) {
            return cache_value;
        }
        let heuristic = match UltTTTMCTSGame::evaluate(state, game_cache) {
            Some(value) => value,
            None => {
                let my_wins = state.status_map.count_me_cells() as f32;
                let opp_wins = state.status_map.count_opp_cells() as f32;
                let mut my_threats = 0.0;
                let mut opp_threats = 0.0;
                for (status_index, _) in state.status_map.iter_map().filter(|(_, c)| c.is_vacant())
                {
                    let (my, opp) = state.map.get_cell(status_index).get_threats();
                    my_threats += my as f32;
                    opp_threats += opp as f32;
                }
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
        _state: &<UltTTTMCTSGame as MCTSGame>::State,
        _mv: &<UltTTTMCTSGame as MCTSGame>::Move,
        _game_cache: &mut <UltTTTMCTSGame as MCTSGame>::Cache,
        _heuristic_cache: &mut Self::Cache,
    ) -> f32 {
        0.0
    }
}
type UltTTTSimulationPolicy = HeuristicCutoff<20>;
use std::convert::TryFrom;
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum CellIndex3x3 {
    #[default]
    TL = 0,
    TM = 1,
    TR = 2,
    ML = 3,
    MM = 4,
    MR = 5,
    BL = 6,
    BM = 7,
    BR = 8,
}
impl From<CellIndex3x3> for usize {
    fn from(cell: CellIndex3x3) -> Self {
        cell as usize
    }
}
impl TryFrom<usize> for CellIndex3x3 {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CellIndex3x3::TL),
            1 => Ok(CellIndex3x3::TM),
            2 => Ok(CellIndex3x3::TR),
            3 => Ok(CellIndex3x3::ML),
            4 => Ok(CellIndex3x3::MM),
            5 => Ok(CellIndex3x3::MR),
            6 => Ok(CellIndex3x3::BL),
            7 => Ok(CellIndex3x3::BM),
            8 => Ok(CellIndex3x3::BR),
            _ => Err(()),
        }
    }
}
impl TryFrom<(u8, u8)> for CellIndex3x3 {
    type Error = ();
    fn try_from(value: (u8, u8)) -> Result<Self, Self::Error> {
        match value {
            (0, 0) => Ok(CellIndex3x3::TL),
            (0, 1) => Ok(CellIndex3x3::TM),
            (0, 2) => Ok(CellIndex3x3::TR),
            (1, 0) => Ok(CellIndex3x3::ML),
            (1, 1) => Ok(CellIndex3x3::MM),
            (1, 2) => Ok(CellIndex3x3::MR),
            (2, 0) => Ok(CellIndex3x3::BL),
            (2, 1) => Ok(CellIndex3x3::BM),
            (2, 2) => Ok(CellIndex3x3::BR),
            _ => Err(()),
        }
    }
}
impl From<CellIndex3x3> for (u8, u8) {
    fn from(cell: CellIndex3x3) -> Self {
        match cell {
            CellIndex3x3::TL => (0, 0),
            CellIndex3x3::TM => (0, 1),
            CellIndex3x3::TR => (0, 2),
            CellIndex3x3::ML => (1, 0),
            CellIndex3x3::MM => (1, 1),
            CellIndex3x3::MR => (1, 2),
            CellIndex3x3::BL => (2, 0),
            CellIndex3x3::BM => (2, 1),
            CellIndex3x3::BR => (2, 2),
        }
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
struct MyMap3x3<T> {
    cells: [T; 9],
}
impl<T: Default + Clone + Copy> MyMap3x3<T> {
    fn new() -> Self {
        MyMap3x3 {
            cells: [T::default(); 9],
        }
    }
    fn init(value: T) -> Self {
        MyMap3x3 { cells: [value; 9] }
    }
    fn get_cell(&self, index: CellIndex3x3) -> &T {
        &self.cells[usize::from(index)]
    }
    fn get_cell_mut(&mut self, index: CellIndex3x3) -> &mut T {
        &mut self.cells[usize::from(index)]
    }
    fn set_cell(&mut self, index: CellIndex3x3, value: T) {
        self.cells[usize::from(index)] = value;
    }
    fn iterate(&self) -> impl Iterator<Item = (CellIndex3x3, &T)> {
        self.cells
            .iter()
            .enumerate()
            .map(|(i, cell)| (CellIndex3x3::try_from(i).unwrap(), cell))
    }
}
use rand::prelude::IteratorRandom;
struct PlainMCTS<G, UP, UC, EP, H, SP>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
    nodes: Vec<PlainNode<G, UP, UC, EP, H>>,
    root_index: usize,
    exploration_constant: f32,
    game_cache: G::Cache,
    heuristic_cache: H::Cache,
    phantom: std::marker::PhantomData<SP>,
}
impl<G, UP, UC, EP, H, SP> PlainMCTS<G, UP, UC, EP, H, SP>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
    fn new(exploration_constant: f32) -> Self {
        Self {
            nodes: vec![],
            root_index: 0,
            exploration_constant,
            game_cache: G::Cache::new(),
            heuristic_cache: H::Cache::new(),
            phantom: std::marker::PhantomData,
        }
    }
}
impl<G, UP, UC, EP, H, SP> MCTSAlgo<G> for PlainMCTS<G, UP, UC, EP, H, SP>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
    fn iterate(&mut self) {
        let mut path = vec![self.root_index];
        let mut current_index = self.root_index;
        while !self.nodes[current_index].get_children().is_empty() {
            let parent_visits = self.nodes[current_index].get_visits();
            let num_parent_children = self.nodes[current_index].get_children().len();
            if self.nodes[current_index]
                .expansion_policy
                .should_expand(parent_visits, num_parent_children)
            {
                break;
            }
            let mut best_child_index = 0;
            let mut best_utc = f32::NEG_INFINITY;
            for vec_index in 0..num_parent_children {
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
        let current_index =
            if G::evaluate(self.nodes[current_index].get_state(), &mut self.game_cache).is_some()
                || self.nodes[current_index].get_visits() == 0
            {
                current_index
            } else {
                let visits = self.nodes[current_index].get_visits();
                let num_parent_children = self.nodes[current_index].get_children().len();
                while let Some(mv) = self.nodes[current_index]
                    .expansion_policy
                    .pop_expandable_move(visits, num_parent_children)
                {
                    let new_state = G::apply_move(
                        self.nodes[current_index].get_state(),
                        &mv,
                        &mut self.game_cache,
                    );
                    let expansion_policy = EP::new(&new_state, &mut self.game_cache);
                    let new_node = PlainNode::new(new_state, mv, expansion_policy);
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
        let mut depth = 0;
        let simulation_result = loop {
            if let Some(final_score) = G::evaluate(&current_state, &mut self.game_cache) {
                break final_score;
            }
            if let Some(heuristic) = SP::should_cutoff(
                &current_state,
                depth,
                &mut self.game_cache,
                &mut self.heuristic_cache,
            ) {
                break heuristic;
            }
            current_state = G::apply_move(
                &current_state,
                &G::available_moves(&current_state)
                    .choose(&mut rand::thread_rng())
                    .expect("No available moves"),
                &mut self.game_cache,
            );
            depth += 1;
        };
        for &node_index in path.iter().rev() {
            self.nodes[node_index].update_stats(simulation_result);
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
        let expansion_policy = EP::new(state, &mut self.game_cache);
        self.nodes
            .push(PlainNode::root_node(state.clone(), expansion_policy));
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
use rand::prelude::SliceRandom;
struct NoGameCache<State, Move> {
    phantom: std::marker::PhantomData<(State, Move)>,
}
impl<State, Move> GameCache<State, Move> for NoGameCache<State, Move> {
    fn new() -> Self {
        NoGameCache {
            phantom: std::marker::PhantomData,
        }
    }
}
struct DynamicC {}
impl<G: MCTSGame> UCTPolicy<G> for DynamicC {
    fn exploration_score(visits: usize, parent_visits: usize, c: f32) -> f32 {
        let dynamic_c = c / (1.0 + (visits as f32).sqrt());
        dynamic_c * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}
struct CachedUTC {
    exploitation: f32,
    exploration: f32,
    last_parent_visits: usize,
}
impl<G: MCTSGame, UP: UCTPolicy<G>> UTCCache<G, UP> for CachedUTC {
    fn new() -> Self {
        CachedUTC {
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
            UP::exploitation_score(acc_value, visits, current_player, perspective_player);
    }
    fn get_exploitation(&self, _v: usize, _a: f32, _c: G::Player, _p: G::Player) -> f32 {
        self.exploitation
    }
    fn update_exploration(&mut self, visits: usize, parent_visits: usize, base_c: f32) {
        if self.last_parent_visits != parent_visits {
            self.exploration = UP::exploration_score(visits, parent_visits, base_c);
            self.last_parent_visits = parent_visits;
        }
    }
    fn get_exploration(&self, _v: usize, _p: usize, _b: f32) -> f32 {
        self.exploration
    }
}
struct ExpandAll<G: MCTSGame> {
    moves: Vec<G::Move>,
}
impl<G: MCTSGame> ExpansionPolicy<G> for ExpandAll<G> {
    fn new(state: &<G as MCTSGame>::State, game_cache: &mut <G as MCTSGame>::Cache) -> Self {
        let moves = if game_cache.get_terminal_value(state).is_some() {
            vec![]
        } else {
            G::available_moves(state).collect::<Vec<_>>()
        };
        ExpandAll { moves }
    }
    fn should_expand(&self, _v: usize, _n: usize) -> bool {
        !self.moves.is_empty()
    }
    fn pop_expandable_move(&mut self, _v: usize, _n: usize) -> Option<<G as MCTSGame>::Move> {
        self.moves.pop()
    }
}
struct ProgressiveWidening<const C: usize, const AN: usize, const AD: usize, G: MCTSGame> {
    unexpanded_moves: Vec<G::Move>,
}
type PWDefault<G> = ProgressiveWidening<2, 1, 2, G>;
impl<const C: usize, const AN: usize, const AD: usize, G: MCTSGame>
    ProgressiveWidening<C, AN, AD, G>
{
    fn allowed_children(visits: usize) -> usize {
        if visits == 0 {
            1
        } else {
            (C as f32 * (visits as f32).powf(AN as f32 / AD as f32)).floor() as usize
        }
    }
}
impl<const C: usize, const AN: usize, const AD: usize, G: MCTSGame> ExpansionPolicy<G>
    for ProgressiveWidening<C, AN, AD, G>
{
    fn new(state: &<G as MCTSGame>::State, game_cache: &mut <G as MCTSGame>::Cache) -> Self {
        let unexpanded_moves = if game_cache.get_terminal_value(state).is_some() {
            vec![]
        } else {
            let mut unexpanded_moves = G::available_moves(state).collect::<Vec<_>>();
            unexpanded_moves.shuffle(&mut rand::thread_rng());
            unexpanded_moves
        };
        ProgressiveWidening { unexpanded_moves }
    }
    fn should_expand(&self, visits: usize, num_parent_children: usize) -> bool {
        num_parent_children < Self::allowed_children(visits) && !self.unexpanded_moves.is_empty()
    }
    fn pop_expandable_move(
        &mut self,
        visits: usize,
        num_parent_children: usize,
    ) -> Option<<G as MCTSGame>::Move> {
        if !self.should_expand(visits, num_parent_children) {
            return None;
        }
        self.unexpanded_moves.pop()
    }
}
struct HeuristicCutoff<const MXD: usize> {}
impl<const MXD: usize, G: MCTSGame, H: Heuristic<G>> SimulationPolicy<G, H>
    for HeuristicCutoff<MXD>
{
    fn should_cutoff(
        state: &G::State,
        depth: usize,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut H::Cache,
    ) -> Option<f32> {
        let heuristic = H::evaluate_state(state, game_cache, heuristic_cache);
        if depth >= MXD || heuristic <= 0.05 || heuristic >= 0.95 {
            Some(heuristic)
        } else {
            None
        }
    }
}
struct NoHeuristicCache<State, Move> {
    phantom: std::marker::PhantomData<(State, Move)>,
}
impl<State, Move> HeuristicCache<State, Move> for NoHeuristicCache<State, Move> {
    fn new() -> Self {
        NoHeuristicCache {
            phantom: std::marker::PhantomData,
        }
    }
}
struct NoHeuristic {}
impl<G: MCTSGame> Heuristic<G> for NoHeuristic {
    type Cache = NoHeuristicCache<G::State, G::Move>;
    fn evaluate_state(
        state: &<G as MCTSGame>::State,
        game_cache: &mut <G as MCTSGame>::Cache,
        _heuristic_cache: &mut Self::Cache,
    ) -> f32 {
        G::evaluate(state, game_cache).unwrap_or(0.5)
    }
    fn evaluate_move(
        _state: &<G as MCTSGame>::State,
        _mv: &<G as MCTSGame>::Move,
        _game_cache: &mut <G as MCTSGame>::Cache,
        _heuristic_cache: &mut Self::Cache,
    ) -> f32 {
        0.0
    }
}
struct PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G>,
    H: Heuristic<G>,
{
    state: G::State,
    visits: usize,
    accumulated_value: f32,
    mv: Option<G::Move>,
    children: Vec<usize>,
    utc_cache: UC,
    expansion_policy: EP,
    phantom: std::marker::PhantomData<(UP, H)>,
}
impl<G, UP, UC, EP, H> PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G>,
    H: Heuristic<G>,
{
    fn root_node(state: G::State, expansion_policy: EP) -> Self {
        PlainNode {
            expansion_policy,
            state,
            visits: 0,
            accumulated_value: 0.0,
            mv: None,
            children: vec![],
            utc_cache: UC::new(),
            phantom: std::marker::PhantomData,
        }
    }
    fn new(state: G::State, mv: G::Move, expansion_policy: EP) -> Self {
        PlainNode {
            expansion_policy,
            state,
            visits: 0,
            accumulated_value: 0.0,
            mv: Some(mv),
            children: vec![],
            utc_cache: UC::new(),
            phantom: std::marker::PhantomData,
        }
    }
    fn add_child(&mut self, child_index: usize) {
        self.children.push(child_index);
    }
    fn get_children(&self) -> &Vec<usize> {
        &self.children
    }
}
impl<G, UP, UC, EP, H> MCTSNode<G> for PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G>,
    H: Heuristic<G>,
{
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
    fn update_stats(&mut self, result: f32) {
        self.visits += 1;
        self.accumulated_value += result;
        self.utc_cache.update_exploitation(
            self.visits,
            self.accumulated_value,
            G::current_player(&self.state),
            G::perspective_player(),
        );
    }
    fn calc_utc(&mut self, parent_visits: usize, c: f32, perspective_player: G::Player) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let exploitation = self.utc_cache.get_exploitation(
            self.visits,
            self.accumulated_value,
            G::current_player(&self.state),
            perspective_player,
        );
        self.utc_cache
            .update_exploration(self.visits, parent_visits, c);
        let exploration = self
            .utc_cache
            .get_exploration(self.visits, parent_visits, c);
        exploitation + exploration
    }
}
trait MCTSPlayer: PartialEq {
    fn next(&self) -> Self;
}
trait GameCache<State, Move> {
    fn new() -> Self;
    fn get_applied_state(&self, _state: &State, _mv: &Move) -> Option<&State> {
        None
    }
    fn insert_applied_state(&mut self, _state: &State, _mv: &Move, _result: State) {}
    fn get_terminal_value(&self, _state: &State) -> Option<&Option<f32>> {
        None
    }
    fn insert_terminal_value(&mut self, _state: &State, _value: Option<f32>) {}
}
trait MCTSGame: Sized {
    type State: Clone + PartialEq;
    type Move;
    type Player: MCTSPlayer;
    type Cache: GameCache<Self::State, Self::Move>;
    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a>;
    fn apply_move(
        state: &Self::State,
        mv: &Self::Move,
        game_cache: &mut Self::Cache,
    ) -> Self::State;
    fn evaluate(state: &Self::State, game_cache: &mut Self::Cache) -> Option<f32>;
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
    fn update_stats(&mut self, result: f32);
    fn calc_utc(&mut self, parent_visits: usize, base_c: f32, perspective_player: G::Player)
        -> f32;
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
trait UTCCache<G: MCTSGame, UP: UCTPolicy<G>> {
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
trait ExpansionPolicy<G: MCTSGame> {
    fn new(state: &G::State, game_cache: &mut G::Cache) -> Self;
    fn should_expand(&self, visits: usize, num_parent_children: usize) -> bool;
    fn pop_expandable_move(&mut self, visits: usize, num_parent_children: usize)
        -> Option<G::Move>;
}
trait HeuristicCache<State, Move> {
    fn new() -> Self;
    fn get_intermediate_score(&self, _state: &State) -> Option<f32> {
        None
    }
    fn insert_intermediate_score(&mut self, _state: &State, _value: f32) {}
    fn get_move_score(&self, _state: &State, _mv: &Move) -> Option<f32> {
        None
    }
    fn insert_move_score(&mut self, _state: &State, _mv: &Move, _value: f32) {}
}
trait Heuristic<G: MCTSGame> {
    type Cache: HeuristicCache<G::State, G::Move>;
    fn evaluate_state(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
    ) -> f32;
    fn evaluate_move(
        state: &G::State,
        mv: &G::Move,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
    ) -> f32;
}
trait SimulationPolicy<G: MCTSGame, H: Heuristic<G>> {
    fn should_cutoff(
        _state: &G::State,
        _depth: usize,
        _game_cache: &mut G::Cache,
        _heuristic_cache: &mut H::Cache,
    ) -> Option<f32> {
        None
    }
}
impl MCTSPlayer for TicTacToeStatus {
    fn next(&self) -> Self {
        match self {
            TicTacToeStatus::Me => TicTacToeStatus::Opp,
            TicTacToeStatus::Opp => TicTacToeStatus::Me,
            _ => panic!("Invalid player"),
        }
    }
}
use std::collections::HashSet;
#[repr(i8)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
enum TicTacToeStatus {
    #[default]
    Vacant = 0,
    Me = 1,
    Opp = -1,
    Tie = 20,
}
impl TicTacToeStatus {
    fn is_vacant(&self) -> bool {
        *self == Self::Vacant
    }
    fn is_not_vacant(&self) -> bool {
        *self != Self::Vacant
    }
    fn evaluate(&self) -> Option<f32> {
        match self {
            TicTacToeStatus::Me => Some(1.0),
            TicTacToeStatus::Opp => Some(0.0),
            TicTacToeStatus::Tie => Some(0.5),
            TicTacToeStatus::Vacant => None,
        }
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
struct TicTacToeGameData {
    map: MyMap3x3<TicTacToeStatus>,
}
impl TicTacToeGameData {
    const SCORE_LINES: [[CellIndex3x3; 3]; 8] = [
        [CellIndex3x3::TL, CellIndex3x3::MM, CellIndex3x3::BR],
        [CellIndex3x3::TR, CellIndex3x3::MM, CellIndex3x3::BL],
        [CellIndex3x3::ML, CellIndex3x3::MM, CellIndex3x3::MR],
        [CellIndex3x3::TM, CellIndex3x3::MM, CellIndex3x3::BM],
        [CellIndex3x3::TL, CellIndex3x3::TM, CellIndex3x3::TR],
        [CellIndex3x3::BL, CellIndex3x3::BM, CellIndex3x3::BR],
        [CellIndex3x3::TL, CellIndex3x3::ML, CellIndex3x3::BL],
        [CellIndex3x3::TR, CellIndex3x3::MR, CellIndex3x3::BR],
    ];
    fn new() -> Self {
        TicTacToeGameData {
            map: MyMap3x3::init(TicTacToeStatus::Vacant),
        }
    }
    fn get_status(&self) -> TicTacToeStatus {
        for score_line in Self::SCORE_LINES.iter() {
            match score_line
                .iter()
                .map(|cell| *self.map.get_cell(*cell) as i8)
                .sum()
            {
                3 => return TicTacToeStatus::Me,
                -3 => return TicTacToeStatus::Opp,
                _ => (),
            }
        }
        if self.map.iterate().all(|(_, v)| v.is_not_vacant()) {
            return TicTacToeStatus::Tie;
        }
        TicTacToeStatus::Vacant
    }
    fn get_status_increment(&self, cell: &CellIndex3x3) -> TicTacToeStatus {
        for score_line in Self::SCORE_LINES.iter() {
            if !score_line.contains(cell) {
                continue;
            }
            match score_line
                .iter()
                .map(|cell| *self.map.get_cell(*cell) as i8)
                .sum()
            {
                3 => return TicTacToeStatus::Me,
                -3 => return TicTacToeStatus::Opp,
                _ => (),
            }
        }
        if self.map.iterate().all(|(_, v)| v.is_not_vacant()) {
            return TicTacToeStatus::Tie;
        }
        TicTacToeStatus::Vacant
    }
    fn set_cell_value(&mut self, cell: CellIndex3x3, value: TicTacToeStatus) {
        self.map.set_cell(cell, value);
    }
    fn get_cell_value(&self, cell: CellIndex3x3) -> TicTacToeStatus {
        *self.map.get_cell(cell)
    }
    fn count_me_cells(&self) -> usize {
        self.map
            .iterate()
            .filter(|(_, v)| matches!(**v, TicTacToeStatus::Me))
            .count()
    }
    fn count_opp_cells(&self) -> usize {
        self.map
            .iterate()
            .filter(|(_, v)| matches!(**v, TicTacToeStatus::Opp))
            .count()
    }
    fn iter_map(&self) -> impl Iterator<Item = (CellIndex3x3, &TicTacToeStatus)> {
        self.map.iterate()
    }
    fn count_non_vacant_cells(&self) -> usize {
        self.map
            .iterate()
            .filter(|(_, v)| v.is_not_vacant())
            .count()
    }
    fn get_threats(&self) -> (u8, u8) {
        let mut me_threats: HashSet<CellIndex3x3> = HashSet::new();
        let mut opp_threats: HashSet<CellIndex3x3> = HashSet::new();
        for score_line in Self::SCORE_LINES.iter() {
            let (threat, vacant) = score_line.iter().fold(
                (0, CellIndex3x3::default()),
                |(mut threat, mut vacant), element| {
                    let cell_value = self.map.get_cell(*element);
                    if cell_value.is_vacant() {
                        vacant = *element;
                    }
                    threat += *cell_value as i8;
                    (threat, vacant)
                },
            );
            match threat {
                2 => {
                    me_threats.insert(vacant);
                }
                -2 => {
                    opp_threats.insert(vacant);
                }
                _ => (),
            }
        }
        (me_threats.len() as u8, opp_threats.len() as u8)
    }
}
