use crate::card::{CardDealer, DestinationCard, TrainColor};
use crate::city::CityToCity;
use crate::map::{ClaimedRoute, Map};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use strum::IntoEnumIterator;

// Every player starts the game with 45 cards.
const NUM_OF_CARS: u8 = 45;

/// All actions taken by a player result in the same result:
/// either it succeeded, which we mark by whether the player's turn is over,
/// or whether it failed, which includes a human-readable error message.
pub type ActionResult = Result<bool, String>;

/// Every player has their own color.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayerColor {
    Black,
    Blue,
    Green,
    Orange,
    Pink,
    Red,
    Yellow,
    White,
}

/// Represents all the actions that a player can take.
/// Used internally to keep track of whether an action is allowed,
/// based on other actions taken by the player in a given turn.
#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    /// The first and only player action per turn.
    ClaimedRoute,
    /// The first and only player action per turn.
    DrewOpenWildTrainCard,
    /// The first or second of two player actions per turn.
    /// Must be followed by `DrewOpenNonWildCardTrain`, or
    /// `DrewCloseTrainCard`.
    DrewOpenNonWildTrainCard,
    /// The first or second of two player actions per turn.
    /// Must be followed by `DrawOpenNonWildCardTrain`, or
    /// `DrawCloseTrainCard`.
    DrewCloseTrainCard,
    /// The first action of two player actions per turn.
    /// Must be followed by `SelectedDestinationCards`.
    DrewDestinationCards,
    /// The second action of two player actions per turn.
    /// Must be preceded by `DrewDestinationCards`.
    /// One exception: the initial selection of destination cards,
    /// which happens before turns have started.
    SelectedDestinationCards,
}

/// Keeps track of actions taken at a given turn.
struct TurnActions {
    /// Initially, `turn` is None. This denotes the initial draw that happens concurrently for all players,
    /// before turns have started.
    /// When we start the turn-based game, then actions are coupled to a turn, which is monotonically increasing.
    turn: Option<usize>,
    /// For a given turn, a player can take at most two actions.
    /// More details in `PlayerAction`.
    actions: SmallVec<[PlayerAction; 2]>,
    /// Human-readable description of the corresponding action that was taken by the player.
    /// This is used to share updates with other players, so no private information is shared in it.
    description: SmallVec<[String; 2]>,
}

impl TurnActions {
    fn new() -> Self {
        Self {
            turn: None,
            actions: SmallVec::new(),
            description: SmallVec::new(),
        }
    }
}

/// Encapsulates all the player information and actions.
///
/// The [`Player`] assumes, if called to do an action, that this is their turn.
/// It will nonetheless verify that it is allowed to do that action this turn,
/// e.g. it will refuse to claim a route if it has already drawn one train card this turn.
///
/// A [`Player`] is not aware of other players in this game: thus, management of inter-player
/// state (e.g. ensuring unique names, verifying whether we should transition to the "end game") are
/// to be taken care of by the [`crate::game_state::Manager`].
pub struct Player {
    /// Unique to each player in the game.
    /// Requests from the web client are authenticated using this id.
    id: usize,
    /// Unique to each player in the game.
    name: String,
    /// Unique to each player in the game.
    color: PlayerColor,
    /// Denotes whether the player to transition from the lobby, and start the game.
    is_ready: bool,
    /// Denotes whether the player is done playing.
    /// That is, once a player has less than three cars left, everyone has one turn left to play.
    /// Once that last turn is over, they are done playing.
    is_done_playing: bool,
    /// The number of cars the player has left.
    /// This is the currency used, alongside train cards, to claim routes.
    cars: u8,
    /// How many points the player has so far.
    /// Points are gained by claiming routes, and at the end of the game we grant extra points for
    /// completed destination cards (or penalize if unfulfilled) and a bonus for longest route.
    points: u8,
    /// Maps how many of a train color a player has.
    /// It is guaranteed that the map has at all times key-value pairs for all train colors.
    train_cards: HashMap<TrainColor, u8>,
    /// After having drawn destination cards, we place them in this "pending" state.
    /// The player will subsequently have to select which ones they want to keep, which are then
    /// moved to the `selected_destination_cards`.
    pending_destination_cards: SmallVec<[DestinationCard; 3]>,
    /// List of destination cards that a player has selected to fulfill.
    /// These cards, when initially drawn, were first moved to the "pending" list, until they made
    /// their way here upon selection.
    selected_destination_cards: Vec<DestinationCard>,
    /// Actions taken by the player during the last turn they have participated in.
    turn_actions: TurnActions,
    /// List of routes claimed by the player.
    claimed_routes: Vec<ClaimedRoute>,

    /// Shared reference by all players to the map.
    map: Option<Rc<Map>>,
    /// Shared reference by all players to the card dealer.
    card_dealer: Option<Rc<RefCell<CardDealer>>>,
}

impl Player {
    /// Creates a new player.
    pub fn new(id: usize, color: PlayerColor) -> Self {
        let train_cards = HashMap::from_iter(TrainColor::iter().map(|color| (color, 0)));

        Self {
            id,
            name: format!("Player {}", id),
            color,
            is_ready: false,
            is_done_playing: false,
            cars: NUM_OF_CARS,
            points: 0,
            train_cards,
            pending_destination_cards: SmallVec::new(),
            selected_destination_cards: Vec::new(),
            turn_actions: TurnActions::new(),
            claimed_routes: Vec::new(),
            map: None,
            card_dealer: None,
        }
    }

