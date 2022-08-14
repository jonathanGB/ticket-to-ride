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
/// Phases of the games, which act as states in the game's finite-state machine.
///
/// # JSON
/// Phases are serialized in snake_case.
pub enum GamePhase {
    /// Initial phase of the game: when players are joining the lobby, before the game has started.
    ///
    /// Players can join the game, change their name and color, and mark themselves as ready.
    ///
    /// Once all players are ready, we move to the next phase.
    InLobby,
    /// When the game starts, players concurrently select their initial set of
    /// [`crate::card::DestinationCard`]s.
    ///
    /// Once all have done so, we move to the turn-based game ([`GamePhase::Playing`]).
    Starting,
    /// The main phase of the game.
    ///
    /// Denotes the main turn-based game, up until when we transition to each player's last turn.
    Playing,
    /// When a player is left with less than three trains, every player has one turn left.
    ///
    /// This last turn is denoted by this special phase.
    LastTurn,
    /// When each player has played their last turn.
    ///
    /// No actions can be taken at this point.
    Done,
}

#[derive(Serialize)]
/// All the information about a game's current state, returned by [`Manager::get_state`].
pub struct GameState<'a> {
    /// The phase of the game.
    pub phase: GamePhase,
    /// Which player's turn it is, which maps to an index in [`GameState::players_state`] (modulo the number of players).
    ///
    /// Initially, this is `None`. This denotes the initial draw that happens concurrently for all players,
    /// before turns have started.
    ///
    /// When we start the turn-based game, then `turn` is set to 0, and increments after each turn.
    pub turn: Option<usize>,
    /// Public information about the decks of train cards and destination cards.
    ///
    /// Until the game has started, this is `None`.
    pub card_dealer_state: Option<CardDealerState<'a>>,
    /// Information about all the players in the game.
    ///
    /// This only contains public information about them, except for requests coming from player _A_,
    /// which also holds private information about _A_ (and only _A_).
    pub players_state: SmallVec<[PlayerState<'a>; MAX_PLAYERS]>,
}

/// All actions taken by a manager have the same `Result`:
///
/// * Either it succeeded, which we mark with an empty tuple.
/// * Or it failed, which includes a human-readable error message.
pub type ManagerActionResult = Result<(), String>;

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
    /// Only relevant in the [`GamePhase::LastTurn`].
    ///
    /// Keeps track of the number of players that have finished playing.
    ///
    /// Once that number equals the number of players, the game is over -- and transition
    /// to the [`GamePhase::Done`].
    num_players_done_playing: usize,
}

impl Manager {
    /// Creates a new [`Manager`] in the [`GamePhase::InLobby`].
    pub fn new() -> Self {
        Self {
            phase: GamePhase::InLobby,
            turn: None,
            map: None,
            card_dealer: None,
            players: SmallVec::new(),
            players_position: HashMap::new(),
            num_players_selected_initial_destination_cards: 0,
            num_players_done_playing: 0,
        }
    }

    /// Returns the game's state, from the perspective of a given player.
    ///
    /// This said perspective is important, because a given player should only be
    /// able to know about the public information of other players, but should know
    /// private information about themselves (e.g. which train cards they have).
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

    /// Returns the number of players in the current game.
    pub fn num_players(&self) -> usize {
        self.players.len()
    }

    #[inline]
    fn get_player_index(&self, player_id: usize) -> Option<usize> {
        self.players_position
            .get(&player_id)
            .map(|player_id| *player_id)
    }

    /// Creates a new [`Player`] (with a unique name and color),
    /// and adds it to the list of players for the current game.
    ///
    /// Returns `None` if we are not in [`GamePhase::InLobby`], or if we have reached the maximum
    /// of allowed players.
    ///
    /// Otherwise, returns the ID of the new player.
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

        for id_length in 1..=MAX_PLAYERS {
            let player_name = format!("Player {:01$}", player_id, id_length);
            if !used_player_names.contains(&*player_name) {
                return player_name;
            }
        }

