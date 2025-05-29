// game cache for UltTTT

use super::*;
use std::collections::{HashMap, HashSet};

pub struct UltTTTGameCache {
    pub cache: HashMap<TicTacToeGameData, BoardAnalysis>,
    pub usage: usize,
}

impl UltTTTGameCache {
    pub fn cache_board_analysis(&mut self, board: &TicTacToeGameData) -> BoardAnalysis {
        if let Some(cached_analysis) = self.cache.get(board) {
            self.usage += 1;
            return cached_analysis.clone();
        }
        let board_analysis = board.board_analysis();
        self.cache.insert(*board, board_analysis.clone());
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

impl UltTTTGameCacheTrait for UltTTTGameCache {
    fn get_status(&mut self, board: &TicTacToeGameData) -> TicTacToeStatus {
        self.cache_board_analysis(board).status
    }
    fn get_board_progress(&mut self, board: &TicTacToeGameData) -> (usize, usize, usize) {
        let board_analysis = self.cache_board_analysis(board);
        (
            board_analysis.my_cells,
            board_analysis.opp_cells,
            board_analysis.played_cells,
        )
    }
    fn get_board_control(&mut self, board: &TicTacToeGameData) -> (f32, f32) {
        let board_analysis = self.cache_board_analysis(board);
        (board_analysis.my_control, board_analysis.opp_control)
    }
    fn get_board_threats(
        &mut self,
        board: &TicTacToeGameData,
    ) -> (HashSet<CellIndex3x3>, HashSet<CellIndex3x3>) {
        let board_analysis = self.cache_board_analysis(board);
        (board_analysis.my_threats, board_analysis.opp_threats)
    }
    fn get_meta_cell_threats(
        &mut self,
        board: &TicTacToeGameData,
        index: CellIndex3x3,
    ) -> (u8, u8, u8, u8) {
        let board_analysis = self.cache_board_analysis(board);
        *board_analysis.meta_cell_threats.get_cell(index)
    }
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
