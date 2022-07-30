use rocket::response::Redirect;
use rocket::serde::Serialize;
use ticket_to_ride::manager::ManagerActionResult;

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
    pub fn new(manager_action_result: ManagerActionResult) -> Self {
        match manager_action_result {
            Ok(_) => Self {
                success: true,
                error_message: None,
            },
            Err(e) => Self {
                success: false,
                error_message: Some(e),
            },
        }
    }
}
