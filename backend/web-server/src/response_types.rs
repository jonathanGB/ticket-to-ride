use rocket::response::Redirect;
use rocket::serde::Serialize;

#[derive(Responder)]
pub enum LoadGameError {
    NoFile(std::io::Error),
    NoGame(Redirect),
    Unauthorized(Redirect),
}

#[derive(Serialize)]
#[serde(tag = "response", crate = "rocket::serde")]
pub enum ChangeNameResponse {
    Unauthorized,
    AlreadyUsed,
    Success,
}
