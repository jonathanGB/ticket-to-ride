use crate::{
    card::CardDealer,
    map::Map,
    player::{Player, PlayerColor},
};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::HashSet;
use strum::IntoEnumIterator;

const MAX_PLAYERS: usize = 5;

#[derive(Serialize, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GamePhase {
    InLobby,
    Starting,
    Playing,
    LastTurn,
    Done,
}

pub struct Manager {
    phase: GamePhase,
    turn: Option<usize>,
    map: Option<Map>,
    card_dealer: Option<CardDealer>,
    players: SmallVec<[Player; MAX_PLAYERS]>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::InLobby,
            turn: None,
            map: None,
            card_dealer: None,
            players: SmallVec::new(),
        }
    }

    pub fn get_state(&self, player_id: usize) {
        unimplemented!()
    }

    pub fn num_players(&self) -> usize {
        self.players.len()
    }

    pub fn add_player(&mut self) -> Option<usize> {
        if self.num_players() == MAX_PLAYERS {
            return None;
        }

        let player_id = self.num_players();

        self.players.push(Player::new(
            player_id,
            self.generate_default_player_color(),
            self.generate_default_player_name(player_id),
        ));

        Some(player_id)
    }

    fn generate_default_player_color(&self) -> PlayerColor {
        let used_player_colors: HashSet<PlayerColor> =
            self.players.iter().map(|player| player.color()).collect();

        for player_color in PlayerColor::iter() {
            if !used_player_colors.contains(&player_color) {
                return player_color;
            }
        }

        unreachable!("There should always be at least one player color left to pick.")
    }

    fn generate_default_player_name(&self, player_id: usize) -> String {
        let used_player_names: HashSet<&str> =
            self.players.iter().map(|player| player.name()).collect();

        for i in 1..=5 {
            let player_name = format!("Player {:01$}", player_id, i);
            if !used_player_names.contains(&*player_name) {
                return player_name;
            }
        }

        unreachable!(
            "To add a player, there must be at most four other players.
             Thus, we should be able to generate a random name in at most 5 tries."
        )
    }

    // TODO: test this.
    pub fn change_player_name(&mut self, player_id: usize, new_name: String) -> Result<(), String> {
        for player in &self.players {
            if player.name() == new_name {
                return Err(format!(
                    "Cannot change name to already existing `{}`",
                    new_name
                ));
            }
        }

        self.players[player_id].change_name(new_name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_phase_to_json() -> serde_json::Result<()> {
        assert_eq!(serde_json::to_string(&GamePhase::InLobby)?, r#""in_lobby""#);
        assert_eq!(serde_json::to_string(&GamePhase::Playing)?, r#""playing""#);
        Ok(())
    }

    #[test]
    fn json_to_game_phase() -> serde_json::Result<()> {
        assert_eq!(
            serde_json::from_str::<GamePhase>(r#""last_turn""#)?,
            GamePhase::LastTurn
        );
        assert_eq!(
            serde_json::from_str::<GamePhase>(r#""done""#)?,
            GamePhase::Done
        );

        Ok(())
    }

    #[test]
    fn invalid_json_to_game_phase() {
        assert!(serde_json::from_str::<GamePhase>(r#""not_started""#).is_err());
    }

    // Tests for `Manager::add_player`.

    #[test]
    fn manager_add_player_unique() {
        let mut m = Manager::new();

        assert_eq!(m.add_player(), Some(0));
        assert_eq!(m.add_player(), Some(1));
        assert_eq!(m.add_player(), Some(2));
        assert_eq!(m.add_player(), Some(3));
        assert_eq!(m.add_player(), Some(4));
        assert!(m.add_player().is_none());
        assert_eq!(m.num_players(), 5);

        for (i, player) in m.players.iter().enumerate() {
            for (j, other_player) in m.players.iter().enumerate() {
                if i == j {
                    continue;
                }

                assert_ne!(player.color(), other_player.color());
                assert_ne!(player.name(), other_player.name());
                assert_ne!(player.id(), other_player.id());
            }
        }
    }

    #[test]
    fn manager_add_player_name_collision() {
        let mut m = Manager::new();

        for i in 1..=4 {
            assert_eq!(m.add_player(), Some(i - 1));
            m.players[i - 1].change_name(format!("Player {:01$}", 4, i));
        }

        assert_eq!(m.add_player(), Some(4));
        assert_eq!(m.num_players(), 5);
        assert_eq!(m.players[4].name(), "Player 00004");
    }
}
