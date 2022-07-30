use crate::city::{City, CityToCity};

use array_init::array_init;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::VecDeque;
use std::iter::repeat;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

const NUM_OPEN_TRAIN_CARDS: usize = 5;
const NUM_WILD_CARDS: usize = 14;
const NUM_NON_WILD_CARDS: usize = 12;
const WILD_CARD_LIMIT: usize = 3;
const NUM_DRAWN_DESTINATION_CARDS: usize = 3;
const NUM_DRAWN_INITIAL_TRAIN_CARDS: usize = 4;

/// Represents the different variants of train cards.
#[derive(Clone, Copy, Debug, Deserialize, Display, EnumIter, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum TrainColor {
    /// Also known as the *Hopper train*.
    Black,
    /// Also known as the *Tanker train*.
    Blue,
    /// Also known as the *Caboose train*.
    Green,
    /// Also known as the *Freight train*.
    Orange,
    /// Also known as the *Box train*.
    Pink,
    /// Also known as the *Coal train*.
    Red,
    /// Also known as the *Passenger train*.
    White,
    /// Also known as the *Locomotive*.
    /// This is a special train that matches with any color.
    Wild,
    /// Also known as the *Reefer train*.
    Yellow,
}

impl TrainColor {
    /// Whether the current color is wild, i.e. matches with any color.
    ///
    /// # Examples:
    /// ```
    /// use ticket_to_ride::card::TrainColor;
    ///
    /// let color = TrainColor::Black;
    /// assert!(!color.is_wild());
    ///
    /// let wild_color = TrainColor::Wild;
    /// assert!(wild_color.is_wild());
    /// ```
    #[inline]
    pub fn is_wild(&self) -> bool {
        *self == TrainColor::Wild
    }

    /// The opposite of `is_wild`.
    ///
    /// # Examples:
    /// ```
    /// use ticket_to_ride::card::TrainColor;
    ///
    /// let color = TrainColor::Black;
    /// assert!(color.is_not_wild());
    ///
    /// let wild_color = TrainColor::Wild;
    /// assert!(!wild_color.is_not_wild());
    /// ```
    #[inline]
    pub fn is_not_wild(&self) -> bool {
        !self.is_wild()
    }
}

/// Encapsulates information about a destination card.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DestinationCard {
    /// The two cities that must be connected to fulfill the destination card.
    pub destination: CityToCity,
    /// How many points are granted once this card is fulfilled.
    /// If not fulfilled, the same amount will rather be substracted.
    pub points: u8,
}

/// Convenience macro to generate a destination card.
macro_rules! destination_card {
    ($start:expr, $end:expr, $points:literal) => {
        DestinationCard {
            destination: ($start, $end),
            points: $points,
        }
    };
}

// TODO: document.
#[derive(Serialize)]
pub struct CardDealerState<'a> {
    open_train_card_deck: &'a [Option<TrainColor>],
    close_train_card_deck_size: usize,
    discarded_train_card_deck_size: usize,
    destination_card_deck_size: usize,
}

/// Entity in charge of dealing as well as shuffling destination and train cards.
#[derive(Debug)]
pub struct CardDealer {
    open_train_card_deck: SmallVec<[Option<TrainColor>; NUM_OPEN_TRAIN_CARDS]>,
    close_train_card_deck: Vec<TrainColor>,
    discarded_train_card_deck: Vec<TrainColor>,
    destination_card_deck: VecDeque<DestinationCard>,
}

impl CardDealer {
    /// Creates a new `CardDealer`, which starts with all decks shuffled and in a valid state.
    /// This means that the open train card deck does not exceed the limit number of wild cards (3).
    ///
    /// # Example
    /// ```
    /// use ticket_to_ride::card::CardDealer;
    ///
    /// let card_dealer = CardDealer::new();
    /// ```
    pub fn new() -> Self {
        let mut all_train_cards = Vec::with_capacity(110);

        for color in TrainColor::iter() {
            let num_of_train_cards_per_color = if color.is_wild() {
                NUM_WILD_CARDS
            } else {
                NUM_NON_WILD_CARDS
            };
            all_train_cards.extend(repeat(color).take(num_of_train_cards_per_color));
        }

        all_train_cards.shuffle(&mut thread_rng());

        let open_train_card_deck: SmallVec<_> = all_train_cards
            .iter_mut()
            .take(NUM_OPEN_TRAIN_CARDS)
            .map(|color| Some(*color))
            .collect();
        let close_train_card_deck: Vec<_> = all_train_cards
            .into_iter()
            .skip(NUM_OPEN_TRAIN_CARDS)
            .collect();

        let mut new_card_dealer = Self {
            open_train_card_deck,
            close_train_card_deck,
            discarded_train_card_deck: Vec::new(),
            destination_card_deck: Self::generate_destination_cards(),
        };

        new_card_dealer.maybe_reshuffle_open_train_card_deck();

        new_card_dealer
    }

