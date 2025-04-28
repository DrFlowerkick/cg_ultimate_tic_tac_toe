// implementation of the first generation monte carlo tree search traits

use my_lib::my_monte_carlo_tree_search::*;
use my_lib::my_tic_tac_toe::*;

use super::ult_ttt::*;

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
