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
        UltTTTMCTSGameNoGameCache,
        DynamicC,
        CachedUTC,
        HPWDefaultTTTNoGameCache,
        UltTTTHeuristic,
        HeuristicCutoff,
    > = PlainMCTS::new(
        UltTTTMCTSConfig::optimized(),
        UltTTTHeuristicConfig::optimized(),
    );
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
            &UltTTTMove::try_from(opp_action).unwrap(),
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
                        &UltTTTMove::try_from(opp_action).unwrap(),
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
#[derive(Debug, Clone, Copy)]
struct UltTTTMCTSConfig {
    base_config: BaseConfig,
}
impl UltTTTMCTSConfig {
    fn optimized() -> Self {
        UltTTTMCTSConfig {
            base_config: BaseConfig {
                exploration_constant: 1.134,
                progressive_widening_constant: 1.933,
                progressive_widening_exponent: 0.455,
                early_cut_off_depth: 18,
            },
        }
    }
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
#[derive(Debug, Clone, Copy)]
struct UltTTTHeuristicConfig {
    base_config: BaseHeuristicConfig,
    meta_weight_base: f32,
    meta_weight_progress_offset: f32,
    meta_cell_big_threat: f32,
    meta_cell_small_threat: f32,
    constraint_factor: f32,
    free_choice_constraint_factor: f32,
    direct_loss_value: f32,
}
impl UltTTTHeuristicConfig {
    fn optimized() -> Self {
        UltTTTHeuristicConfig {
            base_config: BaseHeuristicConfig {
                progressive_widening_initial_threshold: 0.695,
                progressive_widening_decay_rate: 0.928,
                early_cut_off_lower_bound: 0.034,
                early_cut_off_upper_bound: 0.914,
            },
            meta_weight_base: 0.341,
            meta_weight_progress_offset: 0.175,
            meta_cell_big_threat: 3.946,
            meta_cell_small_threat: 1.078,
            constraint_factor: 1.085,
            free_choice_constraint_factor: 1.704,
            direct_loss_value: 0.023,
        }
    }
}
impl HeuristicConfig for UltTTTHeuristicConfig {
    fn progressive_widening_initial_threshold(&self) -> f32 {
        self.base_config.progressive_widening_initial_threshold
    }
    fn progressive_widening_decay_rate(&self) -> f32 {
        self.base_config.progressive_widening_decay_rate
    }
    fn early_cut_off_lower_bound(&self) -> f32 {
        self.base_config.early_cut_off_lower_bound
    }
    fn early_cut_off_upper_bound(&self) -> f32 {
        self.base_config.early_cut_off_upper_bound
    }
}
impl Default for UltTTTHeuristicConfig {
    fn default() -> Self {
        UltTTTHeuristicConfig {
            base_config: BaseHeuristicConfig {
                progressive_widening_initial_threshold: 0.8,
                progressive_widening_decay_rate: 0.95,
                early_cut_off_lower_bound: 0.05,
                early_cut_off_upper_bound: 0.95,
            },
            meta_weight_base: 0.3,
            meta_weight_progress_offset: 0.4,
            meta_cell_big_threat: 3.0,
            meta_cell_small_threat: 1.5,
            constraint_factor: 1.5,
            free_choice_constraint_factor: 1.5,
            direct_loss_value: 0.01,
        }
    }
}
trait UltTTTGameCacheTrait {
    fn get_status(&mut self, board: &TicTacToeGameData) -> TicTacToeStatus;
    fn get_board_progress(&mut self, board: &TicTacToeGameData) -> (usize, usize, usize);
    fn get_board_threats(&mut self, board: &TicTacToeGameData) -> (usize, usize);
    fn get_meta_cell_threats(
        &mut self,
        board: &TicTacToeGameData,
        index: CellIndex3x3,
    ) -> (u8, u8, u8, u8);
}
impl UltTTTGameCacheTrait for NoGameCache<UltTTT, UltTTTMove> {
    fn get_status(&mut self, board: &TicTacToeGameData) -> TicTacToeStatus {
        board.get_status()
    }
    fn get_board_progress(&mut self, board: &TicTacToeGameData) -> (usize, usize, usize) {
        let my_wins = board.count_me_cells();
        let opp_wins = board.count_opp_cells();
        let played_cells = board.count_non_vacant_cells();
        (my_wins, opp_wins, played_cells)
    }
    fn get_board_threats(&mut self, board: &TicTacToeGameData) -> (usize, usize) {
        board.get_threats()
    }
    fn get_meta_cell_threats(
        &mut self,
        board: &TicTacToeGameData,
        index: CellIndex3x3,
    ) -> (u8, u8, u8, u8) {
        board.get_meta_cell_threats(index)
    }
}
struct UltTTTHeuristic {}
impl UltTTTHeuristic {
    fn is_direct_loss(
        player: TicTacToeStatus,
        my_threats: usize,
        my_meta_threats: u8,
        opp_threats: usize,
        opp_meta_threats: u8,
    ) -> bool {
        match player {
            TicTacToeStatus::Me => opp_threats > 0 && opp_meta_threats > 0,
            TicTacToeStatus::Opp => my_threats > 0 && my_meta_threats > 0,
            _ => unreachable!("Player is alway Me or Opp"),
        }
    }
}
impl<GC: UltTTTGameCacheTrait + GameCache<UltTTT, UltTTTMove>> Heuristic<UltTTTMCTSGame<GC>>
    for UltTTTHeuristic
{
    type Cache = NoHeuristicCache<UltTTT, UltTTTMove>;
    type Config = UltTTTHeuristicConfig;
    fn evaluate_state(
        state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
        game_cache: &mut <UltTTTMCTSGame<GC> as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
        perspective_player: Option<<UltTTTMCTSGame<GC> as MCTSGame>::Player>,
        heuristic_config: &Self::Config,
    ) -> f32 {
        let perspective_is_last_player = match perspective_player {
            Some(player) => player == state.last_player,
            None => true,
        };
        if let Some(score) = heuristic_cache.get_intermediate_score(state) {
            return if perspective_is_last_player {
                score
            } else {
                1.0 - score
            };
        }
        let score = match UltTTTMCTSGame::evaluate(state, game_cache) {
            Some(value) => value,
            None => {
                let mut my_threat_sum = 0.0;
                let mut opp_threat_sum = 0.0;
                for (status_index, _) in state.status_map.iter_map().filter(|(_, c)| c.is_vacant())
                {
                    let (my_threats, opp_threats) =
                        game_cache.get_board_threats(state.map.get_cell(status_index));
                    let cell_weight = status_index.cell_weight();
                    let (
                        my_meta_threats,
                        my_meta_small_threats,
                        opp_meta_threats,
                        opp_meta_small_threats,
                    ) = game_cache.get_meta_cell_threats(&state.status_map, status_index);
                    let my_meta_factor = 1.0
                        + heuristic_config.meta_cell_big_threat * my_meta_threats as f32
                        + heuristic_config.meta_cell_small_threat * my_meta_small_threats as f32;
                    let opp_meta_factor = 1.0
                        + heuristic_config.meta_cell_big_threat * opp_meta_threats as f32
                        + heuristic_config.meta_cell_small_threat * opp_meta_small_threats as f32;
                    let constraint_factor = match state.next_action_constraint {
                        NextActionConstraint::MiniBoard(next_board) => {
                            if status_index == next_board {
                                if UltTTTHeuristic::is_direct_loss(
                                    state.last_player,
                                    my_threats,
                                    my_meta_threats,
                                    opp_threats,
                                    opp_meta_threats,
                                ) {
                                    heuristic_cache.insert_intermediate_score(
                                        state,
                                        heuristic_config.direct_loss_value,
                                    );
                                    return if perspective_is_last_player {
                                        heuristic_config.direct_loss_value
                                    } else {
                                        1.0 - heuristic_config.direct_loss_value
                                    };
                                }
                                heuristic_config.constraint_factor
                            } else {
                                1.0
                            }
                        }
                        NextActionConstraint::None => {
                            if UltTTTHeuristic::is_direct_loss(
                                state.last_player,
                                my_threats,
                                my_meta_threats,
                                opp_threats,
                                opp_meta_threats,
                            ) {
                                heuristic_cache.insert_intermediate_score(
                                    state,
                                    heuristic_config.direct_loss_value,
                                );
                                return if perspective_is_last_player {
                                    heuristic_config.direct_loss_value
                                } else {
                                    1.0 - heuristic_config.direct_loss_value
                                };
                            }
                            heuristic_config.free_choice_constraint_factor
                        }
                        NextActionConstraint::Init => {
                            unreachable!("Init is reserved for initial tree root node.")
                        }
                    };
                    let (my_constraint_factor, opp_constraint_factor) = match state.current_player {
                        TicTacToeStatus::Me => (constraint_factor, 1.0),
                        TicTacToeStatus::Opp => (1.0, constraint_factor),
                        _ => unreachable!("Only Me and Opp are allowed for player."),
                    };
                    my_threat_sum +=
                        my_constraint_factor * my_meta_factor * cell_weight * my_threats as f32;
                    opp_threat_sum +=
                        opp_constraint_factor * opp_meta_factor * cell_weight * opp_threats as f32;
                }
                let (my_wins, opp_wins, played_cells) =
                    game_cache.get_board_progress(&state.status_map);
                let progress = played_cells as f32 / 9.0;
                let meta_weight = heuristic_config.meta_weight_base
                    + heuristic_config.meta_weight_progress_offset * progress;
                let threat_weight = 1.0 - meta_weight;
                let max_threat_score = (my_threat_sum + opp_threat_sum).max(1.0);
                let final_score = 0.5
                    + 0.5 * meta_weight * (my_wins as f32 - opp_wins as f32) / 9.0
                    + 0.5 * threat_weight * (my_threat_sum - opp_threat_sum) / max_threat_score;
                final_score.clamp(0.0, 1.0)
            }
        };
        let score = match state.last_player {
            TicTacToeStatus::Me => score,
            TicTacToeStatus::Opp => 1.0 - score,
            _ => unreachable!("Player is alway Me or Opp"),
        };
        heuristic_cache.insert_intermediate_score(state, score);
        if perspective_is_last_player {
            score
        } else {
            1.0 - score
        }
    }
    fn evaluate_move(
        state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
        mv: &<UltTTTMCTSGame<GC> as MCTSGame>::Move,
        game_cache: &mut <UltTTTMCTSGame<GC> as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> f32 {
        let new_state = UltTTTMCTSGame::apply_move(state, mv, game_cache);
        UltTTTHeuristic::evaluate_state(
            &new_state,
            game_cache,
            heuristic_cache,
            None,
            heuristic_config,
        )
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
type UltTTTMCTSGameNoGameCache = UltTTTMCTSGame<NoGameCache<UltTTT, UltTTTMove>>;
type HPWDefaultTTTNoGameCache =
    HeuristicProgressiveWidening<UltTTTMCTSGameNoGameCache, UltTTTHeuristic>;
struct UltTTTMCTSGame<GC: UltTTTGameCacheTrait + GameCache<UltTTT, UltTTTMove>> {
    phantom: std::marker::PhantomData<GC>,
}
impl<GC: UltTTTGameCacheTrait + GameCache<UltTTT, UltTTTMove>> MCTSGame for UltTTTMCTSGame<GC> {
    type State = UltTTT;
    type Move = UltTTTMove;
    type Player = TicTacToeStatus;
    type Cache = GC;
    type Config = UltTTTMCTSConfig;
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
        new_state
            .map
            .get_cell_mut(mv.status_index)
            .set_cell_value(mv.mini_board_index, state.current_player);
        let status = game_cache.get_status(new_state.map.get_cell(mv.status_index));
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
    fn evaluate(state: &Self::State, game_cache: &mut Self::Cache) -> Option<f32> {
        let mut status = game_cache.get_status(&state.status_map);
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
impl CellIndex3x3 {
    fn cell_weight(&self) -> f32 {
        match self {
            CellIndex3x3::MM => 4.0,
            CellIndex3x3::TL | CellIndex3x3::TR | CellIndex3x3::BL | CellIndex3x3::BR => 3.0,
            CellIndex3x3::TM | CellIndex3x3::ML | CellIndex3x3::MR | CellIndex3x3::BM => 2.0,
        }
    }
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
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
    nodes: Vec<PlainNode<G, UP, UC, EP, H>>,
    root_index: usize,
    mcts_config: G::Config,
    heuristic_config: H::Config,
    game_cache: G::Cache,
    heuristic_cache: H::Cache,
    phantom: std::marker::PhantomData<SP>,
}
impl<G, UP, UC, EP, H, SP> PlainMCTS<G, UP, UC, EP, H, SP>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
    fn new(mcts_config: G::Config, heuristic_config: H::Config) -> Self {
        Self {
            nodes: vec![],
            root_index: 0,
            mcts_config,
            heuristic_config,
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
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
{
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
        let expansion_policy = EP::new(
            state,
            &mut self.game_cache,
            &mut self.heuristic_cache,
            &self.heuristic_config,
        );
        self.nodes
            .push(PlainNode::root_node(state.clone(), expansion_policy));
        self.root_index = 0;
        false
    }
    fn iterate(&mut self) {
        let mut path = vec![self.root_index];
        let mut current_index = self.root_index;
        while !self.nodes[current_index].get_children().is_empty() {
            let parent_visits = self.nodes[current_index].get_visits();
            let num_parent_children = self.nodes[current_index].get_children().len();
            if self.nodes[current_index].expansion_policy.should_expand(
                parent_visits,
                num_parent_children,
                &self.mcts_config,
                &self.heuristic_config,
            ) {
                break;
            }
            let mut best_child_index = 0;
            let mut best_utc = f32::NEG_INFINITY;
            for vec_index in 0..num_parent_children {
                let child_index = self.nodes[current_index].get_children()[vec_index];
                let utc = self.nodes[child_index].calc_utc(
                    parent_visits,
                    G::perspective_player(),
                    &self.mcts_config,
                );
                if utc > best_utc {
                    best_utc = utc;
                    best_child_index = child_index;
                }
            }
            path.push(best_child_index);
            current_index = best_child_index;
        }
        let current_index = if (self.nodes[current_index].get_visits() == 0
            && current_index != self.root_index)
            || G::evaluate(self.nodes[current_index].get_state(), &mut self.game_cache).is_some()
        {
            current_index
        } else {
            let num_parent_children = self.nodes[current_index].get_children().len();
            let expandable_moves = self.nodes[current_index]
                .expandable_moves(&self.mcts_config, &self.heuristic_config);
            for mv in expandable_moves {
                let new_state =
                    G::apply_move(&self.nodes[current_index].state, &mv, &mut self.game_cache);
                let expansion_policy = EP::new(
                    &new_state,
                    &mut self.game_cache,
                    &mut self.heuristic_cache,
                    &self.heuristic_config,
                );
                let new_node = PlainNode::new(new_state, mv, expansion_policy);
                self.nodes.push(new_node);
                let child_index = self.nodes.len() - 1;
                self.nodes[current_index].add_child(child_index);
            }
            let child_index = *self.nodes[current_index]
                .get_children()
                .get(num_parent_children)
                .expect("No children at current node");
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
                Some(G::perspective_player()),
                &self.mcts_config,
                &self.heuristic_config,
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
#[derive(Debug, Clone, Copy)]
struct BaseConfig {
    exploration_constant: f32,
    progressive_widening_constant: f32,
    progressive_widening_exponent: f32,
    early_cut_off_depth: usize,
}
impl MCTSConfig for BaseConfig {
    fn exploration_constant(&self) -> f32 {
        self.exploration_constant
    }
    fn progressive_widening_constant(&self) -> f32 {
        self.progressive_widening_constant
    }
    fn progressive_widening_exponent(&self) -> f32 {
        self.progressive_widening_exponent
    }
    fn early_cut_off_depth(&self) -> usize {
        self.early_cut_off_depth
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
struct DynamicC {}
impl<G: MCTSGame> UCTPolicy<G> for DynamicC {
    fn exploration_score(visits: usize, parent_visits: usize, mcts_config: &G::Config) -> f32 {
        let dynamic_c = mcts_config.exploration_constant() / (1.0 + (visits as f32).sqrt());
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
        last_player: G::Player,
        perspective_player: G::Player,
    ) {
        self.exploitation =
            UP::exploitation_score(acc_value, visits, last_player, perspective_player);
    }
    fn get_exploitation(&self, _v: usize, _a: f32, _c: G::Player, _p: G::Player) -> f32 {
        self.exploitation
    }
    fn update_exploration(&mut self, visits: usize, parent_visits: usize, mcts_config: &G::Config) {
        if self.last_parent_visits != parent_visits {
            self.exploration = UP::exploration_score(visits, parent_visits, mcts_config);
            self.last_parent_visits = parent_visits;
        }
    }
    fn get_exploration(&self, _v: usize, _p: usize, _mc: &G::Config) -> f32 {
        self.exploration
    }
}
struct HeuristicProgressiveWidening<G: MCTSGame, H: Heuristic<G>> {
    unexpanded_moves: Vec<(f32, G::Move)>,
    phantom: std::marker::PhantomData<H>,
}
impl<G: MCTSGame, H: Heuristic<G>> HeuristicProgressiveWidening<G, H> {
    fn allowed_children(visits: usize, mcts_config: &G::Config) -> usize {
        if visits == 0 {
            1
        } else {
            (mcts_config.progressive_widening_constant()
                * (visits as f32).powf(mcts_config.progressive_widening_exponent()))
            .floor() as usize
        }
    }
    fn threshold(visits: usize, heuristic_config: &<H as Heuristic<G>>::Config) -> f32 {
        heuristic_config.progressive_widening_initial_threshold()
            * heuristic_config
                .progressive_widening_decay_rate()
                .powi(visits as i32)
    }
}
impl<G: MCTSGame, H: Heuristic<G>> ExpansionPolicy<G, H> for HeuristicProgressiveWidening<G, H> {
    fn new(
        state: &<G as MCTSGame>::State,
        game_cache: &mut <G as MCTSGame>::Cache,
        heuristic_cache: &mut <H as Heuristic<G>>::Cache,
        heuristic_config: &<H as Heuristic<G>>::Config,
    ) -> Self {
        let is_terminal = match game_cache.get_terminal_value(state) {
            Some(status) => status.is_some(),
            None => G::evaluate(state, game_cache).is_some(),
        };
        if is_terminal {
            return HeuristicProgressiveWidening {
                unexpanded_moves: vec![],
                phantom: std::marker::PhantomData,
            };
        }
        let unexpanded_moves = G::available_moves(state).collect::<Vec<_>>();
        let unexpanded_moves = H::sort_moves(
            state,
            unexpanded_moves,
            game_cache,
            heuristic_cache,
            heuristic_config,
        );
        HeuristicProgressiveWidening {
            unexpanded_moves,
            phantom: std::marker::PhantomData,
        }
    }
    fn should_expand(
        &self,
        visits: usize,
        num_parent_children: usize,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> bool {
        let threshold = Self::threshold(visits, heuristic_config);
        num_parent_children < Self::allowed_children(visits, mcts_config)
            && self
                .unexpanded_moves
                .iter()
                .any(|(score, _)| *score >= threshold)
    }
    fn expandable_moves(
        &mut self,
        visits: usize,
        num_parent_children: usize,
        _state: &<G as MCTSGame>::State,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> Box<dyn Iterator<Item = <G as MCTSGame>::Move> + '_> {
        let allowed_children = Self::allowed_children(visits, mcts_config);
        if num_parent_children < allowed_children && !self.unexpanded_moves.is_empty() {
            let num_expandable_moves = self
                .unexpanded_moves
                .len()
                .min(allowed_children - num_parent_children);
            let threshold = Self::threshold(visits, heuristic_config);
            let cutoff_index = self
                .unexpanded_moves
                .iter()
                .position(|(score, _)| *score < threshold)
                .unwrap_or(self.unexpanded_moves.len());
            let selected_count = cutoff_index.min(num_expandable_moves).max(1);
            Box::new(
                self.unexpanded_moves
                    .drain(..selected_count)
                    .map(|(_, mv)| mv),
            )
        } else {
            Box::new(std::iter::empty())
        }
    }
}
struct HeuristicCutoff {}
impl<G: MCTSGame, H: Heuristic<G>> SimulationPolicy<G, H> for HeuristicCutoff {
    fn should_cutoff(
        state: &G::State,
        depth: usize,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut H::Cache,
        perspective_player: Option<G::Player>,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> Option<f32> {
        let heuristic = H::evaluate_state(
            state,
            game_cache,
            heuristic_cache,
            perspective_player,
            heuristic_config,
        );
        if depth >= mcts_config.early_cut_off_depth()
            || heuristic <= heuristic_config.early_cut_off_lower_bound()
            || heuristic >= heuristic_config.early_cut_off_upper_bound()
        {
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
#[derive(Debug, Clone, Copy)]
struct BaseHeuristicConfig {
    progressive_widening_initial_threshold: f32,
    progressive_widening_decay_rate: f32,
    early_cut_off_lower_bound: f32,
    early_cut_off_upper_bound: f32,
}
impl Default for BaseHeuristicConfig {
    fn default() -> Self {
        BaseHeuristicConfig {
            progressive_widening_initial_threshold: 0.8,
            progressive_widening_decay_rate: 0.95,
            early_cut_off_lower_bound: 0.05,
            early_cut_off_upper_bound: 0.95,
        }
    }
}
impl HeuristicConfig for BaseHeuristicConfig {
    fn progressive_widening_initial_threshold(&self) -> f32 {
        self.progressive_widening_initial_threshold
    }
    fn progressive_widening_decay_rate(&self) -> f32 {
        self.progressive_widening_decay_rate
    }
    fn early_cut_off_lower_bound(&self) -> f32 {
        self.early_cut_off_lower_bound
    }
    fn early_cut_off_upper_bound(&self) -> f32 {
        self.early_cut_off_upper_bound
    }
}
struct NoHeuristic {}
impl<G: MCTSGame> Heuristic<G> for NoHeuristic {
    type Cache = NoHeuristicCache<G::State, G::Move>;
    type Config = BaseHeuristicConfig;
    fn evaluate_state(
        state: &<G as MCTSGame>::State,
        game_cache: &mut <G as MCTSGame>::Cache,
        _heuristic_cache: &mut Self::Cache,
        _perspective_player: Option<G::Player>,
        _heuristic_config: &Self::Config,
    ) -> f32 {
        G::evaluate(state, game_cache).unwrap_or(0.5)
    }
    fn evaluate_move(
        _state: &<G as MCTSGame>::State,
        _mv: &<G as MCTSGame>::Move,
        _game_cache: &mut <G as MCTSGame>::Cache,
        _heuristic_cache: &mut Self::Cache,
        _heuristic_config: &Self::Config,
    ) -> f32 {
        0.0
    }
}
struct PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
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
    EP: ExpansionPolicy<G, H>,
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
    fn expandable_moves(
        &mut self,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> Vec<G::Move> {
        let mut expandable_moves = self
            .expansion_policy
            .expandable_moves(
                self.visits,
                self.children.len(),
                &self.state,
                mcts_config,
                heuristic_config,
            )
            .collect::<Vec<_>>();
        expandable_moves.shuffle(&mut rand::thread_rng());
        expandable_moves
    }
}
impl<G, UP, UC, EP, H> MCTSNode<G> for PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
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
            G::last_player(&self.state),
            G::perspective_player(),
        );
    }
    fn calc_utc(
        &mut self,
        parent_visits: usize,
        perspective_player: G::Player,
        mcts_config: &G::Config,
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
}
trait MCTSPlayer: PartialEq {
    fn next(&self) -> Self;
}
trait MCTSConfig {
    fn exploration_constant(&self) -> f32;
    fn progressive_widening_constant(&self) -> f32;
    fn progressive_widening_exponent(&self) -> f32;
    fn early_cut_off_depth(&self) -> usize;
}
trait MCTSGame: Sized {
    type State: Clone + PartialEq;
    type Move;
    type Player: MCTSPlayer;
    type Cache: GameCache<Self::State, Self::Move>;
    type Config: MCTSConfig;
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
trait MCTSNode<G: MCTSGame> {
    fn get_state(&self) -> &G::State;
    fn get_move(&self) -> Option<&G::Move> {
        None
    }
    fn get_visits(&self) -> usize;
    fn get_accumulated_value(&self) -> f32;
    fn update_stats(&mut self, result: f32);
    fn calc_utc(
        &mut self,
        parent_visits: usize,
        perspective_player: G::Player,
        mcts_config: &G::Config,
    ) -> f32;
}
trait MCTSAlgo<G: MCTSGame> {
    fn set_root(&mut self, state: &G::State) -> bool;
    fn iterate(&mut self);
    fn select_move(&self) -> &G::Move;
}
trait UCTPolicy<G: MCTSGame> {
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
    fn exploration_score(visits: usize, parent_visits: usize, mcts_config: &G::Config) -> f32 {
        mcts_config.exploration_constant() * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}
trait UTCCache<G: MCTSGame, UP: UCTPolicy<G>> {
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
    fn update_exploration(&mut self, visits: usize, parent_visits: usize, mcts_config: &G::Config);
    fn get_exploration(&self, visits: usize, parent_visits: usize, mcts_config: &G::Config) -> f32;
}
trait ExpansionPolicy<G: MCTSGame, H: Heuristic<G>> {
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
        _mcts_config: &G::Config,
        _heuristic_config: &H::Config,
    ) -> bool {
        false
    }
    fn expandable_moves<'a>(
        &'a mut self,
        _visits: usize,
        _num_parent_children: usize,
        state: &'a G::State,
        _mcts_config: &G::Config,
        _heuristic_config: &H::Config,
    ) -> Box<dyn Iterator<Item = G::Move> + 'a> {
        G::available_moves(state)
    }
}
trait SimulationPolicy<G: MCTSGame, H: Heuristic<G>> {
    fn should_cutoff(
        _state: &G::State,
        _depth: usize,
        _game_cache: &mut G::Cache,
        _heuristic_cache: &mut H::Cache,
        _perspective_player: Option<G::Player>,
        _mcts_config: &G::Config,
        _heuristic_config: &H::Config,
    ) -> Option<f32> {
        None
    }
}
trait HeuristicConfig {
    fn progressive_widening_initial_threshold(&self) -> f32;
    fn progressive_widening_decay_rate(&self) -> f32;
    fn early_cut_off_upper_bound(&self) -> f32;
    fn early_cut_off_lower_bound(&self) -> f32;
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
    fn get_threats(&self) -> (usize, usize) {
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
        (me_threats.len(), opp_threats.len())
    }
    fn get_meta_cell_threats(&self, cell: CellIndex3x3) -> (u8, u8, u8, u8) {
        if self.get_cell_value(cell).is_not_vacant() {
            return (0, 0, 0, 0);
        }
        let mut my_meta_threats = 0;
        let mut my_meta_small_threats = 0;
        let mut opp_meta_threats = 0;
        let mut opp_meta_small_threats = 0;
        for score_line in Self::SCORE_LINES.iter() {
            if !score_line.contains(&cell) {
                continue;
            }
            let threat: i8 = score_line
                .iter()
                .map(|&c| self.get_cell_value(c) as i8)
                .sum();
            match threat {
                2 => my_meta_threats += 1,
                1 => my_meta_small_threats += 1,
                -1 => opp_meta_small_threats += 1,
                -2 => opp_meta_threats += 1,
                _ => (),
            }
        }
        (
            my_meta_threats,
            my_meta_small_threats,
            opp_meta_threats,
            opp_meta_small_threats,
        )
    }
}