    /// Initializes the player with information to start the game.
    ///
    /// The [`crate::game_state::Manager`] must call this once the game has started, meaning we are out of the
    /// [`crate::game_phase::GamePhase::InLobby`] phase.
    ///
    /// Takes care of the special initial draw of train and destination cards, and initializes references
    /// to the [`Map`] and the [`CardDealer`].
    pub fn initialize_when_game_starts(
        &mut self,
        map: Rc<Map>,
        card_dealer: Rc<RefCell<CardDealer>>,
    ) {
        let (initial_train_cards, initial_destination_cards) =
            card_dealer.borrow_mut().initial_draw();

        for train_card in initial_train_cards {
            self.train_cards
                .entry(train_card)
                .and_modify(|count| *count += 1);
        }

        self.pending_destination_cards
            .extend(initial_destination_cards);

        self.map = Some(map);
        self.card_dealer = Some(card_dealer);
    }

    /// Change the player's name.
    /// This should be unique across players of the game.
    #[inline]
    pub fn change_name(&mut self, name: String) {
        self.name = name;
    }

    /// Access the player's name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Change the player's color.
    /// This should be unique across players of the game.
    #[inline]
    pub fn change_color(&mut self, color: PlayerColor) {
        self.color = color;
    }

    /// Access the player's color.
    #[inline]
    pub fn color(&self) -> PlayerColor {
        self.color
    }

    /// Set whether a player is ready to start the game.
    #[inline]
    pub fn set_ready(&mut self, is_ready: bool) {
        self.is_ready = is_ready;
    }

    /// Access whether a player is ready to start the game.
    #[inline]
    pub fn ready(&self) -> bool {
        self.is_ready
    }

    /// Access how many cars a player has left.
    #[inline]
    pub fn cars(&self) -> u8 {
        self.cars
    }

    /// Set whether a player has taken their last turn of the game.
    #[inline]
    pub fn set_done_playing(&mut self) {
        self.is_done_playing = true;
    }

    // Safe to call as long as the game has started.
    // Otherwise, it will panic.
    #[inline]
    fn get_map(&self) -> &Map {
        self.map.as_ref().unwrap()
    }

    // Safe to call as long as the game has started.
    // Otherwise, it will panic.
    #[inline]
    fn get_card_dealer(&self) -> Ref<CardDealer> {
        self.card_dealer.as_ref().unwrap().borrow()
    }

    // Safe to call as long as the game has started.
    // Otherwise, it will panic.
    #[inline]
    fn get_mut_card_dealer(&self) -> RefMut<CardDealer> {
        self.card_dealer.as_ref().unwrap().borrow_mut()
    }

    /// Clears the turn's actions, and overrides it with the given action and description.
    #[inline]
    fn replace_turn_action(&mut self, turn: usize, action: PlayerAction, description: String) {
        self.turn_actions.turn = Some(turn);

        self.turn_actions.actions.clear();
        self.turn_actions.actions.push(action);

        self.turn_actions.description.clear();
        self.turn_actions.description.push(description);
    }

    /// Append the given action and description to the turn's actions.
    #[inline]
    fn append_turn_action(&mut self, action: PlayerAction, description: String) {
        self.turn_actions.actions.push(action);
        self.turn_actions.description.push(description);
    }

    fn claimed_route_description(
        &self,
        claimed_route: &ClaimedRoute,
        num_wild_cards: u8,
        non_wild_cards: Option<(TrainColor, u8)>,
    ) -> String {
        let cards_used_description = match (num_wild_cards, non_wild_cards) {
            (num_wild_cards, Some((color, num_non_wild_cards))) if num_wild_cards > 0 => {
                format!(
                    "{} wild cards and {} {} cards",
                    num_wild_cards, num_non_wild_cards, color
                )
            }
            (_, Some((color, num_non_wild_cards))) => {
                format!("{} {} cards", num_non_wild_cards, color)
            }
            (num_wild_cards, _) if num_wild_cards > 0 => {
                format!("{} wild cards", num_wild_cards)
            }
            _ => unreachable!(),
        };

        let (start, end) = claimed_route.route;
        let points = Map::calculate_points_for_claimed_route(claimed_route.length);
        format!(
          "{} has claimed a route between {} and {} of length {} ({} points). They did so using {}.",
          self.name, start, end, claimed_route.length, points, cards_used_description
      )
    }

