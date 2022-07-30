use crate::{
    card::{CardDealer, CardDealerState, NUM_DRAWN_DESTINATION_CARDS},
    map::Map,
    player::{Player, PlayerColor, PlayerState},
};

use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Serialize;
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;

const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 5;

#[derive(Clone, Copy, Serialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GamePhase {
    InLobby,
    Starting,
    Playing,
    LastTurn,
    Done,
}

#[derive(Serialize)]
pub struct GameState<'a> {
    phase: GamePhase,
    turn: Option<usize>,
    card_dealer_state: Option<CardDealerState<'a>>,
    players_state: SmallVec<[PlayerState<'a>; MAX_PLAYERS]>,
}

type ManagerActionResult = Result<(), String>;

/// In charge of holding all the state of the game, managing player actions, and transitions amongst players.
///
/// This overall acts as a finite-state machine.
pub struct Manager {
    /// The current phase of the game, which marks nodes (states) in this finite-state machine.
    phase: GamePhase,
    /// Keeps track of the current turn, which is incremented every time
    /// a player finishes their turn.
    ///
    /// This is `None` as long as we are either in [`GamePhase::InLobby`],
    /// or in [`GamePhase::Starting`].
    turn: Option<usize>,
    /// Holds the [`Map`].
    /// Only populated once the game is started!
    map: Option<Map>,
    /// Holds the [`CardDealer`].
    /// Only populated once the game is started!
    card_dealer: Option<CardDealer>,
    /// List of all players.
    ///
    /// In the lobby, the player ID simply matches their index in this array.
    /// However, once we start the game, we shuffle this list, such that the
    /// order in which players play is random. To help keep track of specific
    /// players after shuffling, we map their positions in `players_position`.
    players: SmallVec<[Player; MAX_PLAYERS]>,
    /// Maps a player ID to their position in the `players` array.
    /// Only populated once the game is started!
    players_position: HashMap<usize, usize>,
    /// Only relevant in the [`GamePhase::Starting`].
    ///
    /// Keeps track of the number of players that have selected their initial set of
    /// destination cards.
    ///
    /// Once that number equals the number of players, we are ready to start the turn-based
    /// game -- and transition to the [`GamePhase::Playing`].
    num_players_selected_initial_destination_cards: usize,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::InLobby,
            turn: None,
            map: None,
            card_dealer: None,
            players: SmallVec::new(),
            players_position: HashMap::new(),
            num_players_selected_initial_destination_cards: 0,
        }
    }

    pub fn get_state(&self, player_id: usize) -> GameState {
        GameState {
            phase: self.phase,
            turn: self.turn,
            card_dealer_state: self
                .card_dealer
                .as_ref()
                .map(|card_dealer| card_dealer.get_state()),
            players_state: self
                .players
                .iter()
                .map(|player| player.get_player_state(player_id))
                .collect(),
        }
    }

    pub fn num_players(&self) -> usize {
        self.players.len()
    }

    #[inline]
    fn get_player_index(&self, player_id: usize) -> Option<usize> {
        self.players_position
            .get(&player_id)
            .map(|player_id| *player_id)
    }

    pub fn add_player(&mut self) -> Option<usize> {
        if self.phase != GamePhase::InLobby || self.num_players() == MAX_PLAYERS {
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
    pub fn change_player_name(
        &mut self,
        player_id: usize,
        new_name: String,
    ) -> ManagerActionResult {
        if self.phase != GamePhase::InLobby {
            return Err(String::from(
                "Cannot change player's name outside of the lobby phase.",
            ));
        }

        for player in &self.players {
            if player.name() == new_name {
                return Err(format!(
                    "Cannot change name to already existing `{}`.",
                    new_name
                ));
            }
        }

        self.players[player_id].change_name(new_name);
        Ok(())
    }

    // TODO: test this.
    pub fn change_player_color(
        &mut self,
        player_id: usize,
        new_color: PlayerColor,
    ) -> ManagerActionResult {
        if self.phase != GamePhase::InLobby {
            return Err(String::from(
                "Cannot change player's color outside of the lobby phase.",
            ));
        }

        for player in &self.players {
            if player.color() == new_color {
                return Err(format!(
                    "Cannot change color to `{}`, as it is already used.",
                    new_color
                ));
            }
        }

        self.players[player_id].change_color(new_color);
        Ok(())
    }

    // TODO: test this.
    pub fn set_ready(&mut self, player_id: usize, is_ready: bool) -> ManagerActionResult {
        if self.phase != GamePhase::InLobby {
            return Err(String::from(
                "Cannot change ready status outside of the lobby phase.",
            ));
        }

        self.players[player_id].set_ready(is_ready);

        if self.num_players() >= MIN_PLAYERS && self.players.iter().all(|player| player.ready()) {
            self.start_game()?;
        }

        Ok(())
    }

    fn start_game(&mut self) -> ManagerActionResult {
        let map = Map::new(self.num_players())?;
        let mut card_dealer = CardDealer::new();

        self.phase = GamePhase::Starting;
        self.players.shuffle(&mut thread_rng());

        for (index, player) in self.players.iter_mut().enumerate() {
            self.players_position.insert(player.id(), index);
            player.initialize_when_game_starts(&mut card_dealer);
        }

        self.map = Some(map);
        self.card_dealer = Some(card_dealer);
        Ok(())
    }

    // TODO: test this.
    pub fn select_destination_cards(
        &mut self,
        player_id: usize,
        destination_cards_decisions: SmallVec<[bool; NUM_DRAWN_DESTINATION_CARDS]>,
    ) -> ManagerActionResult {
        if self.phase != GamePhase::Starting && self.phase != GamePhase::Playing {
            return Err(String::from(
                "Cannot select destination cards outside of the starting or playing phases.",
            ));
        }

        let player_index = self.get_player_index(player_id).unwrap();
        self.players[player_index].select_destination_cards(
            destination_cards_decisions,
            self.turn,
            self.card_dealer.as_mut().unwrap(),
        )?;

        if self.phase == GamePhase::Starting {
            self.num_players_selected_initial_destination_cards += 1;

            if self.num_players_selected_initial_destination_cards == self.num_players() {
                self.phase = GamePhase::Playing;
                self.turn = Some(0);
            }
        }

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

    // Tests for `Manager::add_player`.

    #[test]
    fn manager_add_player_outside_of_in_lobby_phase() {
        let mut m = Manager::new();

        m.phase = GamePhase::Starting;
        assert!(m.add_player().is_none());

        m.phase = GamePhase::Playing;
        assert!(m.add_player().is_none());

        m.phase = GamePhase::InLobby;
        assert!(m.add_player().is_some());
    }

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
