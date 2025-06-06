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
        UltTTTMCTSConfig::new_optimized(),
        UltTTTHeuristicConfig::new_optimized(),
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
#[derive(Debug, Clone, Copy, PartialEq)]
struct UltTTTMCTSConfig {
    base_config: BaseConfig,
}
impl UltTTTMCTSConfig {
    fn new_optimized() -> Self {
        UltTTTMCTSConfig {
            base_config: BaseConfig {
                exploration_constant: 1.298,
                progressive_widening_constant: 1.602,
                progressive_widening_exponent: 0.333,
                early_cut_off_depth: 15,
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
#[derive(Debug, Clone, Copy, PartialEq)]
struct UltTTTHeuristicConfig {
    base_config: BaseHeuristicConfig,
    control_base_weight: f32,
    control_progress_offset: f32,
    control_local_steepness: f32,
    control_global_steepness: f32,
    meta_cell_big_threat: f32,
    meta_cell_small_threat: f32,
    threat_steepness: f32,
    constraint_factor: f32,
    free_choice_constraint_factor: f32,
    direct_loss_value: f32,
}
impl UltTTTHeuristicConfig {
    fn new_optimized() -> Self {
        UltTTTHeuristicConfig {
            base_config: BaseHeuristicConfig {
                progressive_widening_initial_threshold: 0.676,
                progressive_widening_decay_rate: 0.814,
                early_cut_off_lower_bound: 0.068,
                early_cut_off_upper_bound: 0.947,
            },
            control_base_weight: 0.538,
            control_progress_offset: 0.228,
            control_local_steepness: 0.099,
            control_global_steepness: 0.599,
            meta_cell_big_threat: 2.143,
            meta_cell_small_threat: 0.746,
            threat_steepness: 0.171,
            constraint_factor: 0.128,
            free_choice_constraint_factor: 0.982,
            direct_loss_value: 0.0,
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
            control_base_weight: 0.3,
            control_progress_offset: 0.4,
            control_local_steepness: 0.15,
            control_global_steepness: 0.3,
            meta_cell_big_threat: 3.0,
            meta_cell_small_threat: 1.5,
            threat_steepness: 0.5,
            constraint_factor: 1.5,
            free_choice_constraint_factor: 1.5,
            direct_loss_value: 0.01,
        }
    }
}
trait UltTTTGameCacheTrait {
    fn get_status(&mut self, board: &TicTacToeGameData) -> TicTacToeStatus;
    fn get_board_progress(&mut self, board: &TicTacToeGameData) -> (usize, usize, usize);
    fn get_board_control(&mut self, board: &TicTacToeGameData) -> (f32, f32);
    fn get_board_threats(
        &mut self,
        board: &TicTacToeGameData,
    ) -> (HashSet<CellIndex3x3>, HashSet<CellIndex3x3>);
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
    fn get_board_control(&mut self, board: &TicTacToeGameData) -> (f32, f32) {
        board.get_board_control()
    }
    fn get_board_threats(
        &mut self,
        board: &TicTacToeGameData,
    ) -> (HashSet<CellIndex3x3>, HashSet<CellIndex3x3>) {
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
    fn get_constraint_factors(
        last_player: TicTacToeStatus,
        my_threats_of_mini_board: &HashSet<CellIndex3x3>,
        my_meta_threats: &HashSet<CellIndex3x3>,
        opp_threats_of_mini_board: &HashSet<CellIndex3x3>,
        opp_meta_threats: &HashSet<CellIndex3x3>,
        mini_board_index: CellIndex3x3,
        constraint_factor: f32,
    ) -> Option<(f32, f32)> {
        if match last_player {
            TicTacToeStatus::Me => {
                !opp_threats_of_mini_board.is_empty()
                    && opp_meta_threats.contains(&mini_board_index)
            }
            TicTacToeStatus::Opp => {
                !my_threats_of_mini_board.is_empty() && my_meta_threats.contains(&mini_board_index)
            }
            _ => unreachable!("Player is always Me or Opp"),
        } {
            return None;
        }
        match last_player {
            TicTacToeStatus::Me => {
                let my_threat_overlap_ratio =
                    UltTTTHeuristic::get_threat_overlap_ratio_for_last_player(
                        my_meta_threats,
                        opp_threats_of_mini_board,
                    );
                Some((
                    1.0 + my_threat_overlap_ratio * constraint_factor,
                    1.0 + (1.0 - my_threat_overlap_ratio) * constraint_factor,
                ))
            }
            TicTacToeStatus::Opp => {
                let opp_threat_overlap_ratio =
                    UltTTTHeuristic::get_threat_overlap_ratio_for_last_player(
                        opp_meta_threats,
                        my_threats_of_mini_board,
                    );
                Some((
                    1.0 + (1.0 - opp_threat_overlap_ratio) * constraint_factor,
                    1.0 + opp_threat_overlap_ratio * constraint_factor,
                ))
            }
            _ => unreachable!("Player is always Me or Opp"),
        }
    }
    fn get_threat_overlap_ratio_for_last_player(
        last_player_meta_threats: &HashSet<CellIndex3x3>,
        current_player_threats: &HashSet<CellIndex3x3>,
    ) -> f32 {
        if current_player_threats.is_empty() {
            return 0.0;
        }
        let num_last_player_back_to_threat_line = last_player_meta_threats
            .intersection(current_player_threats)
            .count();
        num_last_player_back_to_threat_line as f32 / current_player_threats.len() as f32
    }
    fn normalized_tanh(my_score: f32, opp_score: f32, steepness: f32) -> f32 {
        let delta_score = steepness * (my_score - opp_score);
        (delta_score.tanh() + 1.0) / 2.0
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
                let mut my_control_sum = 0.0;
                let mut opp_control_sum = 0.0;
                let mut my_threat_sum = 0.0;
                let mut opp_threat_sum = 0.0;
                let (my_meta_threats, opp_meta_threats) =
                    game_cache.get_board_threats(&state.status_map);
                for (status_index, status) in state.status_map.iter_map() {
                    match status {
                        TicTacToeStatus::Tie => {
                            continue;
                        }
                        TicTacToeStatus::Me => {
                            my_control_sum += status_index.cell_weight();
                        }
                        TicTacToeStatus::Opp => {
                            opp_control_sum += status_index.cell_weight();
                        }
                        TicTacToeStatus::Vacant => {
                            let (my_control, opp_control) =
                                game_cache.get_board_control(state.map.get_cell(status_index));
                            let my_control_score = UltTTTHeuristic::normalized_tanh(
                                my_control,
                                opp_control,
                                heuristic_config.control_local_steepness,
                            );
                            let opp_control_score = 1.0 - my_control_score;
                            my_control_sum += my_control_score * status_index.cell_weight();
                            opp_control_sum += opp_control_score * status_index.cell_weight();
                            let (my_threats, opp_threats) =
                                game_cache.get_board_threats(state.map.get_cell(status_index));
                            let cell_weight = status_index.cell_weight();
                            let (
                                num_my_meta_threats,
                                mum_my_meta_small_threats,
                                num_opp_meta_threats,
                                num_opp_meta_small_threats,
                            ) = game_cache.get_meta_cell_threats(&state.status_map, status_index);
                            let my_meta_factor = 1.0
                                + heuristic_config.meta_cell_big_threat
                                    * num_my_meta_threats as f32
                                + heuristic_config.meta_cell_small_threat
                                    * mum_my_meta_small_threats as f32;
                            let opp_meta_factor = 1.0
                                + heuristic_config.meta_cell_big_threat
                                    * num_opp_meta_threats as f32
                                + heuristic_config.meta_cell_small_threat
                                    * num_opp_meta_small_threats as f32;
                            let (my_constraint_factor, opp_constraint_factor) = match state
                                .next_action_constraint
                            {
                                NextActionConstraint::MiniBoard(next_board) => {
                                    if status_index == next_board {
                                        match UltTTTHeuristic::get_constraint_factors(
                                            state.last_player,
                                            &my_threats,
                                            &my_meta_threats,
                                            &opp_threats,
                                            &opp_meta_threats,
                                            status_index,
                                            heuristic_config.constraint_factor,
                                        ) {
                                            Some((my_factor, opp_factor)) => {
                                                (my_factor, opp_factor)
                                            }
                                            None => {
                                                return if perspective_is_last_player {
                                                    heuristic_config.direct_loss_value
                                                } else {
                                                    1.0 - heuristic_config.direct_loss_value
                                                };
                                            }
                                        }
                                    } else {
                                        (1.0, 1.0)
                                    }
                                }
                                NextActionConstraint::None => {
                                    match UltTTTHeuristic::get_constraint_factors(
                                        state.last_player,
                                        &my_threats,
                                        &my_meta_threats,
                                        &opp_threats,
                                        &opp_meta_threats,
                                        status_index,
                                        heuristic_config.free_choice_constraint_factor,
                                    ) {
                                        Some((my_factor, opp_factor)) => (my_factor, opp_factor),
                                        None => {
                                            return if perspective_is_last_player {
                                                heuristic_config.direct_loss_value
                                            } else {
                                                1.0 - heuristic_config.direct_loss_value
                                            };
                                        }
                                    }
                                }
                                NextActionConstraint::Init => {
                                    unreachable!("Init is reserved for initial tree root node.")
                                }
                            };
                            my_threat_sum += my_constraint_factor
                                * my_meta_factor
                                * cell_weight
                                * my_threats.len() as f32;
                            opp_threat_sum += opp_constraint_factor
                                * opp_meta_factor
                                * cell_weight
                                * opp_threats.len() as f32;
                        }
                    }
                }
                let (_, _, played_cells) = game_cache.get_board_progress(&state.status_map);
                let progress = played_cells as f32 / 9.0;
                let control_weight = heuristic_config.control_base_weight
                    + heuristic_config.control_progress_offset * progress;
                let threat_weight = 1.0 - control_weight;
                control_weight
                    * UltTTTHeuristic::normalized_tanh(
                        my_control_sum,
                        opp_control_sum,
                        heuristic_config.control_global_steepness,
                    )
                    + threat_weight
                        * UltTTTHeuristic::normalized_tanh(
                            my_threat_sum,
                            opp_threat_sum,
                            heuristic_config.threat_steepness,
                        )
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
type PlainMCTS<G, UP, UC, EP, H, SP> = BaseMCTS<
    G,
    PlainNode<G, UP, UC, EP, H>,
    PlainTree<G, UP, UC, EP, H>,
    UP,
    UC,
    EP,
    H,
    SP,
    NoTranspositionTable,
>;
struct BaseMCTS<G, N, T, UP, UC, EP, H, SP, TT>
where
    G: MCTSGame,
    G::State: Eq + std::hash::Hash,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
    TT: TranspositionTable<G, N, T, EP, H>,
{
    tree: T,
    mcts_config: G::Config,
    heuristic_config: H::Config,
    game_cache: G::Cache,
    heuristic_cache: H::Cache,
    transposition_table: TT,
    phantom: std::marker::PhantomData<(N, UP, UC, EP, SP)>,
}
impl<G, N, T, UP, UC, EP, H, SP, TT> BaseMCTS<G, N, T, UP, UC, EP, H, SP, TT>
where
    G: MCTSGame,
    G::State: Eq + std::hash::Hash,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
    TT: TranspositionTable<G, N, T, EP, H>,
{
    fn new(mcts_config: G::Config, heuristic_config: H::Config) -> Self {
        Self {
            tree: T::new(),
            mcts_config,
            heuristic_config,
            game_cache: G::Cache::new(),
            heuristic_cache: H::Cache::new(),
            transposition_table: TT::new(),
            phantom: std::marker::PhantomData,
        }
    }
}
impl<G, N, T, UP, UC, EP, H, SP, TT> MCTSAlgo<G> for BaseMCTS<G, N, T, UP, UC, EP, H, SP, TT>
where
    G: MCTSGame,
    G::State: Eq + std::hash::Hash,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
    SP: SimulationPolicy<G, H>,
    TT: TranspositionTable<G, N, T, EP, H>,
{
    fn set_root(&mut self, state: &G::State) -> bool {
        if let Some(root_id) = self.tree.root_id() {
            if let Some(node_of_state_id) = self.transposition_table.get(state) {
                self.tree.set_root(*node_of_state_id);
                return true;
            }
            if let Some((new_root_id, _)) = self
                .tree
                .get_children(root_id)
                .iter()
                .flat_map(|&(my_move_node_id, _)| {
                    self.tree.get_children(my_move_node_id).iter().map(
                        |&(opponent_move_node_id, _)| {
                            (
                                opponent_move_node_id,
                                self.tree.get_node(opponent_move_node_id).get_state(),
                            )
                        },
                    )
                })
                .find(|(_, opponent_move_node_state)| *opponent_move_node_state == state)
            {
                self.tree.set_root(new_root_id);
                return true;
            }
        }
        let expansion_policy = EP::new(
            state,
            &mut self.game_cache,
            &mut self.heuristic_cache,
            &self.heuristic_config,
        );
        let new_root = N::new(state.clone(), expansion_policy);
        let root_id = self.tree.init_root(new_root);
        self.transposition_table = TT::new();
        self.transposition_table.insert(state.clone(), root_id);
        false
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
        let mut new_children: Vec<T::ID> = Vec::new();
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
                    let new_node = N::new(new_state.clone(), expansion_policy);
                    let new_child_id = tree.add_child(current_id, mv, new_node);
                    transposition_table.insert(new_state, new_child_id);
                    new_children.push(new_child_id);
                }
                let Some (child_index) = new_children . get (0) else { continue ; } ;
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
fn back_propagation<G, N, T, EP, H>(tree: &mut T, path: &[T::ID], result: f32)
where
    G: MCTSGame,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    for &node_id in path.iter().rev() {
        tree.get_node_mut(node_id).update_stats(result);
    }
}
use rand::prelude::SliceRandom;
use std::collections::HashMap;
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
struct NoTranspositionTable {}
impl<G, N, T, EP, H> TranspositionTable<G, N, T, EP, H> for NoTranspositionTable
where
    G: MCTSGame,
    G::State: Eq + std::hash::Hash,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new() -> Self {
        NoTranspositionTable {}
    }
}
struct TranspositionHashMap<G, N, T, EP, H>
where
    G: MCTSGame,
    G::State: Eq + std::hash::Hash,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    table: HashMap<G::State, T::ID>,
}
impl<G, N, T, EP, H> TranspositionTable<G, N, T, EP, H> for TranspositionHashMap<G, N, T, EP, H>
where
    G: MCTSGame,
    G::State: Eq + std::hash::Hash,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }
    fn get(&self, state: &<G as MCTSGame>::State) -> Option<&T::ID> {
        self.table.get(state)
    }
    fn insert(&mut self, state: <G as MCTSGame>::State, value: T::ID) {
        self.table.insert(state, value);
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
    utc_cache: UC,
    expansion_policy: EP,
    phantom: std::marker::PhantomData<(UP, H)>,
}
impl<G, UP, UC, EP, H> MCTSNode<G, EP, H> for PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new(state: G::State, expansion_policy: EP) -> Self {
        PlainNode {
            expansion_policy,
            state,
            visits: 0,
            accumulated_value: 0.0,
            utc_cache: UC::new(),
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
    fn should_expand(
        &self,
        visits: usize,
        num_parent_children: usize,
        mcts_config: &G::Config,
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
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> Vec<G::Move> {
        let mut expandable_moves = self
            .expansion_policy
            .expandable_moves(
                self.visits,
                num_parent_children,
                &self.state,
                mcts_config,
                heuristic_config,
            )
            .collect::<Vec<_>>();
        expandable_moves.shuffle(&mut rand::thread_rng());
        expandable_moves
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
trait MCTSTree<G, N, EP, H>
where
    G: MCTSGame,
    N: MCTSNode<G, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    type ID: Copy + Eq + std::fmt::Debug;
    fn new() -> Self;
    fn init_root(&mut self, root_value: N) -> Self::ID;
    fn set_root(&mut self, new_root_id: Self::ID);
    fn root_id(&self) -> Option<Self::ID>;
    fn get_node(&self, id: Self::ID) -> &N;
    fn get_node_mut(&mut self, id: Self::ID) -> &mut N;
    fn add_child(&mut self, parent_id: Self::ID, mv: G::Move, child_value: N) -> Self::ID;
    fn link_child(&mut self, parent_id: Self::ID, mv: G::Move, child_id: Self::ID);
    fn get_children(&self, id: Self::ID) -> &[(Self::ID, G::Move)];
}
trait TranspositionTable<G, N, T, EP, H>
where
    G: MCTSGame,
    G::State: Eq + std::hash::Hash,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new() -> Self;
    fn get(&self, _state: &G::State) -> Option<&T::ID> {
        None
    }
    fn insert(&mut self, _state: G::State, _value: T::ID) {}
}
trait MCTSNode<G: MCTSGame, EP: ExpansionPolicy<G, H>, H: Heuristic<G>>
where
    G: MCTSGame,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new(state: G::State, expansion_policy: EP) -> Self;
    fn get_state(&self) -> &G::State;
    fn get_visits(&self) -> usize;
    fn get_accumulated_value(&self) -> f32;
    fn update_stats(&mut self, result: f32);
    fn calc_utc(
        &mut self,
        parent_visits: usize,
        perspective_player: G::Player,
        mcts_config: &G::Config,
    ) -> f32;
    fn should_expand(
        &self,
        visits: usize,
        num_parent_children: usize,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> bool;
    fn expandable_moves(
        &mut self,
        num_parent_children: usize,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> Vec<G::Move>;
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
type PlainTree<G, UP, UC, EP, H> = BaseTree<G, PlainNode<G, UP, UC, EP, H>, EP, H>;
struct BaseTree<G, N, EP, H>
where
    G: MCTSGame,
    N: MCTSNode<G, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    nodes: Vec<N>,
    edges: Vec<Vec<(usize, G::Move)>>,
    root_id: usize,
    phantom: std::marker::PhantomData<(G, N, EP, H)>,
}
impl<G, N, EP, H> MCTSTree<G, N, EP, H> for BaseTree<G, N, EP, H>
where
    G: MCTSGame,
    N: MCTSNode<G, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    type ID = usize;
    fn new() -> Self {
        BaseTree {
            nodes: vec![],
            edges: vec![],
            root_id: 0,
            phantom: std::marker::PhantomData,
        }
    }
    fn init_root(&mut self, root_value: N) -> Self::ID {
        self.nodes.clear();
        self.edges.clear();
        self.nodes.push(root_value);
        self.edges.push(vec![]);
        self.root_id = 0;
        self.root_id
    }
    fn set_root(&mut self, new_root_id: Self::ID) {
        self.root_id = new_root_id;
    }
    fn root_id(&self) -> Option<Self::ID> {
        if self.nodes.is_empty() {
            None
        } else {
            Some(self.root_id)
        }
    }
    fn get_node(&self, id: Self::ID) -> &N {
        &self.nodes[id]
    }
    fn get_node_mut(&mut self, id: Self::ID) -> &mut N {
        &mut self.nodes[id]
    }
    fn add_child(&mut self, parent_id: Self::ID, mv: G::Move, child_value: N) -> usize {
        let child_id = self.nodes.len();
        self.nodes.push(child_value);
        self.edges.push(vec![]);
        self.link_child(parent_id, mv, child_id);
        child_id
    }
    fn link_child(&mut self, parent_id: Self::ID, mv: <G as MCTSGame>::Move, child_id: Self::ID) {
        let edge = self
            .edges
            .get_mut(parent_id)
            .expect("Expected edges of parent.");
        edge.push((child_id, mv));
    }
    fn get_children(&self, id: Self::ID) -> &[(Self::ID, <G as MCTSGame>::Move)] {
        &self.edges[id][..]
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
    fn get_threats(&self) -> (HashSet<CellIndex3x3>, HashSet<CellIndex3x3>) {
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
        (me_threats, opp_threats)
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
    fn get_board_control(&self) -> (f32, f32) {
        self.map.iterate().fold(
            (0.0, 0.0),
            |(mut my_control, mut opp_control), (cell, status)| {
                match status {
                    TicTacToeStatus::Me => {
                        my_control += cell.cell_weight();
                    }
                    TicTacToeStatus::Opp => {
                        opp_control += cell.cell_weight();
                    }
                    TicTacToeStatus::Vacant | TicTacToeStatus::Tie => {}
                }
                (my_control, opp_control)
            },
        )
    }
}
