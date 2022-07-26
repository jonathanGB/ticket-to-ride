use rocket::response::Redirect;
use rocket::serde::Serialize;

#[derive(Responder)]
pub enum LoadGameError {
    NoFile(std::io::Error),
    NoGame(Redirect),
    Unauthorized(Redirect),
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ActionResponse {
    success: bool,
    error_message: Option<String>,
}

impl ActionResponse {
    pub fn new_success() -> Self {
        Self {
            success: true,
            error_message: None,
        }
    }

    pub fn new_failure(error_message: String) -> Self {
        Self {
            success: false,
            error_message: Some(error_message),
        }
    }
}