    fn generate_destination_cards() -> VecDeque<DestinationCard> {
        let mut destination_cards = [
            destination_card! {City::Boston, City::Miami, 12},
            destination_card! {City::Calgary, City::Phoenix, 13},
            destination_card! {City::Calgary, City::SaltLakeCity, 7},
            destination_card! {City::Chicago, City::NewOrleans, 7},
            destination_card! {City::Chicago, City::SantaFe, 9},
            destination_card! {City::Dallas, City::NewYork, 11},
            destination_card! {City::Denver, City::ElPaso, 4},
            destination_card! {City::Denver, City::Pittsburgh, 11},
            destination_card! {City::Duluth, City::ElPaso, 10},
            destination_card! {City::Duluth, City::Houston, 8},
            destination_card! {City::Helena, City::LosAngeles, 8},
            destination_card! {City::KansasCity, City::Houston, 5},
            destination_card! {City::LosAngeles, City::Chicago, 16},
            destination_card! {City::LosAngeles, City::Miami, 20},
            destination_card! {City::LosAngeles, City::NewYork, 21},
            destination_card! {City::Montreal, City::Atlanta, 9},
            destination_card! {City::Montreal, City::NewOrleans, 13},
            destination_card! {City::NewYork, City::Atlanta, 6},
            destination_card! {City::Portland, City::Nashville, 17},
            destination_card! {City::Portland, City::Phoenix, 11},
            destination_card! {City::SanFrancisco, City::Atlanta, 17},
            destination_card! {City::SaultStMarie, City::Nashville, 8},
            destination_card! {City::SaultStMarie, City::OklahomaCity, 9},
            destination_card! {City::Seattle, City::LosAngeles, 9},
            destination_card! {City::Seattle, City::NewYork, 22},
            destination_card! {City::Toronto, City::Miami, 10},
            destination_card! {City::Vancouver, City::Montreal, 20},
            destination_card! {City::Vancouver, City::SantaFe, 13},
            destination_card! {City::Winnipeg, City::Houston, 12},
            destination_card! {City::Winnipeg, City::LittleRock, 11},
        ];

        destination_cards.shuffle(&mut thread_rng());
        VecDeque::from(destination_cards)
    }

