// lib for ult_ttt game code

use my_lib::my_map_point::*;
use my_lib::my_map_two_dim::*;
use my_lib::my_monte_carlo_tree_search::*;
use my_lib::my_tic_tac_toe::*;

use std::cmp::Ordering;
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
    pub fn new(
        ult_ttt_data: &'a UltTTT,
        player: MonteCarloPlayer,
        parent_game_turn: usize,
    ) -> Self {
        let mut result = IterUltTTT {
            ult_ttt_data,
            player_action: UltTTTPlayerAction::default(),
            next_action_square_is_specified: false,
            iter_finished: false,
        };
        if parent_game_turn == 0 && player == MonteCarloPlayer::Me {
            // if Me is start_player, only choose cells from big middle cell
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

#[derive(Copy, Clone, PartialEq, Default)]
pub struct UltTTTGameDataUpdate {}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct UltTTT {
    map: MyMap2D<TicTacToeGameData, X, Y>,
    pub status_map: TicTacToeGameData,
    pub status: TicTacToeStatus,
    next_action_square_is_specified: Option<MapPoint<X, Y>>,
}

impl UltTTT {
    pub fn new() -> Self {
        UltTTT {
            map: MyMap2D::new(),
            status_map: TicTacToeGameData::new(),
            status: TicTacToeStatus::Vacant,
            next_action_square_is_specified: None,
        }
    }
    pub fn set_last_opp_action(&mut self, opp_map_point: MapPoint<U, V>) -> TicTacToeStatus {
        let opp_action = UltTTTPlayerAction::from_ext(opp_map_point);
        self.execute_player_action(opp_action, MonteCarloPlayer::Opp)
    }
    pub fn execute_player_action(
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
            // game finished without direct winner
            // count for each player number of won squares; most squares won wins game
            let my_squares = self.status_map.count_player_cells(MonteCarloPlayer::Me);
            let opp_squares = self.status_map.count_player_cells(MonteCarloPlayer::Opp);
            self.status = match my_squares.cmp(&opp_squares) {
                Ordering::Greater => TicTacToeStatus::Player(MonteCarloPlayer::Me),
                Ordering::Less => TicTacToeStatus::Player(MonteCarloPlayer::Opp),
                Ordering::Equal => TicTacToeStatus::Tie,
            };
        } else {
            // player_action.ult_ttt_small points to next TicTacToe for next player to set new value.
            // if this TicTacToe status is not vacant (meaning there are no more cells to set), player can choose from all free cells
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

#[cfg(test)]
mod tests {
    use std::fmt::Display;
    use std::time::Duration;

    use super::*;

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

    impl UltTTT {
        fn get_cell_value(&self, cell: UltTTTPlayerAction) -> TicTacToeStatus {
            self.map
                .get(cell.ult_ttt_big)
                .get_cell_value(cell.ult_ttt_small)
        }
    }

    // ToDo: move this to mcts! Requires Display trait for MonteCarloAction
    fn print_distribution_of_actions(
        mcts: &MonteCarloTreeSearch<UltTTT, UltTTTPlayerAction, UltTTTGameDataUpdate>,
    ) {
        // This function MUST be run after expand_tree() and before choose_and_execute_actions(),
        // if you want to get distribution of possible actions for current game turn.
        let (total_num_of_nodes, children_data) = mcts.node_data_of_root_children();
        eprintln!("Total number of nodes in tree: {}", total_num_of_nodes);
        for (possible_action, num_children_nodes, wins, samples) in
            children_data.iter().filter(|(_, n, ..)| *n > 0)
        {
            eprintln!(
                "Action: {}, percentage of nodes: {:.2}%, win rate: {:.2}%",
                possible_action.to_ext(),
                ((*num_children_nodes as f32) / (total_num_of_nodes as f32)) * 100.0,
                100.0 * wins / samples
            );
        }
    }

    #[test]
    fn one_match() {
        let starting_player = MonteCarloPlayer::Me;
        let mut game_data_me = UltTTT::new();
        let mut game_data_enemy = UltTTT::new();
        let game_mode = MonteCarloGameMode::ByTurns;
        let max_number_of_turns = 81;
        let force_update = true;
        let time_out_first_turn = Duration::from_millis(995);
        let time_out_successive_turns = Duration::from_millis(95);
        let weighting_factor_player_one_me = 1.4;
        let weighting_factor_player_two_enemy = 1.4;
        let use_heuristic_score = false;
        let use_caching_player_one_me = false;
        let use_caching_player_two_enemy = false;
        let debug = true;
        let mut player_one_me: MonteCarloTreeSearch<
            UltTTT,
            UltTTTPlayerAction,
            UltTTTGameDataUpdate,
        > = MonteCarloTreeSearch::new(
            game_mode,
            max_number_of_turns,
            force_update,
            time_out_first_turn,
            time_out_successive_turns,
            weighting_factor_player_one_me,
            use_heuristic_score,
            use_caching_player_one_me,
            debug,
        );
        let mut player_two_enemy: MonteCarloTreeSearch<
            UltTTT,
            UltTTTPlayerAction,
            UltTTTGameDataUpdate,
        > = MonteCarloTreeSearch::new(
            game_mode,
            max_number_of_turns,
            force_update,
            time_out_first_turn,
            time_out_successive_turns,
            weighting_factor_player_two_enemy,
            use_heuristic_score,
            use_caching_player_two_enemy,
            debug,
        );
        let mut active_player = MonteCarloPlayer::Me;
        let mut game_turn = 0;
        // game loop
        while game_data_me.status.is_vacant() {
            // dbg output of first 10 turns
            if game_turn == 10 {
                break;
            }
            // print current game state
            eprintln!("{}", game_data_me);
            game_turn += 1;
            let (active_player_game_data, active_player_mcts, sp, opp_game_data) =
                match active_player {
                    MonteCarloPlayer::Me => {
                        eprint!("{:02} me:  ", game_turn);
                        (
                            &mut game_data_me,
                            &mut player_one_me,
                            starting_player,
                            &mut game_data_enemy,
                        )
                    }
                    MonteCarloPlayer::Opp => {
                        eprint!("{:02} opp: ", game_turn);
                        (
                            &mut game_data_enemy,
                            &mut player_two_enemy,
                            starting_player.next_player(),
                            &mut game_data_me,
                        )
                    }
                };
            let start = active_player_mcts.init_root(active_player_game_data, sp);
            active_player_mcts.expand_tree(start);
            print_distribution_of_actions(active_player_mcts);
            let (new_game_data, chosen_action) = active_player_mcts.choose_and_execute_actions();
            eprintln!("Chosen action: {}", chosen_action.to_ext());
            *active_player_game_data = new_game_data;
            // apply active player action to opponent player game data
            opp_game_data.set_last_opp_action(chosen_action.to_ext());
            // switch active_player
            active_player = active_player.next_player();
        }
        // print result of match
        eprintln!("{}", game_data_me);
        match game_data_me.status {
            TicTacToeStatus::Vacant => println!("Something went wrong here..."),
            TicTacToeStatus::Player(winner) => match winner {
                MonteCarloPlayer::Me => println!("Player One won"),
                MonteCarloPlayer::Opp => println!("Player Two won"),
            },
            TicTacToeStatus::Tie => println!("It's a tie"),
        }
    }
}
