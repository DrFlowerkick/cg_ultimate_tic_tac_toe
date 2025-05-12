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
    last_player: TicTacToeStatus,
}

impl UltTTT {
    pub fn new() -> Self {
        UltTTT {
            map: MyMap3x3::new(),
            status_map: TicTacToeGameData::new(),
            next_action_constraint: NextActionConstraint::Init,
            current_player: TicTacToeStatus::Me,
            last_player: TicTacToeStatus::Me,
        }
    }
    pub fn set_current_player(&mut self, player: TicTacToeStatus) {
        self.current_player = player;
    }
    pub fn next_player(&mut self) {
        self.last_player = self.current_player;
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

pub type UltTTTMCTSGameWithGameCache = UltTTTMCTSGame<UltTTTGameCache>;
pub type UltTTTMCTSGameNoGameCache = UltTTTMCTSGame<NoGameCache<UltTTT, UltTTTMove>>;

pub type HPWDefaultTTTWithGameCache = ProgressiveWidening<UltTTTMCTSGameWithGameCache>;
pub type HPWDefaultTTTNoGameCache = HeuristicProgressiveWidening<UltTTTMCTSGameNoGameCache>;

pub struct UltTTTMCTSConfig {
    pub base_config: BaseConfig,
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

pub struct UltTTTMCTSGame<GC: UltTTTGameCacheTrait + GameCache<UltTTT, UltTTTMove>> {
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
        // apply the move for current player
        new_state
            .map
            .get_cell_mut(mv.status_index)
            .set_cell_value(mv.mini_board_index, state.current_player);
        let status = game_cache.get_status(new_state.map.get_cell(mv.status_index));
        if !status.is_vacant() {
            new_state.status_map.set_cell_value(mv.status_index, status);
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
        let mut status = game_cache.get_status(&state.status_map);
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
    fn last_player(state: &Self::State) -> Self::Player {
        state.last_player
    }
    fn perspective_player() -> Self::Player {
        TicTacToeStatus::Me
    }
}

pub struct UltTTTGameCache {
    pub cache: HashMap<TicTacToeGameData, BoardAnalysis>,
    pub usage: usize,
}

impl UltTTTGameCache {
    pub fn cache_board_analysis(&mut self, board: &TicTacToeGameData) -> BoardAnalysis {
        if let Some(cached_analysis) = self.cache.get(board) {
            self.usage += 1;
            return *cached_analysis;
        }
        let board_analysis = board.board_analysis();
        self.cache.insert(*board, board_analysis);
        board_analysis
    }
}

impl GameCache<UltTTT, UltTTTMove> for UltTTTGameCache {
    fn new() -> Self {
        UltTTTGameCache {
            cache: HashMap::new(),
            usage: 0,
        }
    }
}

pub trait UltTTTGameCacheTrait {
    fn get_status(&mut self, board: &TicTacToeGameData) -> TicTacToeStatus;
    fn get_board_wins(&mut self, board: &TicTacToeGameData) -> (f32, f32);
    fn get_board_threats(&mut self, board: &TicTacToeGameData) -> (f32, f32);
    fn get_meta_cell_threats(
        &mut self,
        board: &TicTacToeGameData,
        index: CellIndex3x3,
    ) -> (i8, i8, i8, i8);
}

impl UltTTTGameCacheTrait for UltTTTGameCache {
    fn get_status(&mut self, board: &TicTacToeGameData) -> TicTacToeStatus {
        self.cache_board_analysis(board).status
    }
    fn get_board_wins(&mut self, board: &TicTacToeGameData) -> (f32, f32) {
        let board_analysis = self.cache_board_analysis(board);
        (board_analysis.my_cells, board_analysis.opp_cells)
    }
    fn get_board_threats(&mut self, board: &TicTacToeGameData) -> (f32, f32) {
        let board_analysis = self.cache_board_analysis(board);
        (board_analysis.my_threats, board_analysis.opp_threats)
    }
    fn get_meta_cell_threats(
        &mut self,
        board: &TicTacToeGameData,
        index: CellIndex3x3,
    ) -> (i8, i8, i8, i8) {
        let board_analysis = self.cache_board_analysis(board);
        *board_analysis.meta_cell_threats.get_cell(index)
    }
}

impl UltTTTGameCacheTrait for NoGameCache<UltTTT, UltTTTMove> {
    fn get_status(&mut self, board: &TicTacToeGameData) -> TicTacToeStatus {
        board.get_status()
    }
    fn get_board_wins(&mut self, board: &TicTacToeGameData) -> (f32, f32) {
        let my_wins = board.count_me_cells() as f32;
        let opp_wins = board.count_opp_cells() as f32;
        (my_wins, opp_wins)
    }
    fn get_board_threats(&mut self, board: &TicTacToeGameData) -> (f32, f32) {
        let (my_threats, opp_threats) = board.get_threats();
        (my_threats as f32, opp_threats as f32)
    }
    fn get_meta_cell_threats(
        &mut self,
        board: &TicTacToeGameData,
        index: CellIndex3x3,
    ) -> (i8, i8, i8, i8) {
        board.get_meta_cell_threats(index)
    }
}

pub struct UltTTTHeuristicConfig {
    pub base_config: BaseHeuristicConfig,
    pub meta_weight_base: f32,
    pub meta_weight_progress_offset: f32,
    pub meta_cell_big_threat: f32,
    pub meta_cell_small_threat: f32,
    pub constraint_factor: f32,
    pub free_choice_constraint_factor: f32,
    pub evaluate_state_recursive_alpha_reduction_factor: f32,
    pub evaluate_state_recursive_early_exit_threshold: f32,
}

impl HeuristicConfig for UltTTTHeuristicConfig {
    fn early_cut_off_lower_bound(&self) -> f32 {
        self.base_config.early_cut_off_lower_bound
    }
    fn early_cut_off_upper_bound(&self) -> f32 {
        self.base_config.early_cut_off_upper_bound
    }
    fn evaluate_state_recursive_depth(&self) -> usize {
        self.base_config.evaluate_state_recursive_depth
    }
    fn evaluate_state_recursive_alpha(&self) -> f32 {
        self.base_config.evaluate_state_recursive_alpha
    }
}

impl Default for UltTTTHeuristicConfig {
    fn default() -> Self {
        UltTTTHeuristicConfig {
            base_config: BaseHeuristicConfig {
                early_cut_off_lower_bound: 0.05,
                early_cut_off_upper_bound: 0.95,
                evaluate_state_recursive_depth: 0,
                evaluate_state_recursive_alpha: 0.7,
            },
            meta_weight_base: 0.3,
            meta_weight_progress_offset: 0.4,
            meta_cell_big_threat: 3.0,
            meta_cell_small_threat: 1.5,
            constraint_factor: 1.5,
            free_choice_constraint_factor: 1.5,
            evaluate_state_recursive_alpha_reduction_factor: 0.9,
            evaluate_state_recursive_early_exit_threshold: 0.95,
        }
    }
}

pub struct UltTTTHeuristic {}

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
        if let Some(score) = heuristic_cache.get_intermediate_score(state) {
            return score;
        }
        let player = match perspective_player {
            Some(player) => player,
            None => state.last_player,
        };
        let score = match UltTTTMCTSGame::evaluate(state, game_cache) {
            Some(value) => value,
            None => {
                // meta progress: wins on status_map
                let (my_wins, opp_wins) = game_cache.get_board_wins(&state.status_map);
                // mini board threats
                let mut my_threat_sum = 0.0;
                let mut opp_threat_sum = 0.0;
                for (status_index, _) in state.status_map.iter_map().filter(|(_, c)| c.is_vacant())
                {
                    let constraint_factor = match state.next_action_constraint {
                        NextActionConstraint::MiniBoard(next_board) => {
                            if status_index == next_board {
                                heuristic_config.constraint_factor
                            } else {
                                1.0
                            }
                        }
                        NextActionConstraint::None => {
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
                    let cell_weight = status_index.cell_weight();
                    let (
                        my_meta_threats,
                        my_meta_small_threats,
                        opp_meta_threats,
                        opp_meta_small_threats,
                    ) = game_cache.get_meta_cell_threats(&state.status_map, status_index);
                    let my_meta_factor = heuristic_config.meta_cell_big_threat
                        * my_meta_threats as f32
                        + heuristic_config.meta_cell_small_threat * my_meta_small_threats as f32;
                    let opp_meta_factor = heuristic_config.meta_cell_big_threat
                        * opp_meta_threats as f32
                        + heuristic_config.meta_cell_small_threat * opp_meta_small_threats as f32;
                    let (my_threats, opp_threats) =
                        game_cache.get_board_threats(state.map.get_cell(status_index));
                    my_threat_sum +=
                        my_constraint_factor * my_meta_factor * cell_weight * my_threats;
                    opp_threat_sum +=
                        opp_constraint_factor * opp_meta_factor * cell_weight * opp_threats;
                }
                // calculate heuristic value
                let progress = (my_wins + opp_wins) / 9.0;
                let meta_weight = heuristic_config.meta_weight_base
                    + heuristic_config.meta_weight_progress_offset * progress;
                let threat_weight = 1.0 - meta_weight;
                let max_threat_score = 1.0_f32.max(my_threat_sum + opp_threat_sum);

                let final_score = 0.5
                    + meta_weight * (my_wins - opp_wins) / 9.0
                    + threat_weight * (my_threat_sum - opp_threat_sum) / max_threat_score;

                final_score.clamp(0.0, 1.0)
            }
        };

        let score = match player {
            TicTacToeStatus::Me => score,
            TicTacToeStatus::Opp => 1.0 - score,
            _ => unreachable!("Player is alway Me or Opp"),
        };
        heuristic_cache.insert_intermediate_score(state, score);
        score
    }

    fn evaluate_state_recursive(
        state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
        game_cache: &mut <UltTTTMCTSGame<GC> as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
        depth: usize,
        alpha: f32,
    ) -> f32 {
        let base_heuristic =
            Self::evaluate_state(state, game_cache, heuristic_cache, None, heuristic_config);

        if depth == 0 || UltTTTMCTSGame::evaluate(state, game_cache).is_some() {
            return base_heuristic;
        }

        let mut worst_response = f32::NEG_INFINITY;
        let next_player_alpha = alpha
            - (alpha - 0.5) * heuristic_config.evaluate_state_recursive_alpha_reduction_factor;
        // If no constraint on next move, this will be many moves to consider.
        // Therefore we use early exit to reduce calculation time.
        for next_player_move in UltTTTMCTSGame::<GC>::available_moves(state) {
            let next_player_state =
                UltTTTMCTSGame::apply_move(state, &next_player_move, game_cache);

            let response_value = Self::evaluate_state_recursive(
                &next_player_state,
                game_cache,
                heuristic_cache,
                heuristic_config,
                depth - 1,
                next_player_alpha,
            );

            if response_value > worst_response {
                worst_response = response_value;
                // early exit, because next player does have guaranteed win
                if worst_response >= heuristic_config.evaluate_state_recursive_early_exit_threshold
                {
                    break;
                }
            }
        }

        // combine base heuristic with worst case response
        alpha * base_heuristic + (1.0 - alpha) * (1.0 - worst_response)
    }

    fn evaluate_move(
        state: &<UltTTTMCTSGame<GC> as MCTSGame>::State,
        mv: &<UltTTTMCTSGame<GC> as MCTSGame>::Move,
        game_cache: &mut <UltTTTMCTSGame<GC> as MCTSGame>::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> f32 {
        let new_state = UltTTTMCTSGame::apply_move(state, mv, game_cache);
        UltTTTHeuristic::evaluate_state_recursive(
            &new_state,
            game_cache,
            heuristic_cache,
            heuristic_config,
            heuristic_config.evaluate_state_recursive_depth(),
            heuristic_config.evaluate_state_recursive_alpha(),
        )
    }
}

use std::cell::RefCell;
pub struct UltTTTHeuristicCache {
    pub cache: HashMap<UltTTT, f32>,
    pub usage: RefCell<usize>,
}

impl HeuristicCache<UltTTT, UltTTTMove> for UltTTTHeuristicCache {
    fn new() -> Self {
        UltTTTHeuristicCache {
            cache: HashMap::new(),
            usage: RefCell::new(0),
        }
    }
    fn get_intermediate_score(&self, state: &UltTTT) -> Option<f32> {
        let cached_score = self.cache.get(state).cloned();
        if cached_score.is_some() {
            *self.usage.borrow_mut() += 1;
        }
        cached_score
    }
    fn insert_intermediate_score(&mut self, state: &UltTTT, score: f32) {
        self.cache.insert(*state, score);
    }
}

#[cfg(test)]
mod tests {
    type ExpandAllTTT = ExpandAll<UltTTTMCTSGame<NoGameCache<UltTTT, UltTTTMove>>>;
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn test_new_mcts_traits_with_ult_ttt() {
        const TIME_OUT_FIRST_TURN: Duration = Duration::from_millis(995);
        const TIME_OUT_SUCCESSIVE_TURNS: Duration = Duration::from_millis(95);

        let mut wins = 0.0;
        let number_of_matches = 10;
        for i in 0..number_of_matches {
            eprintln!("________match {}________", i + 1);
            let mut first_mcts_ult_ttt: PlainMCTS<
                UltTTTMCTSGameNoGameCache,
                DynamicC,
                CachedUTC,
                HPWDefaultTTTNoGameCache,
                UltTTTHeuristic,
                HeuristicCutoff,
            > = PlainMCTS::new(
                UltTTTMCTSConfig::default(),
                UltTTTHeuristicConfig::default(),
            );
            let mut first_ult_ttt_game_data = UltTTT::new();
            first_ult_ttt_game_data.set_current_player(TicTacToeStatus::Me);
            let mut first_time_out = TIME_OUT_FIRST_TURN;
            let mut second_mcts_ult_ttt: PlainMCTS<
                UltTTTMCTSGame<NoGameCache<UltTTT, UltTTTMove>>,
                DynamicC,
                CachedUTC,
                ExpandAllTTT,
                NoHeuristic,
                DefaultSimulationPolicy,
            > = PlainMCTS::new(UltTTTMCTSConfig::default(), BaseHeuristicConfig::default());
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
            /*eprintln!(
                "game cache size / usage: {} / {}",
                first_mcts_ult_ttt.game_cache.cache.len(),
                first_mcts_ult_ttt.game_cache.usage
            );*/
            /*eprintln!(
                "heuristic cache size / usage: {} / {}",
                first_mcts_ult_ttt.heuristic_cache.cache.len(),
                first_mcts_ult_ttt.heuristic_cache.usage.borrow()
            );*/
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
