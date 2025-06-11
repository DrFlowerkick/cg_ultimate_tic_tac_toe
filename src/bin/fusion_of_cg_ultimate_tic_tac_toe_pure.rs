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
    let time_out_first_turn = Duration::from_millis(990);
    let time_out_successive_turns = Duration::from_millis(90);
    let time_out_codingame_input = Duration::from_millis(2000);
    let mut game_data = UltTTT::new();
    let mut mcts_ult_ttt: PlainMCTS<
        UltTTTMCTSGame,
        NoHeuristic,
        UltTTTMCTSConfig,
        CachedUTC,
        NoTranspositionTable,
        StaticC,
        ExpandAll,
        DefaultSimulationPolicy,
    > = PlainMCTS::new(UltTTTMCTSConfig::default(), NoHeuristic {});
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
    let mut turn_counter = 0;
    let (opponent_row, opponent_col) = rx.recv().expect("Failed to receive initial input");
    if opponent_row >= 0 {
        turn_counter += 1;
        game_data.set_current_player(TicTacToeStatus::Opp);
        let opp_action = (opponent_col as u8, opponent_row as u8);
        game_data = UltTTTMCTSGame::apply_move(
            &game_data,
            &UltTTTMove::try_from(opp_action).unwrap(),
            &mut mcts_ult_ttt.game_cache,
        );
    }
    mcts_ult_ttt.set_root(&game_data);
    let start = Instant::now();
    let mut number_of_iterations = 0;
    while start.elapsed() < time_out_first_turn {
        mcts_ult_ttt.iterate();
        number_of_iterations += 1;
    }
    eprintln!("Iterations of first turn: {}", number_of_iterations);
    loop {
        turn_counter += 1;
        let selected_move = *mcts_ult_ttt.select_move();
        game_data =
            UltTTTMCTSGame::apply_move(&game_data, &selected_move, &mut mcts_ult_ttt.game_cache);
        selected_move.execute_action();
        assert!(mcts_ult_ttt.set_root(&game_data));
        let start = Instant::now();
        let mut instant_input_received = Instant::now();
        let mut input_received = false;
        number_of_iterations = 0;
        loop {
            match rx.try_recv() {
                Ok((opponent_row, opponent_col)) => {
                    eprintln!("time from opp perspective: {:?}", start.elapsed());
                    turn_counter += 1;
                    let opp_action = (opponent_col as u8, opponent_row as u8);
                    game_data = UltTTTMCTSGame::apply_move(
                        &game_data,
                        &UltTTTMove::try_from(opp_action).unwrap(),
                        &mut mcts_ult_ttt.game_cache,
                    );
                    if !mcts_ult_ttt.set_root(&game_data) {
                        eprintln!("Reset root after opponent move in turn {}.", turn_counter);
                    }
                    instant_input_received = Instant::now();
                    input_received = true;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    mcts_ult_ttt.iterate();
                    number_of_iterations += 1;
                    if start.elapsed() > time_out_codingame_input {
                        panic!("Timeout while waiting for codingame input");
                    }
                    if input_received
                        && instant_input_received.elapsed() > time_out_successive_turns
                    {
                        eprintln!(
                            "time from my perspective: {:?}",
                            instant_input_received.elapsed()
                        );
                        eprintln!("total time of iterations: {:?}", start.elapsed());
                        break;
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    panic!("Codingame input thread disconnected");
                }
            }
        }
        eprintln!("Iterations of successive turns: {}", number_of_iterations);
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
struct UltTTTMCTSConfig {
    base_config: BaseConfig,
}
impl Default for UltTTTMCTSConfig {
    fn default() -> Self {
        UltTTTMCTSConfig {
            base_config: BaseConfig {
                exploration_constant: 1.4,
                progressive_widening_constant: 2.0,
                progressive_widening_exponent: 0.5,
                early_cut_off_depth: 30,
            },
        }
    }
}
impl MCTSConfig for UltTTTMCTSConfig {
    fn exploration_constant(&self) -> f32 {
        self.base_config.exploration_constant
    }
    fn progressive_widening_constant(&self) -> f32 {
        self.base_config.progressive_widening_constant
    }
    fn progressive_widening_exponent(&self) -> f32 {
        self.base_config.progressive_widening_exponent
    }
    fn early_cut_off_depth(&self) -> usize {
        self.base_config.early_cut_off_depth
    }
}
use std::cmp::Ordering;
use std::fmt::Write;
#[derive(Copy, Clone, PartialEq, Default)]
struct UltTTTMove {
    status_index: CellIndex3x3,
    mini_board_index: CellIndex3x3,
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
    last_player: TicTacToeStatus,
}
impl UltTTT {
    fn new() -> Self {
        UltTTT {
            map: MyMap3x3::new(),
            status_map: TicTacToeGameData::new(),
            next_action_constraint: NextActionConstraint::Init,
            current_player: TicTacToeStatus::Me,
            last_player: TicTacToeStatus::Me,
        }
    }
    fn set_current_player(&mut self, player: TicTacToeStatus) {
        self.current_player = player;
    }
    fn next_player(&mut self) {
        self.last_player = self.current_player;
        self.current_player = self.current_player.next();
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
        _game_cache: &mut Self::Cache,
    ) -> Self::State {
        let mut new_state = *state;
        new_state
            .map
            .get_cell_mut(mv.status_index)
            .set_cell_value(mv.mini_board_index, state.current_player);
        let status = new_state.map.get_cell(mv.status_index).get_status();
        if !status.is_vacant() {
            new_state.status_map.set_cell_value(mv.status_index, status);
        }
        new_state.next_action_constraint = if new_state
            .status_map
            .get_cell_value(mv.mini_board_index)
            .is_vacant()
        {
            NextActionConstraint::MiniBoard(mv.mini_board_index)
        } else {
            NextActionConstraint::None
        };
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
    fn last_player(state: &Self::State) -> Self::Player {
        state.last_player
    }
    fn perspective_player() -> Self::Player {
        TicTacToeStatus::Me
    }
}
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
struct NoTranspositionTable {}
impl<State, ID> TranspositionTable<State, ID> for NoTranspositionTable {
    fn new() -> Self {
        NoTranspositionTable {}
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
struct BaseConfig {
    exploration_constant: f32,
    progressive_widening_constant: f32,
    progressive_widening_exponent: f32,
    early_cut_off_depth: usize,
}
use rand::prelude::SliceRandom;
struct ExpandAll {}
impl<G, H, Config> ExpansionPolicy<G, H, Config> for ExpandAll
where
    G: MCTSGame,
    H: Heuristic<G>,
    Config: MCTSConfig,
{
    fn new(
        _state: &G::State,
        _game_cache: &mut G::Cache,
        _heuristic_cache: &mut H::Cache,
        _heuristic_config: &H::Config,
    ) -> Self {
        ExpandAll {}
    }
    fn expandable_moves(
        &mut self,
        _visits: usize,
        _num_parent_children: usize,
        state: &G::State,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> Vec<G::Move> {
        let mut moves: Vec<_> = G::available_moves(state).collect();
        moves.shuffle(&mut rand::thread_rng());
        moves
    }
}
struct DefaultSimulationPolicy {}
impl<G: MCTSGame, H: Heuristic<G>, Config: MCTSConfig> SimulationPolicy<G, H, Config>
    for DefaultSimulationPolicy
{
}
struct StaticC {}
impl<G: MCTSGame, Config: MCTSConfig> UCTPolicy<G, Config> for StaticC {}
struct CachedUTC {
    exploitation: f32,
    exploration: f32,
    last_parent_visits: usize,
}
impl<G, UTC, Config> UTCCache<G, UTC, Config> for CachedUTC
where
    G: MCTSGame,
    UTC: UCTPolicy<G, Config>,
    Config: MCTSConfig,
{
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
        last_player: G::Player,
        perspective_player: G::Player,
    ) {
        self.exploitation =
            UTC::exploitation_score(acc_value, visits, last_player, perspective_player);
    }
    fn get_exploitation(&self, _v: usize, _a: f32, _c: G::Player, _p: G::Player) -> f32 {
        self.exploitation
    }
    fn update_exploration(&mut self, visits: usize, parent_visits: usize, mcts_config: &Config) {
        if self.last_parent_visits != parent_visits {
            self.exploration = UTC::exploration_score(visits, parent_visits, mcts_config);
            self.last_parent_visits = parent_visits;
        }
    }
    fn get_exploration(&self, _v: usize, _p: usize, _mc: &Config) -> f32 {
        self.exploration
    }
}
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
impl HeuristicConfig for NoHeuristic {}
impl<G: MCTSGame> Heuristic<G> for NoHeuristic {
    type Cache = NoHeuristicCache<G::State, G::Move>;
    type Config = Self;
    fn evaluate_state(
        state: &G::State,
        game_cache: &mut G::Cache,
        _heuristic_cache: &mut Self::Cache,
        _perspective_player: Option<G::Player>,
        _heuristic_config: &Self::Config,
    ) -> f32 {
        G::evaluate(state, game_cache).unwrap_or(0.5)
    }
    fn evaluate_move(
        _state: &G::State,
        _mv: &G::Move,
        _game_cache: &mut G::Cache,
        _heuristic_cache: &mut Self::Cache,
        _heuristic_config: &Self::Config,
    ) -> f32 {
        0.0
    }
}
use rand::prelude::IteratorRandom;
struct PlainMCTS<G, H, MC, UC, TT, UP, EP, SP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig,
    UC: UTCCache<G, UP, MC>,
    TT: TranspositionTable<G::State, usize>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
    SP: SimulationPolicy<G, H, MC>,
{
    tree: PlainTree<G, H, Self, UC>,
    mcts_config: MC,
    heuristic_config: H::Config,
    game_cache: G::Cache,
    heuristic_cache: H::Cache,
    transposition_table: TT,
    phantom: std::marker::PhantomData<()>,
}
impl<G, H, MC, UC, TT, UP, EP, SP> PlainMCTS<G, H, MC, UC, TT, UP, EP, SP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig,
    UC: UTCCache<G, UP, MC>,
    TT: TranspositionTable<G::State, usize>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
    SP: SimulationPolicy<G, H, MC>,
{
    fn new(mcts_config: MC, heuristic_config: H::Config) -> Self {
        Self {
            tree: PlainTree::new(),
            mcts_config,
            heuristic_config,
            game_cache: G::Cache::new(),
            heuristic_cache: H::Cache::new(),
            transposition_table: TT::new(),
            phantom: std::marker::PhantomData,
        }
    }
}
impl<G, H, MC, UC, TT, UP, EP, SP> MCTSAlgo<G, H> for PlainMCTS<G, H, MC, UC, TT, UP, EP, SP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig,
    UC: UTCCache<G, UP, MC>,
    TT: TranspositionTable<G::State, usize>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
    SP: SimulationPolicy<G, H, MC>,
{
    type Tree = PlainTree<G, H, Self, UC>;
    type NodeID = usize;
    type Config = MC;
    type TranspositionTable = TT;
    type UTC = UP;
    type Expansion = EP;
    type Simulation = SP;
    fn set_root(&mut self, state: &G::State) -> bool {
        if let Some(root_id) = self.tree.root_id() {
            if let Some(node_of_state_id) = self.transposition_table.get(state) {
                self.tree.set_root(*node_of_state_id);
                return true;
            }
            if let Some((new_root_id, _)) =
                self.tree
                    .get_children(root_id)
                    .iter()
                    .map(|&(my_move_node_id, _)| {
                        (
                            my_move_node_id,
                            self.tree.get_node(my_move_node_id).get_state(),
                        )
                    })
                    .chain(self.tree.get_children(root_id).iter().flat_map(
                        |&(my_move_node_id, _)| {
                            self.tree.get_children(my_move_node_id).iter().map(
                                |&(opponent_move_node_id, _)| {
                                    (
                                        opponent_move_node_id,
                                        self.tree.get_node(opponent_move_node_id).get_state(),
                                    )
                                },
                            )
                        },
                    ))
                    .find(|(_, move_node_state)| *move_node_state == state)
            {
                self.tree.set_root(new_root_id);
                return true;
            }
        }
        self.reset_root(state);
        false
    }
    fn reset_root(&mut self, state: &<G as MCTSGame>::State) {
        let expansion_policy = EP::new(
            state,
            &mut self.game_cache,
            &mut self.heuristic_cache,
            &self.heuristic_config,
        );
        let new_root = PlainNode::new(state.clone(), expansion_policy);
        let root_id = self.tree.init_root(new_root);
        self.transposition_table = TT::new();
        self.transposition_table.insert(state.clone(), root_id);
    }
    fn iterate(&mut self) {
        let (tree, mcts_config, heuristic_config, game_cache, heuristic_cache, transposition_table) = (
            &mut self.tree,
            &self.mcts_config,
            &self.heuristic_config,
            &mut self.game_cache,
            &mut self.heuristic_cache,
            &mut self.transposition_table,
        );
        let root_id = tree
            .root_id()
            .expect("Tree root must be initialized before iterate.");
        let mut path = vec![root_id];
        let mut current_id = root_id;
        let mut new_children: Vec<Self::NodeID> = Vec::new();
        loop {
            while !tree.get_children(current_id).is_empty() {
                let parent_visits = tree.get_node(current_id).get_visits();
                let num_parent_children = tree.get_children(current_id).len();
                if tree.get_node(current_id).should_expand(
                    parent_visits,
                    num_parent_children,
                    mcts_config,
                    heuristic_config,
                ) {
                    break;
                }
                let mut best_child_index: Option<_> = None;
                let mut best_utc = f32::NEG_INFINITY;
                for vec_index in 0..num_parent_children {
                    let (child_index, _) = tree.get_children(current_id)[vec_index];
                    let utc = tree.get_node_mut(child_index).calc_utc(
                        parent_visits,
                        G::perspective_player(),
                        mcts_config,
                    );
                    if utc > best_utc {
                        best_utc = utc;
                        best_child_index = Some(child_index);
                    }
                }
                let best_child_index = best_child_index.expect("Could not find best child index.");
                path.push(best_child_index);
                current_id = best_child_index;
            }
            if (tree.get_node(current_id).get_visits() == 0 && current_id != root_id)
                || G::evaluate(tree.get_node(current_id).get_state(), game_cache).is_some()
            {
                break;
            } else {
                let num_parent_children = tree.get_children(current_id).len();
                let expandable_moves = tree.get_node_mut(current_id).expandable_moves(
                    num_parent_children,
                    mcts_config,
                    heuristic_config,
                );
                for mv in expandable_moves {
                    let new_state =
                        G::apply_move(tree.get_node(current_id).get_state(), &mv, game_cache);
                    if let Some(&cached_node_id) = transposition_table.get(&new_state) {
                        tree.link_child(current_id, mv, cached_node_id);
                        let visits = tree.get_node(cached_node_id).get_visits();
                        if visits == 0 {
                            new_children.push(cached_node_id);
                        } else {
                            let get_accumulated_value =
                                tree.get_node(cached_node_id).get_accumulated_value();
                            back_propagation(tree, &path, get_accumulated_value / visits as f32);
                        }
                        continue;
                    }
                    let expansion_policy =
                        EP::new(&new_state, game_cache, heuristic_cache, heuristic_config);
                    let new_node = PlainNode::new(new_state.clone(), expansion_policy);
                    let new_child_id = tree.add_child(current_id, mv, new_node);
                    transposition_table.insert(new_state, new_child_id);
                    new_children.push(new_child_id);
                }
                let Some (child_index) = new_children . first () else { continue ; } ;
                path.push(*child_index);
                current_id = *child_index;
                break;
            };
        }
        let mut current_state = tree.get_node(current_id).get_state().clone();
        let mut depth = 0;
        let simulation_result = loop {
            if let Some(final_score) = G::evaluate(&current_state, game_cache) {
                break final_score;
            }
            if let Some(heuristic) = SP::should_cutoff(
                &current_state,
                depth,
                game_cache,
                heuristic_cache,
                Some(G::perspective_player()),
                mcts_config,
                heuristic_config,
            ) {
                break heuristic;
            }
            current_state = G::apply_move(
                &current_state,
                &G::available_moves(&current_state)
                    .choose(&mut rand::thread_rng())
                    .expect("No available moves"),
                game_cache,
            );
            depth += 1;
        };
        back_propagation(tree, &path, simulation_result);
    }
    fn select_move(&self) -> &G::Move {
        let root_id = self
            .tree
            .root_id()
            .expect("Root node must be initialized before selecting a move");
        let (_, mv) = self
            .tree
            .get_children(root_id)
            .iter()
            .max_by_key(|&&(child_id, _)| self.tree.get_node(child_id).get_visits())
            .expect("could not find move id");
        mv
    }
}
fn back_propagation<G, H, A, T>(tree: &mut T, path: &[A::NodeID], result: f32)
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    T: MCTSTree<G, H, A>,
{
    for &node_id in path.iter().rev() {
        tree.get_node_mut(node_id).update_stats(result);
    }
}
struct PlainNode<G, H, MC, UC, UP, EP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig,
    UC: UTCCache<G, UP, MC>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
{
    state: G::State,
    visits: usize,
    accumulated_value: f32,
    utc_cache: UC,
    expansion_policy: EP,
    phantom: std::marker::PhantomData<(H, MC, UP)>,
}
impl<G, H, MC, UC, UP, EP> MCTSNode<G, H, MC, UP, EP> for PlainNode<G, H, MC, UC, UP, EP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig,
    UC: UTCCache<G, UP, MC>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
{
    type Cache = UC;
    fn new(state: G::State, expansion_policy: EP) -> Self {
        PlainNode {
            state,
            visits: 0,
            accumulated_value: 0.0,
            utc_cache: UC::new(),
            expansion_policy,
            phantom: std::marker::PhantomData,
        }
    }
    fn get_state(&self) -> &G::State {
        &self.state
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
            G::last_player(&self.state),
            G::perspective_player(),
        );
    }
    fn calc_utc(
        &mut self,
        parent_visits: usize,
        perspective_player: G::Player,
        mcts_config: &MC,
    ) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let exploitation = self.utc_cache.get_exploitation(
            self.visits,
            self.accumulated_value,
            G::last_player(&self.state),
            perspective_player,
        );
        self.utc_cache
            .update_exploration(self.visits, parent_visits, mcts_config);
        let exploration = self
            .utc_cache
            .get_exploration(self.visits, parent_visits, mcts_config);
        exploitation + exploration
    }
    fn should_expand(
        &self,
        visits: usize,
        num_parent_children: usize,
        mcts_config: &MC,
        heuristic_config: &H::Config,
    ) -> bool {
        self.expansion_policy.should_expand(
            visits,
            num_parent_children,
            mcts_config,
            heuristic_config,
        )
    }
    fn expandable_moves(
        &mut self,
        num_parent_children: usize,
        mcts_config: &MC,
        heuristic_config: &H::Config,
    ) -> Vec<G::Move> {
        self.expansion_policy.expandable_moves(
            self.visits,
            num_parent_children,
            &self.state,
            mcts_config,
            heuristic_config,
        )
    }
}
type Node<G, H, A, UC> = PlainNode<
    G,
    H,
    <A as MCTSAlgo<G, H>>::Config,
    UC,
    <A as MCTSAlgo<G, H>>::UTC,
    <A as MCTSAlgo<G, H>>::Expansion,
>;
struct PlainTree<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UTCCache<G, A::UTC, A::Config>,
{
    nodes: Vec<Node<G, H, A, UC>>,
    edges: Vec<Vec<(usize, G::Move)>>,
    root_id: usize,
    phantom: std::marker::PhantomData<(G, H, UC)>,
}
impl<G, H, A, UC> MCTSTree<G, H, A> for PlainTree<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H, NodeID = usize>,
    UC: UTCCache<G, A::UTC, A::Config>,
{
    type Node = Node<G, H, A, UC>;
    fn new() -> Self {
        PlainTree {
            nodes: vec![],
            edges: vec![],
            root_id: 0,
            phantom: std::marker::PhantomData,
        }
    }
    fn init_root(&mut self, root_value: Self::Node) -> A::NodeID {
        self.nodes.clear();
        self.edges.clear();
        self.nodes.push(root_value);
        self.edges.push(vec![]);
        self.root_id = 0;
        self.root_id
    }
    fn set_root(&mut self, new_root_id: A::NodeID) {
        self.root_id = new_root_id;
    }
    fn root_id(&self) -> Option<A::NodeID> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(self.root_id)
        }
    }
    fn get_node(&self, id: A::NodeID) -> &Self::Node {
        &self.nodes[id]
    }
    fn get_node_mut(&mut self, id: A::NodeID) -> &mut Self::Node {
        &mut self.nodes[id]
    }
    fn add_child(&mut self, parent_id: A::NodeID, mv: G::Move, child_value: Self::Node) -> usize {
        let child_id = self.nodes.len();
        self.nodes.push(child_value);
        self.edges.push(vec![]);
        self.link_child(parent_id, mv, child_id);
        child_id
    }
    fn link_child(&mut self, parent_id: A::NodeID, mv: G::Move, child_id: A::NodeID) {
        self.edges
            .get_mut(parent_id)
            .expect("Expected edges of parent.")
            .push((child_id, mv));
    }
    fn get_children(&self, id: A::NodeID) -> &[(A::NodeID, G::Move)] {
        &self.edges[id][..]
    }
}
trait TranspositionTable<State, ID> {
    fn new() -> Self;
    fn get(&self, _state: &State) -> Option<&ID> {
        None
    }
    fn insert(&mut self, _state: State, _value: ID) {}
    fn clear(&mut self) {}
}
trait MCTSConfig {
    fn exploration_constant(&self) -> f32 {
        1.4
    }
    fn progressive_widening_constant(&self) -> f32 {
        2.0
    }
    fn progressive_widening_exponent(&self) -> f32 {
        0.5
    }
    fn early_cut_off_depth(&self) -> usize {
        20
    }
}
trait MCTSAlgo<G: MCTSGame, H: Heuristic<G>>: Sized {
    type Tree: MCTSTree<G, H, Self>;
    type NodeID: Copy + Eq + std::fmt::Debug;
    type Config: MCTSConfig;
    type TranspositionTable: TranspositionTable<G::State, Self::NodeID>;
    type UTC: UCTPolicy<G, Self::Config>;
    type Expansion: ExpansionPolicy<G, H, Self::Config>;
    type Simulation: SimulationPolicy<G, H, Self::Config>;
    fn set_root(&mut self, state: &G::State) -> bool;
    fn reset_root(&mut self, state: &G::State);
    fn iterate(&mut self);
    fn select_move(&self) -> &G::Move;
}
trait UCTPolicy<G: MCTSGame, Config: MCTSConfig> {
    fn exploitation_score(
        accumulated_value: f32,
        visits: usize,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32 {
        let raw = accumulated_value / visits as f32;
        if last_player == perspective_player {
            raw
        } else {
            1.0 - raw
        }
    }
    fn exploration_score(visits: usize, parent_visits: usize, mcts_config: &Config) -> f32 {
        mcts_config.exploration_constant() * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}
trait ExpansionPolicy<G: MCTSGame, H: Heuristic<G>, Config: MCTSConfig> {
    fn new(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut H::Cache,
        heuristic_config: &H::Config,
    ) -> Self;
    fn should_expand(
        &self,
        _visits: usize,
        _num_parent_children: usize,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> bool {
        false
    }
    fn expandable_moves(
        &mut self,
        _visits: usize,
        _num_parent_children: usize,
        state: &G::State,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> Vec<G::Move>;
}
trait SimulationPolicy<G: MCTSGame, H: Heuristic<G>, Config: MCTSConfig> {
    fn should_cutoff(
        _state: &G::State,
        _depth: usize,
        _game_cache: &mut G::Cache,
        _heuristic_cache: &mut H::Cache,
        _perspective_player: Option<G::Player>,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> Option<f32> {
        None
    }
}
trait MCTSTree<G, H, A>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Node: MCTSNode<G, H, A::Config, A::UTC, A::Expansion>;
    fn new() -> Self;
    fn init_root(&mut self, root_value: Self::Node) -> A::NodeID;
    fn set_root(&mut self, new_root_id: A::NodeID);
    fn root_id(&self) -> Option<A::NodeID>;
    fn get_node(&self, id: A::NodeID) -> &Self::Node;
    fn get_node_mut(&mut self, id: A::NodeID) -> &mut Self::Node;
    fn add_child(
        &mut self,
        parent_id: A::NodeID,
        mv: G::Move,
        child_value: Self::Node,
    ) -> A::NodeID;
    fn link_child(&mut self, parent_id: A::NodeID, mv: G::Move, child_id: A::NodeID);
    fn get_children(&self, id: A::NodeID) -> &[(A::NodeID, G::Move)];
}
trait UTCCache<G, UTC, Config>
where
    G: MCTSGame,
    UTC: UCTPolicy<G, Config>,
    Config: MCTSConfig,
{
    fn new() -> Self;
    fn update_exploitation(
        &mut self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    );
    fn get_exploitation(
        &self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32;
    fn update_exploration(&mut self, visits: usize, parent_visits: usize, mcts_config: &Config);
    fn get_exploration(&self, visits: usize, parent_visits: usize, mcts_config: &Config) -> f32;
}
trait MCTSNode<G, H, MC, UP, EP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
{
    type Cache: UTCCache<G, UP, MC>;
    fn new(state: G::State, expansion_policy: EP) -> Self;
    fn get_state(&self) -> &G::State;
    fn get_visits(&self) -> usize;
    fn get_accumulated_value(&self) -> f32;
    fn update_stats(&mut self, result: f32);
    fn calc_utc(
        &mut self,
        parent_visits: usize,
        perspective_player: G::Player,
        mcts_config: &MC,
    ) -> f32;
    fn should_expand(
        &self,
        visits: usize,
        num_parent_children: usize,
        mcts_config: &MC,
        heuristic_config: &H::Config,
    ) -> bool;
    fn expandable_moves(
        &mut self,
        num_parent_children: usize,
        mcts_config: &MC,
        heuristic_config: &H::Config,
    ) -> Vec<G::Move>;
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
    type Player: GamePlayer;
    type Cache: GameCache<Self::State, Self::Move>;
    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a>;
    fn apply_move(
        state: &Self::State,
        mv: &Self::Move,
        game_cache: &mut Self::Cache,
    ) -> Self::State;
    fn evaluate(state: &Self::State, game_cache: &mut Self::Cache) -> Option<f32>;
    fn current_player(state: &Self::State) -> Self::Player;
    fn last_player(state: &Self::State) -> Self::Player;
    fn perspective_player() -> Self::Player;
}
trait GamePlayer: PartialEq {
    fn next(&self) -> Self;
}
trait HeuristicCache<State, Move> {
    fn new() -> Self;
    fn get_intermediate_score(&self, _state: &State) -> Option<f32> {
        None
    }
    fn insert_intermediate_score(&mut self, _state: &State, _score: f32) {}
    fn get_move_score(&self, _state: &State, _mv: &Move) -> Option<f32> {
        None
    }
    fn insert_move_score(&mut self, _state: &State, _mv: &Move, _score: f32) {}
}
trait HeuristicConfig {
    fn progressive_widening_initial_threshold(&self) -> f32 {
        0.8
    }
    fn progressive_widening_decay_rate(&self) -> f32 {
        0.95
    }
    fn early_cut_off_upper_bound(&self) -> f32 {
        0.05
    }
    fn early_cut_off_lower_bound(&self) -> f32 {
        0.95
    }
}
trait Heuristic<G: MCTSGame> {
    type Cache: HeuristicCache<G::State, G::Move>;
    type Config: HeuristicConfig;
    fn evaluate_state(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        perspective_player: Option<G::Player>,
        heuristic_config: &Self::Config,
    ) -> f32;
    fn evaluate_move(
        state: &G::State,
        mv: &G::Move,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> f32;
    fn sort_moves(
        state: &G::State,
        moves: Vec<G::Move>,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> Vec<(f32, G::Move)> {
        let mut heuristic_moves = moves
            .into_iter()
            .map(|mv| {
                (
                    Self::evaluate_move(state, &mv, game_cache, heuristic_cache, heuristic_config),
                    mv,
                )
            })
            .collect::<Vec<_>>();
        heuristic_moves.shuffle(&mut rand::thread_rng());
        heuristic_moves
            .sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        heuristic_moves
    }
}
impl GamePlayer for TicTacToeStatus {
    fn next(&self) -> Self {
        match self {
            TicTacToeStatus::Me => TicTacToeStatus::Opp,
            TicTacToeStatus::Opp => TicTacToeStatus::Me,
            _ => panic!("Invalid player"),
        }
    }
}
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
}
