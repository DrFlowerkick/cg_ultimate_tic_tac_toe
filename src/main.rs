use std::cmp::Ordering;
use std::fmt::Write;
use std::io;
use std::time::Duration;

use my_lib::my_map_point::*;
use my_lib::my_map_two_dim::*;
use my_lib::my_monte_carlo_tree_search::*;
use my_lib::my_tic_tac_toe::*;

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
            // if Me is start_player, only choose cells from big middle cell
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
            // game finished without direct winner
            // count for each player number of won squares; most squares won wins game
            let my_squares = self.status_map.count_player_cells(MonteCarloPlayer::Me);
            let opp_squares = self.status_map.count_player_cells(MonteCarloPlayer::Opp);
            self.status = match my_squares.cmp(&opp_squares) {
                Ordering::Greater => TicTacToeStatus::Player(MonteCarloPlayer::Me),
                Ordering::Less => TicTacToeStatus::Player(MonteCarloPlayer::Opp),
                Ordering::Equal => TicTacToeStatus::Tie,
            };
        }
        // player_action.ult_ttt_small points to next TicTacToe for next player to set new value.
        // if this TicTacToe status is not vacant (meaning there are no more cells to set), player can choose from all free cells
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
        // game end or cell won/lost/tie
        self.execute_player_action(my_action, MonteCarloPlayer::Me)
            .is_not_vacant()
            || self
                .status_map
                .get_cell_value(my_action.ult_ttt_big)
                .is_player()
    }
    fn apply_opp_action(&mut self, player_action: &impl MonteCarloPlayerAction) -> bool {
        let opp_action = *UltTTTPlayerAction::downcast_self(player_action);
        // game end or cell won/lost/tie
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
        // no random game_data updates for TicTacToe
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
        //dummy
        true
    }
    fn check_consistency_of_game_data_update(
        &mut self,
        _current_game_state: &Self,
        _game_data_update: &impl MonteCarloGameDataUpdate,
        _played_turns: usize,
    ) -> bool {
        //dummy
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
        //dummy
        true
    }
}

macro_rules! parse_input {
    ($x:expr_2021, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
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
    // game loop
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
            // check starting_player
            if opponent_row >= 0 {
                starting_player = MonteCarloPlayer::Opp;
                turn_counter += 1;
                let opp_action =
                    MapPoint::<U, V>::new(opponent_col as usize, opponent_row as usize);
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
        let (_my_game_data, my_action) = mcts.choose_and_execute_actions();
        my_action.execute_action();

        turn_counter += 2;
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;

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
        let weighting_factor_player_one_me = 2.4;
        let weighting_factor_player_two_enemy = 1.4;
        let use_heuristic_score = false;
        let use_caching_player_one_me = true;
        let use_caching_player_two_enemy = true;
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