    /// Try to claim a route for a player.
    ///
    /// Returns an `Err` if either:
    ///   * There was already an action taken this turn.
    ///   * There are not enough cars to claim this route.
    ///   * The player does not have enough of the specified card(s) in their inventory.
    ///   * The underlying [`Map::claim_route_for_player`] disallows the claim.
    ///
    /// Otherwise, claims the route, does a bunch of bookkeeping, and returns `Ok(true)`
    /// to denote that the player's turn is over.
    pub fn claim_route(
        &mut self,
        route: CityToCity,
        parallel_route_index: usize,
        cards: Vec<TrainColor>,
        turn: usize,
    ) -> ActionResult {
        if let Some(last_turn) = self.turn_actions.turn {
            if last_turn == turn {
                return Err(String::from(
                    "Cannot claim route if you have drawn a train card or destination cards this turn."
                ));
            }
        }

        if cards.len() > self.cars as usize {
            return Err(format!(
                "Cannot claim route from {} to {} with {} cards, whilst having only {} cars left.",
                route.0,
                route.1,
                cards.len(),
                self.cars
            ));
        }

        let mut num_wild_cards = 0;
        let mut non_wild_cards = None;
        for card in &cards {
            if card.is_wild() {
                num_wild_cards += 1;
            } else {
                // We don't override the last non-wild-card as we go.
                // Note that this is technically wrong if the hand illegally contains more
                // than one color, but this is already verified by `Map::claim_route_for_player`.
                non_wild_cards = match non_wild_cards {
                    Some((color, num)) => Some((color, num + 1)),
                    None => Some((*card, 1)),
                }
            }
        }

        if num_wild_cards > 0 {
            let inventory_wild_cards = self.train_cards.get(&TrainColor::Wild).unwrap();

            if inventory_wild_cards < &num_wild_cards {
                return Err(format!(
                    "Cannot claim a route using {} wild cards, whilst having only {} left.",
                    num_wild_cards, inventory_wild_cards
                ));
            }
        }

        if let Some((color, num)) = non_wild_cards {
            let inventory_non_wild_cards = self.train_cards.get(&color).unwrap();

            if inventory_non_wild_cards < &num {
                return Err(format!(
                    "Cannot claim a route using {} {} cards, whilst having only {} left.",
                    num, color, inventory_non_wild_cards
                ));
            }
        }

        // Try to claim the route.
        let claimed_route =
            self.get_map()
                .claim_route_for_player(route, parallel_route_index, &cards, self.id)?;

        // At this point, we have successfully claimed the route. Some player bookkeeping is in order.

        self.replace_turn_action(
            turn,
            PlayerAction::ClaimedRoute,
            self.claimed_route_description(&claimed_route, num_wild_cards, non_wild_cards),
        );

        if num_wild_cards > 0 {
            self.train_cards
                .entry(TrainColor::Wild)
                .and_modify(|inventory_wild_cards| *inventory_wild_cards -= num_wild_cards);
        }

        if let Some((color, num)) = non_wild_cards {
            self.train_cards
                .entry(color)
                .and_modify(|inventory_non_wild_cards| {
                    *inventory_non_wild_cards -= num;
                });
        }

        self.points += Map::calculate_points_for_claimed_route(claimed_route.length);
        self.cars -= claimed_route.length;
        self.claimed_routes.push(claimed_route);
        self.get_mut_card_dealer().discard_train_cards(cards);

        // Turn is over.
        Ok(true)
    }

    #[inline]
    fn drew_open_train_card_description(&self, card: TrainColor, reshuffled: bool) -> String {
        if reshuffled {
            format!("{} drew a {} train card from the open deck. The open deck was then re-shuffled because there were three wild cards.", self.name, card)
        } else {
            format!(
                "{} drew a {} train card from the open deck.",
                self.name, card
            )
        }
    }

    /// Try to draw a train card from the open deck, at the given `card_index`.
    ///
    /// Returns an `Err` if either:
    ///   * They have drawn a destination card as the first action of the given turn.
    ///   * There is no card at this index.
    ///   * This is their second draw, and the card drawn is a wild card.
    ///
    /// Otherwise, draws the given card, stores it, and returns `Ok` encapsulating whether the turn is over.
    /// The turn is over (`true`) if this was their second draw this turn,
    /// or if there is no valid cards left to draw anyway.
    pub fn draw_open_train_card(&mut self, card_index: usize, turn: usize) -> ActionResult {
        let turn_second_draw = match self.turn_actions.turn {
            Some(last_turn) if last_turn != turn => false,
            Some(_) => {
                if self.turn_actions.actions[0] == PlayerAction::DrewDestinationCards {
                    return Err(format!("Cannot draw a train card after having already drawn destination cards this turn."));
                } else {
                    true
                }
            }
            None => false,
        };

        let (card, reshuffled) = self
            .get_mut_card_dealer()
            .draw_from_open_train_card_deck(card_index, turn_second_draw)?;

        // Drew card successfully. Store that card, and decide whether the player's turn is over.

        self.train_cards.entry(card).and_modify(|count| *count += 1);

        let action_description = self.drew_open_train_card_description(card, reshuffled);
        if card.is_wild() {
            self.replace_turn_action(
                turn,
                PlayerAction::DrewOpenWildTrainCard,
                action_description,
            );

            // Turn is over after drawing an open wild card.
            Ok(true)
        } else if turn_second_draw {
            self.append_turn_action(PlayerAction::DrewOpenNonWildTrainCard, action_description);

            // Turn is over if this was the second draw this turn.
            Ok(true)
        } else {
            self.replace_turn_action(
                turn,
                PlayerAction::DrewOpenNonWildTrainCard,
                action_description,
            );

            // Turn is over if there is no valid cards to be drawn this turn.
            Ok(!self.get_card_dealer().can_player_draw_again_this_turn())
        }
    }

    fn drew_close_train_card_description(&self) -> String {
        format!("{} drew a train card from the close deck.", self.name)
    }

    /// Try to draw a train card from the close deck.
    ///
    /// Returns an `Err` if either:
    ///  * They have drawn a destination card as the first action of the given turn.
    ///  * There is no cards left in the close deck.
    ///
    /// Otherwise, draws the top card, stores it, and returns `Ok` encapsulating whether the turn is over.
    /// The turn is over (`true`) if this was their second draw this turn,
    /// or if there is no valid cards left to draw anyway.
    pub fn draw_close_train_card(&mut self, turn: usize) -> ActionResult {
        let turn_second_draw = match self.turn_actions.turn {
            Some(last_turn) if last_turn != turn => false,
            Some(_) => {
                if self.turn_actions.actions[0] == PlayerAction::DrewDestinationCards {
                    return Err(format!("Cannot draw a train card after having already drawn destination cards this turn."));
                } else {
                    true
                }
            }
            None => false,
        };

        let card = self
            .get_mut_card_dealer()
            .draw_from_close_train_card_deck()?;

        self.train_cards.entry(card).and_modify(|count| *count += 1);

        let description = self.drew_close_train_card_description();
        if turn_second_draw {
            self.append_turn_action(PlayerAction::DrewCloseTrainCard, description);

            // Turn is over if this was the second draw this turn.
            Ok(true)
        } else {
            self.replace_turn_action(turn, PlayerAction::DrewCloseTrainCard, description);

            // Turn is over if there is no valid cards to be drawn this turn.
            Ok(!self.get_card_dealer().can_player_draw_again_this_turn())
        }
    }

