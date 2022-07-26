use rocket::serde::Deserialize;
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