        unreachable!(
            "To add a player, there must be at most four other players.
             Thus, we should be able to generate a random name in at most 5 tries."
        )
    }

    /// Changes the given player's name.
    ///
    /// Returns an `Err` if either:
    ///   * We are not in [`GamePhase::InLobby`].
    ///   * A player already has the same name.
    ///
    /// Otherwise, returns `Ok(())`.
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

    /// Changes the given player's color.
    ///
    /// Returns an `Err` if either:
    ///   * We are not in [`GamePhase::InLobby`].
    ///   * A player already has the same color.
    ///
    /// Otherwise, returns `Ok(())`.
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

    /// Changes the given player's _ready_ status (to `true` or `false`).
    ///
    /// Returns an `Err` if we are not in [`GamePhase::InLobby`].
    ///
    /// Otherwise, returns `Ok(())`.
    ///
    /// If and only if all players are ready, then we start the game, which entails:
    ///   * Creating a [`Map`] and a [`CardDealer`].
    ///   * Transitioning to [`GamePhase::Starting`].
    ///   * Drawing the initial set of train and destination cards for each player.
    ///   * Shuffling the order of players.
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

    #[inline]
    fn has_game_started(&self) -> ManagerActionResult {
        if self.phase == GamePhase::Starting || self.has_turn_based_game_started().is_ok() {
            Ok(())
        } else {
            Err(String::from(
                "Cannot play if the game has not started, or if it has ended.",
            ))
        }
    }

    #[inline]
    fn has_turn_based_game_started(&self) -> ManagerActionResult {
        if self.phase == GamePhase::Playing || self.phase == GamePhase::LastTurn {
            Ok(())
        } else {
            Err(String::from(
                "Cannot play if the turn-based game has not started, or if it has ended.",
            ))
        }
    }

    /// # Panic!
    /// This should only be called once the turn-based game has started,
    /// as it assumes that `self.turn` is defined, and that the number of players is >0.
    #[inline]
    fn is_player_turn(&self, player_index: usize) -> ManagerActionResult {
        if self.turn.unwrap() % self.num_players() == player_index {
            Ok(())
        } else {
            Err(String::from("This is not your turn!"))
        }
    }

    /// # Panic!
    /// This should only be called once the turn-based game has started,
    /// as it assumes that `self.turn` is defined.
    #[inline]
    fn increment_turn(&mut self) {
        *self.turn.as_mut().unwrap() += 1;
    }

    /// Updates players' and game's state, depending on whether a given player is done playing.
    ///
    /// If we are in [`GamePhase::LastTurn`], the player is marked as done.
    ///
    /// Furthermore, if all players are done, then we transition to [`GamePhase::Done`].
    /// When we do so, we update the points of each player, based on whether they have fulfilled
    /// or not their destination cards. Finally, we compute the game's longest route, and grant
    /// points to those having built said longest route.
    fn maybe_player_and_game_done(&mut self, player_index: usize) {
        if self.phase != GamePhase::LastTurn {
            return;
        }
        self.num_players_done_playing += 1;
        self.players[player_index].set_done_playing();

        if self.num_players_done_playing != self.num_players() {
            return;
        }

        let map = self.map.as_mut().unwrap();

        self.phase = GamePhase::Done;
        let all_longest_routes: SmallVec<[u16; MAX_PLAYERS]> = self
            .players
            .iter_mut()
            .map(|player| player.finalize_game(map))
            .collect();
        let max_longest_route = *all_longest_routes.iter().max().unwrap();

        all_longest_routes
            .into_iter()
            .enumerate()
            .for_each(|(player_index, longest_route)| {
                self.players[player_index]
                    .set_has_longest_route(longest_route == max_longest_route);
            });
    }

    /// Allows a given player to select from the set of destination cards --
    /// which they will try to fulfill.
    ///
    /// Returns an `Err` if either:
    ///   * We are not in [`GamePhase::Starting`], [`GamePhase::Playing`], nor [`GamePhase::LastTurn`].
    ///   * This is not the player's turn, once the turn-based game has started.
    ///   * [`Player::select_destination_cards`] failed.
    ///
    /// Otherwise, returns `Ok(())`, and increments the turn (if the turn-based game has started).
    /// As all actions that mark the end of the turn, we subsequently verify whether the
    /// player is done playing. More details in `Manager::maybe_player_and_game_done`.
    ///
    /// If this selection happens during [`GamePhase::Starting`], we check whether all players have selected
    /// their destination cards. If that is the case, then we:
    ///   * Transition to [`GamePhase::Playing`].
    ///   * Set the turn to 0.
    pub fn select_destination_cards(
        &mut self,
        player_id: usize,
        destination_cards_decisions: SmallVec<[bool; NUM_DRAWN_DESTINATION_CARDS]>,
    ) -> ManagerActionResult {
        self.has_game_started()?;

        let player_index = self.get_player_index(player_id).unwrap();
        if self.has_turn_based_game_started().is_ok() {
            self.is_player_turn(player_index)?;
        }

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
        } else {
            self.increment_turn();
            self.maybe_player_and_game_done(player_index);
        }

        Ok(())
    }

    /// Allows a given player to draw destination cards.
    ///
    /// Returns an `Err` if either:
    ///   * We are not in [`GamePhase::Playing`], nor [`GamePhase::LastTurn`].
    ///   * This is not the player's turn.
    ///   * [`Player::draw_destination_cards`] failed.
    ///
    /// Otherwise, returns `Ok(())`.
    pub fn draw_destination_cards(&mut self, player_id: usize) -> ManagerActionResult {
        self.has_turn_based_game_started()?;

        let player_index = self.get_player_index(player_id).unwrap();
        self.is_player_turn(player_index)?;

        self.players[player_index]
            .draw_destination_cards(self.turn.unwrap(), self.card_dealer.as_mut().unwrap())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{city::City, map::ClaimedRoute};

    // Tests for `GamePhase`.

    #[test]
    fn game_phase_to_json() -> serde_json::Result<()> {
        assert_eq!(serde_json::to_string(&GamePhase::InLobby)?, r#""in_lobby""#);
        assert_eq!(
            serde_json::to_string(&GamePhase::Starting)?,
            r#""starting""#
        );
        assert_eq!(serde_json::to_string(&GamePhase::Playing)?, r#""playing""#);
        assert_eq!(
            serde_json::to_string(&GamePhase::LastTurn)?,
            r#""last_turn""#
        );
        assert_eq!(serde_json::to_string(&GamePhase::Done)?, r#""done""#);

        Ok(())
    }

    // Tests for `Manager`.

    #[test]
    fn manager_new() {
        let m = Manager::new();

        assert_eq!(m.phase, GamePhase::InLobby);
        assert!(m.turn.is_none());
        assert!(m.map.is_none());
        assert!(m.card_dealer.is_none());
        assert!(m.players.is_empty());
        assert!(m.players_position.is_empty());
        assert_eq!(m.num_players_selected_initial_destination_cards, 0);
    }

    #[test]
    fn manager_add_player_outside_of_in_lobby_phase() {
        let mut m = Manager::new();

        m.phase = GamePhase::Starting;
        assert!(m.add_player().is_none());

        m.phase = GamePhase::Playing;
        assert!(m.add_player().is_none());

        assert!(m.players.is_empty());

        m.phase = GamePhase::InLobby;
        let player_id = m.add_player();
        assert!(player_id.is_some());
        let player_id = player_id.unwrap();

        let game_state = m.get_state(player_id);
        assert_eq!(game_state.phase, GamePhase::InLobby);
        assert!(game_state.turn.is_none());
        assert!(game_state.card_dealer_state.is_none());
        assert_eq!(game_state.players_state.len(), 1);
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

        let game_state = m.get_state(0);
        assert_eq!(game_state.players_state.len(), MAX_PLAYERS);
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

    #[test]
    fn manager_change_player_name() {
        let mut m = Manager::new();

        let player_id = m.add_player().unwrap();
        let other_player_id = m.add_player().unwrap();

        let new_name = String::from("Bob");
        assert!(m.change_player_name(player_id, new_name.clone()).is_ok());
        assert_eq!(m.players[0].name(), new_name.clone());

        assert!(m.change_player_name(player_id, new_name.clone()).is_err());
        assert!(m
            .change_player_name(other_player_id, new_name.clone())
            .is_err());
        assert_ne!(m.players[1].name(), new_name);
    }

    #[test]
    fn manager_change_player_name_wrong_phase() {
        let mut m = Manager::new();

        let player_id = m.add_player().unwrap();
        let other_player_id = m.add_player().unwrap();

        m.phase = GamePhase::Playing;

        let new_name = String::from("Bob");
        assert!(m.change_player_name(player_id, new_name.clone()).is_err());
        assert!(m.change_player_name(other_player_id, new_name).is_err());
    }

    #[test]
    fn manager_change_player_color() {
        let mut m = Manager::new();

        let player_id = m.add_player().unwrap();
        let other_player_id = m.add_player().unwrap();

        let new_color = PlayerColor::Yellow;
        assert!(m.change_player_color(player_id, new_color).is_ok());
        assert_eq!(m.players[0].color(), new_color);

        assert!(m.change_player_color(player_id, new_color).is_err());
        assert!(m.change_player_color(other_player_id, new_color).is_err());
        assert_ne!(m.players[1].color(), new_color);
    }

    #[test]
    fn manager_change_player_color_wrong_phase() {
        let mut m = Manager::new();

        let player_id = m.add_player().unwrap();
        let other_player_id = m.add_player().unwrap();

        m.phase = GamePhase::Playing;

        let new_color = PlayerColor::Yellow;
        assert!(m.change_player_color(player_id, new_color).is_err());
        assert!(m.change_player_color(other_player_id, new_color).is_err());
    }

    #[test]
    fn manager_set_ready() {
        let mut m = Manager::new();

        let player_id = m.add_player().unwrap();
        let other_player_id = m.add_player().unwrap();

        assert!(m.set_ready(player_id, true).is_ok());
        assert_eq!(m.phase, GamePhase::InLobby);
        assert!(m.players[0].ready());

        assert!(m.set_ready(player_id, false).is_ok());
        assert_eq!(m.phase, GamePhase::InLobby);
        assert_eq!(m.players[0].ready(), false);

        assert!(m.set_ready(other_player_id, true).is_ok());
        assert_eq!(m.phase, GamePhase::InLobby);
        assert!(m.players[1].ready());

        assert!(m.set_ready(player_id, true).is_ok());
        assert_eq!(m.phase, GamePhase::Starting);
        assert!(m.players[0].ready());

        assert!(m.turn.is_none());
        assert!(m.map.is_some());
        assert!(m.card_dealer.is_some());

        let game_state = m.get_state(player_id);
        assert_eq!(game_state.phase, GamePhase::Starting);
        assert!(game_state.turn.is_none());
        assert!(game_state.card_dealer_state.is_some());
        assert_eq!(game_state.players_state.len(), 2);
        assert_ne!(game_state.players_state[0], game_state.players_state[1]);
    }

    #[test]
    fn manager_set_ready_wrong_phase() {
        let mut m = Manager::new();

        let player_id = m.add_player().unwrap();
        let other_player_id = m.add_player().unwrap();

        m.phase = GamePhase::Playing;

        let is_ready = false;
        assert!(m.set_ready(player_id, is_ready).is_err());
        assert!(m.set_ready(other_player_id, is_ready).is_err());
    }

    #[test]
    fn manager_select_destination_cards() {
        let mut m = Manager::new();

        let player_id = m.add_player().unwrap();
        let other_player_id = m.add_player().unwrap();

        let destination_cards_decisions = smallvec![true, false, true];
        assert!(m
            .select_destination_cards(player_id, destination_cards_decisions.clone())
            .is_err());

        assert!(m.set_ready(player_id, true).is_ok());
        assert!(m.set_ready(other_player_id, true).is_ok());

        for player in &m.players {
            assert_eq!(
                player.get_private_state().pending_destination_cards.len(),
                NUM_DRAWN_DESTINATION_CARDS
            );
            assert!(player
                .get_private_state()
                .selected_destination_cards
                .is_empty());
        }

        // Invalid selection, because at least two cards must be selected when we start the game.
        let invalid_cards_decisions = smallvec![true, false, false];
        assert!(m
            .select_destination_cards(player_id, invalid_cards_decisions)
            .is_err());

        assert!(m
            .select_destination_cards(player_id, destination_cards_decisions.clone())
            .is_ok());
        assert_eq!(m.phase, GamePhase::Starting);
        assert!(m.turn.is_none());

        let player_index = m.get_player_index(player_id).unwrap();
        assert!(m.players[player_index]
            .get_private_state()
            .pending_destination_cards
            .is_empty());
        // Only 2 cards were previously selected.
        assert_eq!(
            m.players[player_index]
                .get_private_state()
                .selected_destination_cards
                .len(),
            2
        );

        // Same player can't select cards again in the same turn.
        assert!(m
            .select_destination_cards(player_id, destination_cards_decisions.clone())
            .is_err());

        assert!(m
            .select_destination_cards(other_player_id, destination_cards_decisions)
            .is_ok());
        assert_eq!(m.phase, GamePhase::Playing);
        assert_eq!(m.turn, Some(0));

        let (player_id_first, player_id_second) = if m.get_player_index(player_id) == Some(0) {
            (player_id, other_player_id)
        } else {
            (other_player_id, player_id)
        };

        assert!(m.draw_destination_cards(player_id_first).is_ok());

        assert_eq!(m.turn, Some(0));

        let destination_cards_decisions = smallvec![true, false, false];
        let invalid_destination_cards_decisions = smallvec![false, false, false];
        // Wrong turn.
        assert!(m
            .select_destination_cards(player_id_second, destination_cards_decisions.clone())
            .is_err());
        // Invalid selection, because at least one card during the turn-based game.
        assert!(m
            .select_destination_cards(player_id_first, invalid_destination_cards_decisions.clone())
            .is_err());

        assert!(m
            .select_destination_cards(player_id_first, destination_cards_decisions.clone())
            .is_ok());

        assert_eq!(m.turn, Some(1));
    }

    #[test]
    fn manager_select_destination_cards_game_done() {
        let mut m = Manager::new();

        let player_id = m.add_player().unwrap();
        m.add_player().unwrap();

        m.phase = GamePhase::Done;

        let destination_cards_decisions = smallvec![true, false, true];
        assert!(m
            .select_destination_cards(player_id, destination_cards_decisions)
            .is_err());
    }

    #[test]
    fn manager_select_destination_cards_last_turn() {
        let mut m = Manager::new();

        let player_id = m.add_player().unwrap();
        let other_player_id = m.add_player().unwrap();

        assert!(m.set_ready(player_id, true).is_ok());
        assert!(m.set_ready(other_player_id, true).is_ok());

        m.phase = GamePhase::LastTurn;
        m.turn = Some(40);

        let (player_id_first, player_id_second) = if m.get_player_index(player_id) == Some(0) {
            (player_id, other_player_id)
        } else {
            (other_player_id, player_id)
        };

        m.players[0].get_mut_public_state().claimed_routes = vec![ClaimedRoute {
            route: (City::LosAngeles, City::SanFrancisco),
            parallel_route_index: 0,
            length: 3,
        }];

        assert_eq!(m.players[0].get_public_state().is_done_playing, false);
        assert!(m.players[0].get_public_state().has_longest_route.is_none());
        assert_eq!(m.players[1].get_public_state().is_done_playing, false);
        assert!(m.players[1].get_public_state().has_longest_route.is_none());

        let destination_cards_decisions = smallvec![true, false, true];
        assert!(m.draw_destination_cards(player_id_first).is_ok());
        assert!(m
            .select_destination_cards(player_id_first, destination_cards_decisions)
            .is_ok());

        assert_eq!(m.turn, Some(41));
        assert!(m.players[0].get_public_state().is_done_playing);
        assert!(m.players[0].get_public_state().has_longest_route.is_none());
        assert_eq!(m.players[1].get_public_state().is_done_playing, false);
        assert!(m.players[1].get_public_state().has_longest_route.is_none());

        let destination_cards_decisions = smallvec![true, false, true];
        assert!(m.draw_destination_cards(player_id_second).is_ok());
        assert!(m
            .select_destination_cards(player_id_second, destination_cards_decisions)
            .is_ok());

        assert_eq!(m.turn, Some(42));
        assert_eq!(m.phase, GamePhase::Done);
        assert!(m.players[0].get_public_state().is_done_playing);
        assert_eq!(
            m.players[0].get_public_state().has_longest_route,
            Some(true)
        );
        assert!(m.players[1].get_public_state().is_done_playing);
        assert_eq!(
            m.players[1].get_public_state().has_longest_route,
            Some(false)
        );
    }

    #[test]
    fn manager_draw_destination_cards() {
        let mut m = Manager::new();

        let player_id = m.add_player().unwrap();
        let other_player_id = m.add_player().unwrap();

        assert!(m.set_ready(player_id, true).is_ok());
        assert!(m.set_ready(other_player_id, true).is_ok());

        m.phase = GamePhase::Playing;
        m.turn = Some(0);

        let (player_id_first, player_id_second) = if m.get_player_index(player_id) == Some(0) {
            (player_id, other_player_id)
        } else {
            (other_player_id, player_id)
        };

        assert!(m.draw_destination_cards(player_id_first).is_ok());
        assert_eq!(
            m.players[0]
                .get_private_state()
                .pending_destination_cards
                .len(),
            NUM_DRAWN_DESTINATION_CARDS
        );
        // Can't draw again this turn.
        assert!(m.draw_destination_cards(player_id_first).is_err());
        // Wrong turn.
        assert!(m.draw_destination_cards(player_id_second).is_err());

        assert_eq!(m.turn, Some(0));

        let destination_cards_decisions = smallvec![true, false, false];
        assert!(m
            .select_destination_cards(player_id_first, destination_cards_decisions.clone())
            .is_ok());

        assert_eq!(m.turn, Some(1));

        // Next turn has started: it's not the first player's turn anymore.
        assert!(m.draw_destination_cards(player_id_first).is_err());
        assert!(m.draw_destination_cards(player_id_second).is_ok());
        // Can't draw again this turn.
        assert!(m.draw_destination_cards(player_id_second).is_err());

        assert_eq!(m.turn, Some(1));

        assert!(m
            .select_destination_cards(player_id_second, destination_cards_decisions.clone())
            .is_ok());

        assert_eq!(m.turn, Some(2));
    }

    #[test]
    fn manager_game_started() {
        let mut m = Manager::new();

        assert!(m.has_game_started().is_err());
        assert!(m.has_turn_based_game_started().is_err());

        m.phase = GamePhase::Starting;

        assert!(m.has_game_started().is_ok());
        assert!(m.has_turn_based_game_started().is_err());

        m.phase = GamePhase::Playing;

        assert!(m.has_game_started().is_ok());
        assert!(m.has_turn_based_game_started().is_ok());

        m.phase = GamePhase::LastTurn;

        assert!(m.has_game_started().is_ok());
        assert!(m.has_turn_based_game_started().is_ok());

        m.phase = GamePhase::Done;

        assert!(m.has_game_started().is_err());
        assert!(m.has_turn_based_game_started().is_err());
    }
}