    #[inline]
    fn drew_destination_card_description(&self) -> String {
        format!(
            "{} drew {} destination cards. They have not selected which to keep yet.",
            self.name,
            self.pending_destination_cards.len()
        )
    }

    /// Try to draw destination cards.
    ///
    /// Returns an `Err` if either:
    ///   * There was already an action taken this turn.
    ///   * The destination card deck is empty.
    ///
    /// Otherwise, draws up three destination cards, stores them as *pending*,
    /// and returns `Ok(false)` to denote that the turn is not over yet.
    /// Indeed, the player is expected to end the turn by first selecting
    /// which of the drawn destination cards they want to keep -- via
    /// [`Player::select_destination_cards`].
    ///
    /// Note that less than tree destination cards may be returned, if there were
    /// less than three left in the deck.
    pub fn draw_destination_cards(&mut self, turn: usize) -> ActionResult {
        if let Some(last_turn) = self.turn_actions.turn {
            if last_turn == turn {
                return Err(format!(
                    "Cannot draw destination cards if you have drawn a train card this turn."
                ));
            }
        }

        let mut destination_cards = self
            .get_mut_card_dealer()
            .draw_from_destination_card_deck()?;

        std::mem::swap(&mut self.pending_destination_cards, &mut destination_cards);

        self.replace_turn_action(
            turn,
            PlayerAction::DrewDestinationCards,
            self.drew_destination_card_description(),
        );

        // Turns is never over when drawing from the destination deck.
        // The player must select the destination cards drawn to terminate it.
        Ok(false)
    }

    #[inline]
    fn selected_destination_cards_description(&self, num: usize) -> String {
        format!(
            "{} selected {} destination cards out of {}.",
            self.name,
            num,
            self.pending_destination_cards.len()
        )
    }

    /// Try to select from the set of destination cards already drawn this turn -- via
    /// [`Player::draw_destination_cards`].
    ///
    /// Returns an `Err` if either:
    ///   * The given `destination_cards_decisions` don't have the same length as the pending set.
    ///   * Fewer destination cards are selected than the minimum required. Specifically:
    ///     * On the initial draw (which is denoted via a `turn` set to `None`), at least two
    ///       out of three must be selected.
    ///     * Otherwise, at least one card must be selected.
    ///
    /// Otherwise, moves the selected destination cards from *pending* to *selected*, clears the
    /// *pending* list, and returns `Ok(true)` to denote that the player's turn is over.
    ///
    /// The caller shares the selection decision via an array of booleans. As its size must
    /// match the size of the *pending* list, and both lists are ordered, we can map them 1:1
    /// based on their indices: if `destination_cards_decisions[i] == true`, then `pending[i]`
    /// is selected.
    pub fn select_destination_cards(
        &mut self,
        destination_cards_decisions: SmallVec<[bool; 3]>,
        turn: Option<usize>,
    ) -> ActionResult {
        if destination_cards_decisions.len() != self.pending_destination_cards.len() {
            return Err(format!(
                "Submitted {} destination cards decisions, but {} were drawn.",
                destination_cards_decisions.len(),
                self.pending_destination_cards.len()
            ));
        }

        let min_to_select = match (self.turn_actions.turn, turn) {
            (Some(last_turn), Some(turn)) => {
                if last_turn != turn {
                    return Err(String::from("Cannot select destination cards before having drawn destination cards first."));
                } else if self.turn_actions.actions[0] != PlayerAction::DrewDestinationCards {
                    return Err(String::from(
                        "Cannot select destination cards after having drawn a train card.",
                    ));
                }

                // On a normal turn, at least one destination card must be selected.
                1
            }
            // On the initial draw, at least two destination cards must be selected.
            (None, None) => 2,
            _ => unreachable!(),
        };

        let num_selected = destination_cards_decisions
            .iter()
            .filter(|destination_card| **destination_card)
            .count();
        if num_selected < min_to_select {
            return Err(format!(
                "Cannot select only {} destination cards, whilst the minimum is {}.",
                num_selected, min_to_select
            ));
        }

        // We have validated that the player can select the given cards.
        self.append_turn_action(
            PlayerAction::SelectedDestinationCards,
            self.selected_destination_cards_description(num_selected),
        );

        // Note that we iterate backwards, because `remove` shifts all elements after the removed item.
        // Going forward would thus break the mapping we implicitly have using indices.
        let mut discarded_destination_cards = SmallVec::new();
        for i in (0..destination_cards_decisions.len()).rev() {
            let destination_card = self.pending_destination_cards.remove(i);

            if destination_cards_decisions[i] {
                self.selected_destination_cards.push(destination_card);
            } else {
                discarded_destination_cards.push(destination_card);
            }
        }

        self.get_mut_card_dealer()
            .discard_destination_cards(discarded_destination_cards);

        // Selecting destination cards always ends the turn.
        Ok(true)
    }

    // TODO: add an "end game" function that calculates how many destination cards are fulfilled,
    // and what the player's longest route is.

    // TODO: figure out we want to represent the public and private state of the player.

    // pub fn get_state_for_player(&self, player_id: usize) -> PlayerState {
    //     unimplemented!()
    // }
}

// pub struct PlayerState {
//     public_player_state: PublicPlayerState,
//     private_player_state: Option<PrivatePlayerState>,
// }

// pub struct PublicPlayerState {}