    fn should_reshuffle_open_train_card_deck(&self) -> bool {
        let mut num_wild_cards_in_open_train_card_deck = 0;
        let mut num_non_wild_cards_in_open_train_card_deck = 0;
        for train_card in &self.open_train_card_deck {
            match train_card {
                Some(color) => {
                    if color.is_wild() {
                        num_wild_cards_in_open_train_card_deck += 1;
                    } else {
                        num_non_wild_cards_in_open_train_card_deck += 1;
                    }
                }
                None => {}
            }
        }

        // If there is less than 3 wild cards in the open deck, then we should not reshuffle.
        if num_wild_cards_in_open_train_card_deck < WILD_CARD_LIMIT {
            return false;
        }

        // Otherwise, we should reshuffle as long as there is at least 3 non-wild card in any decks.
        // If we did not verify that, we could end up reshuffling ad infinitum.
        let mut total_non_wild_cards_in_all_decks = num_non_wild_cards_in_open_train_card_deck;

        for deck in [&self.close_train_card_deck, &self.discarded_train_card_deck] {
            for train_card in deck {
                if train_card.is_not_wild() {
                    total_non_wild_cards_in_all_decks += 1;

                    if total_non_wild_cards_in_all_decks >= WILD_CARD_LIMIT {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn maybe_reshuffle_open_train_card_deck(&mut self) -> bool {
        if !self.should_reshuffle_open_train_card_deck() {
            return false;
        }

        // We should re-shuffle. Let's move cards from the open deck to the discarded deck.
        self.discarded_train_card_deck.extend(
            self.open_train_card_deck
                .drain(..)
                .filter_map(|train_card| train_card),
        );

        // Re-fill open deck from the close deck.
        for _ in 0..NUM_OPEN_TRAIN_CARDS {
            match self.close_train_card_deck.pop() {
                Some(color) => self.open_train_card_deck.push(Some(color)),
                None => break,
            }
        }

        self.maybe_reshuffle_and_swap_discarded_deck();

        // Make sure the open deck is full.
        // If not, re-fill the open deck again from the close deck.
        let num_open_train_cards = self.open_train_card_deck.len();
        for _ in 0..(NUM_OPEN_TRAIN_CARDS - num_open_train_cards) {
            match self.close_train_card_deck.pop() {
                Some(color) => self.open_train_card_deck.push(Some(color)),
                None => break,
            }
        }

        // We are done re-shuffling, but it is possible that the open deck again has
        // three or more wild cards in the open deck.
        self.maybe_reshuffle_open_train_card_deck();

        true
    }

    /// Draws from the top of the close train card deck, and returns the card.
    ///
    /// If there are no more cards left in that deck, returns an `Err`.
    ///
    /// If the close deck is empty after the draw is done, it will re-shuffle the discarded deck of train cards and swap it.
    ///
    /// # Example
    /// ```
    /// use ticket_to_ride::card::CardDealer;
    ///
    /// let mut card_dealer = CardDealer::new();
    ///
    /// match card_dealer.draw_from_close_train_card_deck() {
    ///     Ok(train_card) => println!("Picked {:?} from the close deck.", train_card),
    ///     Err(e) => println!("{}", e),
    /// }
    /// ```
    pub fn draw_from_close_train_card_deck(&mut self) -> Result<TrainColor, String> {
        match self.close_train_card_deck.pop() {
            Some(card_drawn) => {
                self.maybe_reshuffle_and_swap_discarded_deck();

                Ok(card_drawn)
            }
            None => Err(String::from(
                "There is no cards left in the close train card deck.",
            )),
        }
    }

    /// Draws a train card from the open deck, based on the given `card_index`.
    ///
    /// If the selected index has a wild card, this returns an error if `is_second_draw` is true;
    /// that is, we prevent selecting a wild card on a player's second draw in a given turn.
    ///
    /// If it finds a card, the specified slot will be replaced with the top card of the close train card deck.
    /// Based on the replacement card, the open train card deck may need to be re-shuffled.
    ///
    /// If it finds a card but the close train card deck is empty, the specified slot will be
    /// replaced with `None`, and no re-shuffle will occur.
    ///
    /// In both of these cases, it returns the selected card as well as a boolean indicating whether
    /// we had to re-shuffle the open train card deck upon replacing it.
    ///
    /// If there are no cards at that index, or if the index is out of bounds, returns `Err`.
    /// Note that at most 5 train cards are openly displayed at a time.
    ///
    /// # Example
    /// ```
    /// use ticket_to_ride::card::CardDealer;
    ///
    /// let mut card_dealer = CardDealer::new();
    /// let invalid_card_index = 6;
    /// let valid_card_index = 2;
    /// let is_second_draw = false;
    ///
    /// assert!(card_dealer.draw_from_open_train_card_deck(invalid_card_index, is_second_draw).is_err());
    ///
    /// match card_dealer.draw_from_open_train_card_deck(valid_card_index, is_second_draw) {
    ///     Ok(train_card) => println!("Picked {:?} from the open deck.", train_card),
    ///     Err(e) => println!("{}", e),
    /// }
    /// ```
    pub fn draw_from_open_train_card_deck(
        &mut self,
        card_index: usize,
        is_second_draw: bool,
    ) -> Result<(TrainColor, bool), String> {
        let card = self.peek_at_open_train_card(card_index)?;

        if is_second_draw && card.is_wild() {
            Err(String::from(
                "Cannot draw a wild card after having already drawn a train card this turn.",
            ))
        } else {
            self.open_train_card_deck[card_index] = self.draw_from_close_train_card_deck().ok();

            Ok((card, self.maybe_reshuffle_open_train_card_deck()))
        }
    }

    /// Draws three destination cards from the top of the destination cards deck.
    ///
    /// If there are less than three destination cards left in the deck, it will return what is left.
    ///
    /// However, if the destination card deck is empty, it returns an `Err`.
    ///
    /// # Example
    /// ```
    /// use ticket_to_ride::card::CardDealer;
    ///
    /// let mut card_dealer = CardDealer::new();
    ///
    /// let drawn_destination_cards = card_dealer.draw_from_destination_card_deck();
    /// assert!(drawn_destination_cards.is_ok());
    /// assert!(drawn_destination_cards.unwrap().len() <= 3);
    /// ```
    pub fn draw_from_destination_card_deck(
        &mut self,
    ) -> Result<SmallVec<[DestinationCard; NUM_DRAWN_DESTINATION_CARDS]>, String> {
        if self.destination_card_deck.is_empty() {
            return Err(String::from(
                "Cannot draw from the destination card deck, as it is empty.",
            ));
        }

        let mut drawn_destination_cards = SmallVec::new();

        for _ in 0..NUM_DRAWN_DESTINATION_CARDS {
            match self.destination_card_deck.pop_back() {
                Some(destination_card) => drawn_destination_cards.push(destination_card),
                None => break,
            }
        }

        Ok(drawn_destination_cards)
    }

    /// The first draw of the game, during the [`crate::manager::GamePhase::Starting`] phase, returns four train cards
    /// and three destination cards.
    ///
    /// # Example
    /// ```
    /// use ticket_to_ride::card::CardDealer;
    ///
    /// let mut card_dealer = CardDealer::new();
    ///
    /// let (starting_train_cards, starting_destination_cards) = card_dealer.initial_draw();
    /// ```
    pub fn initial_draw(
        &mut self,
    ) -> (
        [TrainColor; NUM_DRAWN_INITIAL_TRAIN_CARDS],
        [DestinationCard; NUM_DRAWN_DESTINATION_CARDS],
    ) {
        // Note that it is safe to unwrap in both cases, as initial draws cannot fail
        // considering the number of cards we start with, and the maximum number of players.
        (
            array_init(|_| self.draw_from_close_train_card_deck().unwrap()),
            self.draw_from_destination_card_deck()
                .unwrap()
                .into_inner()
                .unwrap(),
        )
    }

    /// Adds the given train cards to the deck of discarded train cards.
    ///
    /// If the close train card deck is empty, we re-shuffle the discarded deck and swap it.
    /// # Example
    /// ```
    /// use ticket_to_ride::card::{CardDealer, TrainColor};
    ///
    /// let mut card_dealer = CardDealer::new();
    /// let train_cards_to_discard = vec![TrainColor::Red, TrainColor::Wild];
    ///
    /// card_dealer.discard_train_cards(train_cards_to_discard);
    /// ```
    pub fn discard_train_cards(&mut self, train_cards: Vec<TrainColor>) {
        // Note that insertion order in the discard deck does not matter.
        self.discarded_train_card_deck.extend(train_cards);

        self.maybe_reshuffle_and_swap_discarded_deck();
    }

    /// Adds the given destination cards to the bottom of the destination cards deck.
    ///
    /// If players go through all the undiscarded destination cards, they will cycle through
    /// the discarded destination cards.
    /// # Example
    /// ```
    /// use smallvec::smallvec;
    /// use ticket_to_ride::card::{CardDealer, DestinationCard};
    /// use ticket_to_ride::city::City;
    ///
    /// let mut card_dealer = CardDealer::new();
    /// let destination_cards_to_discard = smallvec![
    ///     DestinationCard {
    ///         destination: (City::Chicago, City::SantaFe),
    ///         points: 9,
    ///     },
    /// ];
    ///
    /// card_dealer.discard_destination_cards(destination_cards_to_discard);
    /// ```
    pub fn discard_destination_cards(
        &mut self,
        destination_cards: SmallVec<[DestinationCard; NUM_DRAWN_DESTINATION_CARDS]>,
    ) {
        for destination_card in destination_cards {
            self.destination_card_deck.push_front(destination_card);
        }
    }

    #[inline]
    fn maybe_reshuffle_and_swap_discarded_deck(&mut self) {
        if !self.close_train_card_deck.is_empty() || self.discarded_train_card_deck.is_empty() {
            return;
        }

        self.discarded_train_card_deck.shuffle(&mut thread_rng());

        std::mem::swap(
            &mut self.close_train_card_deck,
            &mut &mut self.discarded_train_card_deck,
        );
    }

    #[inline]
    fn peek_at_open_train_card(&self, card_index: usize) -> Result<TrainColor, String> {
        if card_index >= self.open_train_card_deck.len() {
            return Err(format!(
                "Card looked up at index {} is out of bounds (size {}).",
                card_index,
                self.open_train_card_deck.len()
            ));
        }

        match self.open_train_card_deck[card_index] {
            Some(card) => Ok(card),
            None => Err(format!("No cards found at index {}.", card_index)),
        }
    }

    /// Predicate that determines whether should be allowed to draw a train card again this turn.
    ///
    /// This is separate to the rule of not being allowed to draw a train card after having already
    /// drawn an open wild card. This helper ensures that if a player drew a card from the close deck,
    /// or a non-wild card from the open deck, that there are still cards the player could draw on that
    /// same turn.
    ///
    /// Specifically, if there are no cards left in any deck, or if the only cards left are wild cards in
    /// the open deck, then the player cannot draw again, thus we will terminate their turn earlier.
    #[inline]
    pub fn can_player_draw_again_this_turn(&self) -> bool {
        !self.close_train_card_deck.is_empty()
            || self
                .open_train_card_deck
                .iter()
                .any(|card| card.is_some() && card.unwrap().is_not_wild())
    }

    /// Mutable accessor to the open train card deck.
    ///
    /// Should only be used for testing!
    pub fn get_mut_open_train_card_deck(
        &mut self,
    ) -> &mut SmallVec<[Option<TrainColor>; NUM_OPEN_TRAIN_CARDS]> {
        &mut self.open_train_card_deck
    }

    /// Accessor to the close train card deck.
    ///
    /// Should only be used for testing!
    pub fn get_close_train_card_deck(&self) -> &Vec<TrainColor> {
        &self.close_train_card_deck
    }

    /// Mutable accessor to the close train card deck.
    ///
    /// Should only be used for testing!
    pub fn get_mut_close_train_card_deck(&mut self) -> &mut Vec<TrainColor> {
        &mut self.close_train_card_deck
    }

    /// Accessor to the discarded train card deck.
    ///
    /// Should only be used for testing!
    pub fn get_discarded_train_card_deck(&self) -> &Vec<TrainColor> {
        &self.discarded_train_card_deck
    }

    /// Accessor to the destination card deck.
    ///
    /// Should only be used for testing!
    pub fn get_destination_card_deck(&self) -> &VecDeque<DestinationCard> {
        &self.destination_card_deck
    }

    /// Mutable accessor to the destination card deck.
    ///
    /// Should only be used for testing!
    pub fn get_mut_destination_card_deck(&mut self) -> &mut VecDeque<DestinationCard> {
        &mut self.destination_card_deck
    }

    // TODO: test this.
    pub fn get_state(&self) -> CardDealerState {
        CardDealerState {
            open_train_card_deck: &self.open_train_card_deck,
            close_train_card_deck_size: self.close_train_card_deck.len(),
            discarded_train_card_deck_size: self.discarded_train_card_deck.len(),
            destination_card_deck_size: self.destination_card_deck.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    // Tests for `TrainColor`.

    #[test]
    fn train_color_to_string() {
        assert_eq!(TrainColor::Orange.to_string(), "orange");
        assert_eq!(TrainColor::Pink.to_string(), "pink");
    }

    #[test]
    fn train_color_to_json() -> serde_json::Result<()> {
        assert_eq!(serde_json::to_string(&TrainColor::Blue)?, r#""blue""#);
        assert_eq!(serde_json::to_string(&TrainColor::Red)?, r#""red""#);
        Ok(())
    }

    #[test]
    fn json_to_train_color() -> serde_json::Result<()> {
        assert_eq!(
            serde_json::from_str::<TrainColor>(r#""wild""#)?,
            TrainColor::Wild
        );
        assert_eq!(
            serde_json::from_str::<TrainColor>(r#""green""#)?,
            TrainColor::Green
        );

        Ok(())
    }

    #[test]
    fn invalid_json_to_train_color() {
        assert!(serde_json::from_str::<TrainColor>(r#""turquoise""#).is_err());
    }

    // Tests for `CardDealer`.

    #[test]
    fn new_card_dealer() {
        let card_dealer = CardDealer::new();

        assert_eq!(card_dealer.open_train_card_deck.len(), NUM_OPEN_TRAIN_CARDS);
        assert!(
            card_dealer
                .open_train_card_deck
                .iter()
                .filter_map(|train_card| *train_card)
                .filter(|color| color.is_wild())
                .count()
                < WILD_CARD_LIMIT
        );

        // 110 cards total, minus 5 in the open train card deck.
        assert_eq!(
            card_dealer.close_train_card_deck.len() + card_dealer.discarded_train_card_deck.len(),
            105
        );
        assert_eq!(card_dealer.destination_card_deck.len(), 30);

        let mut num_train_cards_per_color = HashMap::new();

        for train_card in &card_dealer.open_train_card_deck {
            assert!(train_card.is_some());
            num_train_cards_per_color
                .entry(train_card.unwrap())
                .and_modify(|num| *num += 1)
                .or_insert(1);
        }

        for deck in [
            &card_dealer.close_train_card_deck,
            &card_dealer.discarded_train_card_deck,
        ] {
            for train_card in deck {
                num_train_cards_per_color
                    .entry(*train_card)
                    .and_modify(|num| *num += 1)
                    .or_insert(1);
            }
        }

        for color in TrainColor::iter() {
            let expected_num = if color.is_wild() {
                NUM_WILD_CARDS
            } else {
                NUM_NON_WILD_CARDS
            };
            assert_eq!(num_train_cards_per_color[&color], expected_num);
        }
    }

    #[test]
    fn new_card_dealer_different_every_time() {
        // With 110 cards (12 cards for the 8 train cards, and 14 wild cards),
        // there is technically a 110! / (12!^8 * 14!) of generating the same deck twice.
        // As for the destination deck, that probability is 30!.
        let first_card_dealer = CardDealer::new();
        let second_card_dealer = CardDealer::new();

        assert_ne!(
            first_card_dealer.close_train_card_deck,
            second_card_dealer.close_train_card_deck
        );
        assert_ne!(
            first_card_dealer.destination_card_deck,
            second_card_dealer.destination_card_deck
        );
    }

    #[test]
    fn card_dealer_should_reshuffle() {
        let mut card_dealer = CardDealer::new();
        let open_train_card_deck = [
            Some(TrainColor::Wild),
            Some(TrainColor::Red),
            Some(TrainColor::Black),
            Some(TrainColor::Wild),
            Some(TrainColor::Wild),
        ];
        let close_train_card_deck = [
            TrainColor::Red,
            TrainColor::Orange,
            TrainColor::Black,
            TrainColor::Green,
            TrainColor::Blue,
        ];
        card_dealer.open_train_card_deck = open_train_card_deck.into();
        card_dealer.close_train_card_deck = close_train_card_deck.clone().into();
        card_dealer.discarded_train_card_deck.clear();

        // There should be a re-shuffle. What happens afterwards is as follows:
        //  1. The open deck is moved to the discarded deck.
        //  2. The close deck is popped into the open deck.
        //  3. The close deck is empty, so we shuffle the discarded deck and swap with the close deck.
        assert!(card_dealer.maybe_reshuffle_open_train_card_deck());
        assert_eq!(
            card_dealer.open_train_card_deck,
            close_train_card_deck
                .iter()
                .rev()
                .map(|color| Some(*color))
                .collect::<SmallVec<[_; NUM_OPEN_TRAIN_CARDS]>>()
        );
        assert!(card_dealer.discarded_train_card_deck.is_empty());

        let mut num_train_cards_per_color = HashMap::new();
        for train_card in &card_dealer.close_train_card_deck {
            num_train_cards_per_color
                .entry(*train_card)
                .and_modify(|num| *num += 1)
                .or_insert(1);
        }

        // Based on the initial `open_train_card_deck`.
        assert_eq!(num_train_cards_per_color[&TrainColor::Wild], 3);
        assert_eq!(num_train_cards_per_color[&TrainColor::Red], 1);
        assert_eq!(num_train_cards_per_color[&TrainColor::Black], 1);
    }

    #[test]
    fn card_dealer_should_not_reshuffle_if_under_wild_card_limit() {
        let mut card_dealer = CardDealer::new();
        let open_train_card_deck = [
            Some(TrainColor::Blue),
            Some(TrainColor::Red),
            Some(TrainColor::Black),
            Some(TrainColor::Wild),
            Some(TrainColor::Wild),
        ];
        card_dealer.open_train_card_deck = open_train_card_deck.into();

        assert!(!card_dealer.maybe_reshuffle_open_train_card_deck());
        assert_eq!(
            card_dealer.open_train_card_deck,
            open_train_card_deck.into()
        );
    }

    #[test]
    fn card_dealer_should_not_reshuffle_if_not_enough_non_wild_cards_left() {
        let mut card_dealer = CardDealer::new();
        let open_train_card_deck = [
            Some(TrainColor::Wild),
            None,
            Some(TrainColor::Black),
            Some(TrainColor::Wild),
            Some(TrainColor::Wild),
        ];
        card_dealer.open_train_card_deck = open_train_card_deck.into();
        card_dealer.close_train_card_deck.clear();
        card_dealer.discarded_train_card_deck.clear();

        assert!(!card_dealer.maybe_reshuffle_open_train_card_deck());
        assert_eq!(
            card_dealer.open_train_card_deck,
            open_train_card_deck.into()
        );
    }

    #[test]
    fn card_dealer_draw_from_close_deck() {
        let mut card_dealer = CardDealer::new();
        let expected_card = card_dealer.close_train_card_deck.last().cloned();
        assert_eq!(
            card_dealer.draw_from_close_train_card_deck().ok(),
            expected_card
        );

        card_dealer.close_train_card_deck = vec![TrainColor::Blue];
        card_dealer.discarded_train_card_deck = vec![TrainColor::Red];

        assert_eq!(
            card_dealer.draw_from_close_train_card_deck(),
            Ok(TrainColor::Blue)
        );
        assert!(card_dealer.discarded_train_card_deck.is_empty());
        assert_eq!(
            card_dealer.draw_from_close_train_card_deck(),
            Ok(TrainColor::Red)
        );
        assert!(card_dealer.draw_from_close_train_card_deck().is_err());
    }

    #[test]
    fn car_dealer_draw_from_open_deck_err() {
        let mut card_dealer = CardDealer::new();
        card_dealer.open_train_card_deck = [
            Some(TrainColor::Blue),
            None,
            Some(TrainColor::Black),
            Some(TrainColor::Wild),
            Some(TrainColor::Wild),
        ]
        .into();

        assert!(card_dealer
            .draw_from_open_train_card_deck(/*card_index=*/ 1, /*is_second_draw= */ false)
            .is_err());
        assert!(card_dealer
            .draw_from_open_train_card_deck(/*card_index=*/ 6, /*is_second_draw= */ false)
            .is_err());
    }

    #[test]
    fn car_dealer_draw_from_open_deck_wild() {
        let mut card_dealer = CardDealer::new();
        card_dealer.open_train_card_deck = [
            Some(TrainColor::Blue),
            None,
            Some(TrainColor::Black),
            Some(TrainColor::Wild),
            Some(TrainColor::Wild),
        ]
        .into();

        assert_eq!(
            card_dealer
                .draw_from_open_train_card_deck(/*card_index=*/ 3, /*is_second_draw= */ false),
            Ok((TrainColor::Wild, false))
        );

        // 4th index is also a wild card, which returns an error if `is_second_draw` is enabled.
        assert!(card_dealer
            .draw_from_open_train_card_deck(/*card_index=*/ 4, /*is_second_draw= */ true)
            .is_err());
    }

    #[test]
    fn car_dealer_draw_from_open_deck_empty_close_deck() {
        let mut card_dealer = CardDealer::new();
        card_dealer.open_train_card_deck = [
            Some(TrainColor::White),
            None,
            Some(TrainColor::Black),
            Some(TrainColor::Wild),
            Some(TrainColor::Wild),
        ]
        .into();
        card_dealer.close_train_card_deck.clear();

        assert_eq!(
            card_dealer
                .draw_from_open_train_card_deck(/*card_index=*/ 0, /*is_second_draw */ false),
            Ok((TrainColor::White, false))
        );
        assert!(card_dealer.open_train_card_deck[0].is_none());
    }

    #[test]
    fn car_dealer_draw_from_open_deck_reshuffle() {
        let mut card_dealer = CardDealer::new();
        card_dealer.open_train_card_deck = [
            Some(TrainColor::White),
            Some(TrainColor::Red),
            Some(TrainColor::Black),
            Some(TrainColor::Wild),
            Some(TrainColor::Wild),
        ]
        .into();
        card_dealer.close_train_card_deck = vec![
            TrainColor::Blue,
            TrainColor::Blue,
            TrainColor::Blue,
            TrainColor::Blue,
            TrainColor::Blue,
            TrainColor::Wild,
        ];

        assert_eq!(
            card_dealer
                .draw_from_open_train_card_deck(/*card_index=*/ 0, /*is_second_draw= */ false),
            Ok((TrainColor::White, true))
        );
        assert_eq!(card_dealer.open_train_card_deck.len(), NUM_OPEN_TRAIN_CARDS);

        for train_card in card_dealer.open_train_card_deck {
            assert_eq!(train_card, Some(TrainColor::Blue));
        }
    }

    #[test]
    fn card_dealer_discard_train_card_with_non_empty_close_deck() {
        let mut card_dealer = CardDealer::new();
        let discard_cards = vec![TrainColor::Yellow];
        let close_cards = vec![TrainColor::Pink];
        card_dealer.close_train_card_deck = close_cards.clone();
        card_dealer.discarded_train_card_deck.clear();

        card_dealer.discard_train_cards(discard_cards.clone());
        assert_eq!(card_dealer.close_train_card_deck, close_cards);
        assert_eq!(card_dealer.discarded_train_card_deck, discard_cards);
    }

    #[test]
    fn card_dealer_discard_train_card_with_empty_close_deck() {
        let mut card_dealer = CardDealer::new();
        let discard_cards = vec![TrainColor::Yellow];
        card_dealer.close_train_card_deck.clear();
        card_dealer.discarded_train_card_deck.clear();

        card_dealer.discard_train_cards(discard_cards.clone());
        assert_eq!(card_dealer.close_train_card_deck, discard_cards);
        assert!(card_dealer.discarded_train_card_deck.is_empty());
    }

    #[test]
    fn card_dealer_draw_destination_card() {
        let mut card_dealer = CardDealer::new();
        assert_eq!(card_dealer.destination_card_deck.len(), 30);
        let expected_destination_cards: SmallVec<[_; NUM_DRAWN_DESTINATION_CARDS]> = card_dealer
            .destination_card_deck
            .iter()
            .skip(27)
            .rev()
            .cloned()
            .collect();

        assert_eq!(
            card_dealer.draw_from_destination_card_deck(),
            Ok(expected_destination_cards)
        );
    }

    #[test]
    fn card_dealer_draw_destination_card_use_discarded_cards() {
        let mut card_dealer = CardDealer::new();
        let only_destination_card = destination_card! {City::Boston, City::Montreal, 5};
        card_dealer.destination_card_deck = VecDeque::from([only_destination_card.clone()]);

        let discarded_destination_cards = smallvec![
            destination_card! {City::Duluth, City::Vancouver, 15},
            destination_card! {City::LosAngeles, City::ElPaso, 6},
        ];
        card_dealer.discard_destination_cards(discarded_destination_cards.clone());

        assert_eq!(
            card_dealer.draw_from_destination_card_deck(),
            Ok([
                only_destination_card,
                discarded_destination_cards[0].clone(),
                discarded_destination_cards[1].clone()
            ]
            .into())
        );
    }

    #[test]
    fn card_dealer_draw_destination_card_partial() {
        let mut card_dealer = CardDealer::new();
        let only_destination_card = destination_card! {City::Boston, City::Montreal, 5};
        card_dealer.destination_card_deck = VecDeque::from([only_destination_card.clone()]);

        let mut expected_destination_cards: SmallVec<[DestinationCard; 3]> = SmallVec::new();
        expected_destination_cards.push(only_destination_card);
        assert_eq!(
            card_dealer.draw_from_destination_card_deck(),
            Ok(expected_destination_cards)
        );
    }

    #[test]
    fn card_dealer_draw_destination_empty() {
        let mut card_dealer = CardDealer::new();
        card_dealer.destination_card_deck.clear();

        assert!(card_dealer.draw_from_destination_card_deck().is_err());
    }

    #[test]
    fn card_dealer_initial_draw() {
        let mut card_dealer = CardDealer::new();
        let train_cards_drawn: [_; NUM_DRAWN_INITIAL_TRAIN_CARDS] = card_dealer
            .close_train_card_deck
            .iter()
            .rev()
            .take(NUM_DRAWN_INITIAL_TRAIN_CARDS)
            .cloned()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let destination_cards_drawn: [_; NUM_DRAWN_DESTINATION_CARDS] = card_dealer
            .destination_card_deck
            .iter()
            .rev()
            .take(NUM_DRAWN_DESTINATION_CARDS)
            .cloned()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        assert_eq!(
            card_dealer.initial_draw(),
            (train_cards_drawn, destination_cards_drawn)
        );
    }

    // Accessor tests.

    #[test]
    fn card_dealer_open_train_card_mut_accessor() {
        let mut card_dealer = CardDealer::new();
        let mut open_train_card_deck = card_dealer.open_train_card_deck.clone();

        assert_eq!(
            card_dealer.get_mut_open_train_card_deck(),
            &mut open_train_card_deck
        );
    }

    #[test]
    fn card_dealer_close_train_card_accessor() {
        let card_dealer = CardDealer::new();

        assert_eq!(
            card_dealer.get_close_train_card_deck(),
            &card_dealer.close_train_card_deck
        );
    }

    #[test]
    fn card_dealer_close_train_card_mut_accessor() {
        let mut card_dealer = CardDealer::new();
        let mut close_train_card_deck = card_dealer.close_train_card_deck.clone();

        assert_eq!(
            card_dealer.get_mut_close_train_card_deck(),
            &mut close_train_card_deck
        );
    }

    #[test]
    fn card_dealer_discarded_train_card_accessor() {
        let card_dealer = CardDealer::new();

        assert_eq!(
            card_dealer.get_discarded_train_card_deck(),
            &card_dealer.discarded_train_card_deck
        );
    }

    #[test]
    fn card_dealer_destination_card_accessor() {
        let card_dealer = CardDealer::new();

        assert_eq!(
            card_dealer.get_destination_card_deck(),
            &card_dealer.destination_card_deck
        );
    }

    #[test]
    fn card_dealer_destination_card_mut_accessor() {
        let mut card_dealer = CardDealer::new();
        let mut destination_card_deck = card_dealer.destination_card_deck.clone();

        assert_eq!(
            card_dealer.get_mut_destination_card_deck(),
            &mut destination_card_deck
        );
    }

    // Micro-benchmarks.

    use test::Bencher;

    #[bench]
    fn card_dealer(b: &mut Bencher) {
        b.iter(|| test::black_box(CardDealer::new()));
    }
}
