use rocket::serde::Deserialize;
use smallvec::SmallVec;
use ticket_to_ride::player::PlayerColor;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ChangeNameRequest {
    pub new_name: String,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ChangeColorRequest {
    pub new_color: PlayerColor,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SetPlayerReadyRequest {
    pub is_ready: bool,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SelectDestinationCardsRequest {
    pub destination_cards_decisions: SmallVec<[bool; 3]>,
}
