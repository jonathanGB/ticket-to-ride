//! All the types of JSON requests the server supports.
//!
//! These requests should be valid JSON (otherwise the server will refuse them),
//! part of the request body.

use rocket::serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use ticket_to_ride::{
    card::{TrainColor, NUM_DRAWN_DESTINATION_CARDS},
    city::CityToCity,
    player::PlayerColor,
};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(crate = "rocket::serde")]
/// Expected request when calling [`crate::router::change_player_name()`].
pub struct ChangeNameRequest {
    /// New player name.
    pub new_name: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(crate = "rocket::serde")]
/// Expected request when calling [`crate::router::change_player_color()`].
pub struct ChangeColorRequest {
    /// New player color.
    pub new_color: PlayerColor,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(crate = "rocket::serde")]
/// Expected request when calling [`crate::router::set_player_ready()`].
pub struct SetPlayerReadyRequest {
    /// Whether the player is ready or not.
    pub is_ready: bool,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(crate = "rocket::serde")]
/// Expected request when calling [`crate::router::select_destination_cards()`].
pub struct SelectDestinationCardsRequest {
    /// The player's decision regarding whether they want to select a given destination card, or not.
    ///
    /// Maps 1:1 to the _pending_ destination cards.
    pub destination_cards_decisions: SmallVec<[bool; NUM_DRAWN_DESTINATION_CARDS]>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(crate = "rocket::serde")]
/// Expected request when calling [`crate::router::draw_open_train_card()`].
pub struct DrawOpenTrainCardRequest {
    /// The index of the open train card to draw.
    pub card_index: usize,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(crate = "rocket::serde")]
/// Expected request when calling [`crate::router::claim_route()`].
pub struct ClaimRouteRequest {
    /// The route (pair of [`ticket_to_ride::city::City`]) to claim.
    pub route: CityToCity,
    /// As there can be many routes connecting two cities,
    /// the request must specify which of the _parallel_ routes they want to claim.
    pub parallel_route_index: usize,
    /// The train cards used to claim the route.
    pub cards: Vec<TrainColor>,
}

#[cfg(test)]
mod tests {
    use smallvec::smallvec;

    use super::*;

    #[test]
    fn json_to_change_name_request() -> serde_json::Result<()> {
        let request = ChangeNameRequest {
            new_name: String::from("joe"),
        };
        assert_eq!(
            serde_json::from_str::<ChangeNameRequest>(r#"{ "new_name": "joe" }"#)?,
            request
        );

        Ok(())
    }

    #[test]
    fn json_to_change_color_request() -> serde_json::Result<()> {
        let request = ChangeColorRequest {
            new_color: PlayerColor::Pink,
        };
        assert_eq!(
            serde_json::from_str::<ChangeColorRequest>(r#"{ "new_color": "pink" }"#)?,
            request
        );

        Ok(())
    }

    #[test]
    fn json_to_set_player_ready_request() -> serde_json::Result<()> {
        let request = SetPlayerReadyRequest { is_ready: true };
        assert_eq!(
            serde_json::from_str::<SetPlayerReadyRequest>(r#"{ "is_ready": true }"#)?,
            request
        );

        let request = SetPlayerReadyRequest { is_ready: false };
        assert_eq!(
            serde_json::from_str::<SetPlayerReadyRequest>(r#"{ "is_ready": false }"#)?,
            request
        );

        Ok(())
    }

    #[test]
    fn json_to_select_destination_cards_request() -> serde_json::Result<()> {
        let request = SelectDestinationCardsRequest {
            destination_cards_decisions: smallvec![true, false, true],
        };
        assert_eq!(
            serde_json::from_str::<SelectDestinationCardsRequest>(
                r#"{ "destination_cards_decisions": [true, false, true] }"#
            )?,
            request
        );

        Ok(())
    }

    #[test]
    fn json_to_draw_open_train_card_request() -> serde_json::Result<()> {
        let request = DrawOpenTrainCardRequest { card_index: 3 };
        assert_eq!(
            serde_json::from_str::<DrawOpenTrainCardRequest>(r#"{ "card_index": 3 }"#)?,
            request
        );

        Ok(())
    }
}