// pub struct PrivatePlayerState {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::City;

    // Tests for `PlayerColor`.

    #[test]
    fn player_color_to_json() -> serde_json::Result<()> {
        assert_eq!(serde_json::to_string(&PlayerColor::Blue)?, r#""blue""#);
        assert_eq!(serde_json::to_string(&PlayerColor::Red)?, r#""red""#);
        Ok(())
    }

    #[test]
    fn json_to_player_color() -> serde_json::Result<()> {
        assert_eq!(
            serde_json::from_str::<PlayerColor>(r#""pink""#)?,
            PlayerColor::Pink
        );
        assert_eq!(
            serde_json::from_str::<PlayerColor>(r#""green""#)?,
            PlayerColor::Green
        );

        Ok(())
    }

    #[test]
    fn invalid_json_to_player_color() {
        assert!(serde_json::from_str::<PlayerColor>(r#""turquoise""#).is_err());
    }

    // Tests for `Player`.
    const PLAYER_ID: usize = 0;
    const PLAYER_COLOR: PlayerColor = PlayerColor::Orange;

    #[test]
    fn player_new() {
        let player = Player::new(PLAYER_ID, PLAYER_COLOR);
        assert_eq!(player.id, PLAYER_ID);
        assert_eq!(player.color, PLAYER_COLOR);
        assert_eq!(player.name, format!("Player {}", PLAYER_ID));
        assert_eq!(player.is_ready, false);
        assert_eq!(player.is_done_playing, false);
        assert_eq!(player.cars, NUM_OF_CARS);
        assert_eq!(player.points, 0);
        assert!(player.pending_destination_cards.is_empty());
        assert!(player.selected_destination_cards.is_empty());
        assert_eq!(player.turn_actions.turn, None);
        assert!(player.turn_actions.actions.is_empty());
        assert!(player.turn_actions.description.is_empty());
        assert!(player.claimed_routes.is_empty());
        assert!(player.map.is_none());
        assert!(player.card_dealer.is_none());
    }

    #[test]
    fn player_change() {
        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        assert_eq!(player.color(), PLAYER_COLOR);
        assert_eq!(player.name(), format!("Player {}", PLAYER_ID));
        assert_eq!(player.ready(), false);
        assert_eq!(player.is_done_playing, false);

        let new_color = PlayerColor::Yellow;
        player.change_color(new_color);
        assert_eq!(player.color(), new_color);

        let new_name = String::from("NewPlayer");
        player.change_name(new_name.clone());
        assert_eq!(player.name(), new_name);

        player.set_ready(true);
        assert!(player.ready());
        player.set_ready(false);
        assert!(!player.ready());

        player.set_done_playing();
        assert!(player.is_done_playing);
    }

    #[test]
    fn player_initialize_when_game_starts() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));
        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);

        let sum_train_cards: u8 = player.train_cards.values().sum();
        assert_eq!(sum_train_cards, 4);
        assert_eq!(player.pending_destination_cards.len(), 3);
        assert!(player.map.is_some());
        assert!(player.card_dealer.is_some());
        assert!(player.selected_destination_cards.is_empty());
    }

    #[test]
    fn player_claim_route_same_turn() {
        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);

        let route = (City::Chicago, City::Pittsburgh);
        let route_index = 0;
        let cards = vec![TrainColor::Black, TrainColor::Black, TrainColor::Black];
        let turn = 5;

        player.turn_actions.turn = Some(turn);

        assert_eq!(
            player.claim_route(route, route_index, cards, turn),
            Err(String::from(
                "Cannot claim route if you have drawn a train card or destination cards this turn."
            ))
        );
    }

    #[test]
    fn player_claim_route_missing_cars() {
        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);

        let route = (City::Chicago, City::Pittsburgh);
        let route_index = 0;
        let cards = vec![TrainColor::Black, TrainColor::Black, TrainColor::Black];
        let turn = 5;

        player.cars = 2;

        assert_eq!(
            player.claim_route(route, route_index, cards, turn),
            Err(String::from(
                "Cannot claim route from Chicago to Pittsburgh with 3 cards, whilst having only 2 cars left.",
            ))
        );
        assert!(player.claimed_routes.is_empty());
    }

    #[test]
    fn player_claim_route_missing_wild_cars() {
        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);

        let route = (City::Chicago, City::Pittsburgh);
        let route_index = 0;
        let cards = vec![TrainColor::Wild, TrainColor::Wild, TrainColor::Black];
        let turn = 5;

        player.train_cards.insert(TrainColor::Wild, 1);

        assert_eq!(
            player.claim_route(route, route_index, cards, turn),
            Err(String::from(
                "Cannot claim a route using 2 wild cards, whilst having only 1 left.",
            ))
        );
        assert!(player.claimed_routes.is_empty());
    }

    #[test]
    fn player_claim_route_missing_non_wild_cars() {
        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);

        let route = (City::Chicago, City::Pittsburgh);
        let route_index = 0;
        let cards = vec![TrainColor::Wild, TrainColor::Black, TrainColor::Black];
        let turn = 5;

        player.train_cards.insert(TrainColor::Wild, 1);
        player.train_cards.insert(TrainColor::Black, 1);

        assert_eq!(
            player.claim_route(route, route_index, cards, turn),
            Err(String::from(
                "Cannot claim a route using 2 black cards, whilst having only 1 left.",
            ))
        );
        assert!(player.claimed_routes.is_empty());
    }

    #[test]
    fn player_claim_route_map_returns_err() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let route = (City::Chicago, City::Pittsburgh);
        let route_index = 0;
        let cards = vec![TrainColor::Wild, TrainColor::Black, TrainColor::Black];
        let turn = 5;

        assert!(map
            .claim_route_for_player(route, route_index, &cards, PLAYER_ID)
            .is_ok());

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);

        player.train_cards.insert(TrainColor::Wild, 1);
        player.train_cards.insert(TrainColor::Black, 2);

        let route_index = 1;
        assert_eq!(
            player.claim_route(route, route_index, cards, turn),
            Err(String::from(
                "Cannot claim more than one route between Chicago and Pittsburgh.",
            ))
        );
        assert!(player.claimed_routes.is_empty());
    }

    #[test]
    fn player_claim_route() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let route = (City::Chicago, City::Pittsburgh);
        let parallel_route_index = 0;
        let cards = vec![TrainColor::Wild, TrainColor::Black, TrainColor::Black];
        let turn = 5;

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);

        player.train_cards.insert(TrainColor::Wild, 1);
        player.train_cards.insert(TrainColor::Black, 3);

        assert_eq!(
            player.claim_route(route, parallel_route_index, cards.clone(), turn),
            Ok(true)
        );

        assert_eq!(player.turn_actions.turn, Some(turn));
        assert_eq!(player.turn_actions.actions.len(), 1);
        assert_eq!(player.turn_actions.actions[0], PlayerAction::ClaimedRoute);
        assert_eq!(player.turn_actions.description.len(), 1);
        assert_eq!(
            player.turn_actions.description[0],
            String::from(
                "Player 0 has claimed a route between Chicago and Pittsburgh of length 3 (4 points). They did so using 1 wild cards and 2 black cards."
            )
        );

        // Based on the cards used to claim the route.
        assert_eq!(player.train_cards.get(&TrainColor::Wild), Some(&0));
        assert_eq!(player.train_cards.get(&TrainColor::Black), Some(&1));

        assert_eq!(player.points, 4);
        assert_eq!(player.cars, NUM_OF_CARS - 3);
        assert_eq!(
            player.claimed_routes,
            vec![ClaimedRoute {
                route,
                parallel_route_index,
                length: 3
            }]
        );

        assert!(player.card_dealer.is_some());
        let discarded_train_cards = player
            .card_dealer
            .unwrap()
            .borrow()
            .get_discarded_train_card_deck()
            .clone();
        assert!(discarded_train_cards.len() >= 3);
        assert_eq!(
            discarded_train_cards.as_slice()[discarded_train_cards.len() - 3..],
            cards
        );
    }

    #[test]
    fn player_draw_open_train_card_drawn_destination_card_already() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let card_index = 0;
        let turn = 5;

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        player.turn_actions.turn = Some(turn);
        player
            .turn_actions
            .actions
            .push(PlayerAction::DrewDestinationCards);

        assert_eq!(
            player.draw_open_train_card(card_index, turn),
            Err(String::from(
                "Cannot draw a train card after having already drawn destination cards this turn."
            ))
        );
    }

    #[test]
    fn player_draw_open_train_card_wild_card_second_draw() {
        let map = Rc::new(Map::new(2).unwrap());
        let mut card_dealer = CardDealer::new();

        let card_index = 0;
        let turn = 5;

        card_dealer.get_mut_open_train_card_deck()[card_index] = Some(TrainColor::Wild);
        let card_dealer = Rc::new(RefCell::new(card_dealer));

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        player.turn_actions.turn = Some(turn);
        player
            .turn_actions
            .actions
            .push(PlayerAction::DrewCloseTrainCard);

        assert_eq!(
            player.draw_open_train_card(card_index, turn),
            Err(String::from(
                "Cannot draw a wild card after having already drawn a train card this turn."
            ))
        );
    }

    #[test]
    fn player_draw_open_train_card_wild_card_first_draw() {
        let map = Rc::new(Map::new(2).unwrap());
        let mut card_dealer = CardDealer::new();

        let card_index = 0;
        let turn = 5;
        let selected_card = TrainColor::Wild;

        card_dealer.get_mut_open_train_card_deck()[card_index] = Some(selected_card);
        let card_dealer = Rc::new(RefCell::new(card_dealer));

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        let inventory_wild_cards = player.train_cards.get(&selected_card).cloned().unwrap();

        assert_eq!(player.draw_open_train_card(card_index, turn), Ok(true));
        assert_eq!(
            player.train_cards.get(&selected_card).cloned(),
            Some(inventory_wild_cards + 1)
        );

        assert_eq!(player.turn_actions.turn, Some(turn));
        assert_eq!(player.turn_actions.actions.len(), 1);
        assert_eq!(
            player.turn_actions.actions[0],
            PlayerAction::DrewOpenWildTrainCard
        );
        assert_eq!(player.turn_actions.description.len(), 1);
        assert_eq!(
            player.turn_actions.description[0],
            String::from("Player 0 drew a wild train card from the open deck.")
        );
    }

    #[test]
    fn player_draw_open_train_card_non_wild_card_first_draw() {
        let map = Rc::new(Map::new(2).unwrap());
        let mut card_dealer = CardDealer::new();

        let card_index = 0;
        let turn = 5;
        let selected_card = TrainColor::Red;

        card_dealer.get_mut_open_train_card_deck()[card_index] = Some(selected_card);
        let card_dealer = Rc::new(RefCell::new(card_dealer));

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        let inventory_wild_cards = player.train_cards.get(&selected_card).cloned().unwrap();

        assert_eq!(player.draw_open_train_card(card_index, turn), Ok(false));
        assert_eq!(
            player.train_cards.get(&selected_card).cloned(),
            Some(inventory_wild_cards + 1)
        );

        assert_eq!(player.turn_actions.turn, Some(turn));
        assert_eq!(player.turn_actions.actions.len(), 1);
        assert_eq!(
            player.turn_actions.actions[0],
            PlayerAction::DrewOpenNonWildTrainCard
        );
        assert_eq!(player.turn_actions.description.len(), 1);
        assert_eq!(
            player.turn_actions.description[0],
            String::from("Player 0 drew a red train card from the open deck.")
        );
    }

    #[test]
    fn player_draw_open_train_card_non_wild_card_second_draw() {
        let map = Rc::new(Map::new(2).unwrap());
        let mut card_dealer = CardDealer::new();

        let card_index = 0;
        let turn = 5;
        let selected_card = TrainColor::Red;

        card_dealer.get_mut_open_train_card_deck()[card_index] = Some(selected_card);
        let card_dealer = Rc::new(RefCell::new(card_dealer));

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        player.turn_actions.turn = Some(turn);
        player
            .turn_actions
            .actions
            .push(PlayerAction::DrewCloseTrainCard);
        player.turn_actions.description.push(String::new());
        let inventory_wild_cards = player.train_cards.get(&selected_card).cloned().unwrap();

        assert_eq!(player.draw_open_train_card(card_index, turn), Ok(true));
        assert_eq!(
            player.train_cards.get(&selected_card).cloned(),
            Some(inventory_wild_cards + 1)
        );

        assert_eq!(player.turn_actions.turn, Some(turn));
        assert_eq!(player.turn_actions.actions.len(), 2);
        assert_eq!(
            player.turn_actions.actions[1],
            PlayerAction::DrewOpenNonWildTrainCard
        );
        assert_eq!(player.turn_actions.description.len(), 2);
        assert_eq!(
            player.turn_actions.description[1],
            String::from("Player 0 drew a red train card from the open deck.")
        );
    }

    #[test]
    fn player_draw_close_train_card_drawn_destination_card_already() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let turn = 5;

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        player.turn_actions.turn = Some(turn);
        player
            .turn_actions
            .actions
            .push(PlayerAction::DrewDestinationCards);

        assert_eq!(
            player.draw_close_train_card(turn),
            Err(String::from(
                "Cannot draw a train card after having already drawn destination cards this turn."
            ))
        );
    }

    #[test]
    fn player_draw_close_train_card_first_draw() {
        let map = Rc::new(Map::new(2).unwrap());
        let mut card_dealer = CardDealer::new();

        let turn = 5;
        let selected_card = TrainColor::Wild;

        // Insert the wild card 4 cards under the top, so it reaches the top of the deck
        // after the initial draw.
        let close_train_card_deck_len = card_dealer.get_close_train_card_deck().len();
        card_dealer
            .get_mut_close_train_card_deck()
            .insert(close_train_card_deck_len - 4, selected_card);
        let card_dealer = Rc::new(RefCell::new(card_dealer));

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        player.turn_actions.turn = Some(turn - 1);

        let inventory_wild_cards = player.train_cards.get(&selected_card).cloned().unwrap();

        assert_eq!(player.draw_close_train_card(turn), Ok(false));
        assert_eq!(
            player.train_cards.get(&selected_card).cloned(),
            Some(inventory_wild_cards + 1)
        );
        assert_eq!(player.turn_actions.turn, Some(turn));
        assert_eq!(player.turn_actions.actions.len(), 1);
        assert_eq!(
            player.turn_actions.actions[0],
            PlayerAction::DrewCloseTrainCard
        );
        assert_eq!(player.turn_actions.description.len(), 1);
        assert_eq!(
            player.turn_actions.description[0],
            String::from("Player 0 drew a train card from the close deck.")
        );
    }

    #[test]
    fn player_draw_close_train_card_second_draw() {
        let map = Rc::new(Map::new(2).unwrap());
        let mut card_dealer = CardDealer::new();

        let turn = 5;
        let selected_card = TrainColor::Green;

        // Insert the selected card 4 cards under the top, so it reaches the top of the deck
        // after the initial draw.
        let close_train_card_deck_len = card_dealer.get_close_train_card_deck().len();
        card_dealer
            .get_mut_close_train_card_deck()
            .insert(close_train_card_deck_len - 4, selected_card);
        let card_dealer = Rc::new(RefCell::new(card_dealer));

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.turn_actions.turn = Some(turn);
        player
            .turn_actions
            .actions
            .push(PlayerAction::DrewCloseTrainCard);
        player.turn_actions.description.push(String::new());
        player.initialize_when_game_starts(map, card_dealer);

        let inventory_wild_cards = player.train_cards.get(&selected_card).cloned().unwrap();

        assert_eq!(player.draw_close_train_card(turn), Ok(true));
        assert_eq!(
            player.train_cards.get(&selected_card).cloned(),
            Some(inventory_wild_cards + 1)
        );
        assert_eq!(player.turn_actions.turn, Some(turn));
        assert_eq!(player.turn_actions.actions.len(), 2);
        assert_eq!(
            player.turn_actions.actions[1],
            PlayerAction::DrewCloseTrainCard
        );
        assert_eq!(player.turn_actions.description.len(), 2);
        assert_eq!(
            player.turn_actions.description[1],
            String::from("Player 0 drew a train card from the close deck.")
        );
    }

    #[test]
    fn player_draw_destination_card_drawn_train_card_already() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let turn = 5;

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        player.turn_actions.turn = Some(turn);
        player
            .turn_actions
            .actions
            .push(PlayerAction::DrewCloseTrainCard);

        assert_eq!(
            player.draw_destination_cards(turn),
            Err(String::from(
                "Cannot draw destination cards if you have drawn a train card this turn."
            ))
        );
    }

    #[test]
    fn player_draw_destination_card_drawn_train_card_emtpy() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = CardDealer::new();

        let card_dealer = Rc::new(RefCell::new(card_dealer));

        let turn = 5;

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        player
            .card_dealer
            .as_mut()
            .unwrap()
            .borrow_mut()
            .get_mut_destination_card_deck()
            .clear();

        assert_eq!(
            player.draw_destination_cards(turn),
            Err(String::from(
                "Cannot draw from the destination card deck, as it is empty."
            ))
        );
    }

    #[test]
    fn player_draw_destination_card() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let turn = 5;

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        player.pending_destination_cards.clear();

        let expected_destination_cards: Vec<DestinationCard> = player
            .card_dealer
            .as_ref()
            .unwrap()
            .borrow()
            .get_destination_card_deck()
            .iter()
            .rev()
            .take(3)
            .cloned()
            .collect();

        assert_eq!(player.draw_destination_cards(turn), Ok(false));
        assert_eq!(
            player.pending_destination_cards.as_slice(),
            expected_destination_cards
        );
        assert_eq!(player.turn_actions.turn, Some(turn));
        assert_eq!(player.turn_actions.actions.len(), 1);
        assert_eq!(
            player.turn_actions.actions[0],
            PlayerAction::DrewDestinationCards
        );
        assert_eq!(player.turn_actions.description.len(), 1);
        assert_eq!(
            player.turn_actions.description[0],
            String::from(
                "Player 0 drew 3 destination cards. They have not selected which to keep yet."
            )
        );
    }

    #[test]
    fn player_select_destination_card_initial_wrong_size() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let turn = None;

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);

        let selected_cards = smallvec![true, true];

        assert_eq!(
            player.select_destination_cards(selected_cards, turn),
            Err(String::from(
                "Submitted 2 destination cards decisions, but 3 were drawn."
            ))
        );
    }

    #[test]
    fn player_select_destination_card_initial_not_enough_selected() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let turn = None;

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);

        let selected_cards = smallvec![true, false, false];

        assert_eq!(
            player.select_destination_cards(selected_cards, turn),
            Err(String::from(
                "Cannot select only 1 destination cards, whilst the minimum is 2."
            ))
        );
    }

    #[test]
    fn player_select_destination_card_initial() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let turn = None;

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);

        let selected_destination_cards_decisions = smallvec![true, false, true];
        // The selected cards are inserted in opposite order of what they are in the pending list.
        let selected_destination_cards = vec![
            player.pending_destination_cards[2].clone(),
            player.pending_destination_cards[0].clone(),
        ];
        let discarded_destination_card = player.pending_destination_cards[1].clone();
        assert_eq!(
            player.select_destination_cards(selected_destination_cards_decisions, turn),
            Ok(true)
        );
        assert!(player.turn_actions.turn.is_none());
        assert_eq!(player.turn_actions.actions.len(), 1);
        assert_eq!(
            player.turn_actions.actions[0],
            PlayerAction::SelectedDestinationCards
        );
        assert_eq!(player.turn_actions.description.len(), 1);
        assert_eq!(
            player.turn_actions.description[0],
            String::from("Player 0 selected 2 destination cards out of 3.")
        );
        assert_eq!(
            player.selected_destination_cards,
            selected_destination_cards
        );
        assert!(player.pending_destination_cards.is_empty());
        assert_eq!(
            player
                .card_dealer
                .as_ref()
                .unwrap()
                .borrow()
                .get_destination_card_deck()
                .front(),
            Some(&discarded_destination_card)
        );
    }

    #[test]
    fn player_select_destination_card_not_initial_drawn_train_card_already() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let turn = Some(5);

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        player.turn_actions.turn = turn;
        player
            .turn_actions
            .actions
            .push(PlayerAction::DrewCloseTrainCard);
        player.turn_actions.description.push(String::from(""));

        let selected_cards = smallvec![true, false, false];

        assert_eq!(
            player.select_destination_cards(selected_cards, turn),
            Err(String::from(
                "Cannot select destination cards after having drawn a train card."
            ))
        );
    }

    #[test]
    fn player_select_destination_card_not_initial_not_enough_selected() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let turn = Some(5);

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        player.turn_actions.turn = turn;
        player
            .turn_actions
            .actions
            .push(PlayerAction::DrewDestinationCards);
        player.turn_actions.description.push(String::from(""));

        let selected_cards = smallvec![false, false, false];

        assert_eq!(
            player.select_destination_cards(selected_cards, turn),
            Err(String::from(
                "Cannot select only 0 destination cards, whilst the minimum is 1."
            ))
        );
    }

    #[test]
    fn player_select_destination_card_not_initial() {
        let map = Rc::new(Map::new(2).unwrap());
        let card_dealer = Rc::new(RefCell::new(CardDealer::new()));

        let turn = Some(5);

        let mut player = Player::new(PLAYER_ID, PLAYER_COLOR);
        player.initialize_when_game_starts(map, card_dealer);
        player.turn_actions.turn = turn;
        player
            .turn_actions
            .actions
            .push(PlayerAction::DrewDestinationCards);
        player.turn_actions.description.push(String::from(""));

        let selected_destination_cards_decisions = smallvec![false, true, false];
        let selected_destination_cards = vec![player.pending_destination_cards[1].clone()];
        let discarded_destination_cards = vec![
            player.pending_destination_cards[0].clone(),
            player.pending_destination_cards[2].clone(),
        ];
        assert_eq!(
            player.select_destination_cards(selected_destination_cards_decisions, turn),
            Ok(true)
        );
        assert_eq!(player.turn_actions.turn, turn);
        assert_eq!(player.turn_actions.actions.len(), 2);
        assert_eq!(
            player.turn_actions.actions[1],
            PlayerAction::SelectedDestinationCards
        );
        assert_eq!(player.turn_actions.description.len(), 2);
        assert_eq!(
            player.turn_actions.description[1],
            String::from("Player 0 selected 1 destination cards out of 3.")
        );
        assert_eq!(
            player.selected_destination_cards,
            selected_destination_cards
        );
        assert!(player.pending_destination_cards.is_empty());
        assert_eq!(
            player
                .card_dealer
                .as_ref()
                .unwrap()
                .borrow()
                .get_destination_card_deck()
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<_>>(),
            discarded_destination_cards
        );
    }
}
