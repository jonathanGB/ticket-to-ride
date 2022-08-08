//! All the custom responses the server supports.

use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use ticket_to_ride::manager::ManagerActionResult;

/// Types of error when loading a game.
#[derive(Responder)]
pub enum LoadGameError {
    NoFile(std::io::Error),
    NoGame(Redirect),
    Unauthorized(Redirect),
}

/// The general response to player actions, serializable in JSON.
#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ActionResponse {
    /// Whether the action succeeded.
    pub success: bool,
    /// If the action succeeded, this is `None`.
    ///
    /// If the action failed, a human-readable error message is provided.
    pub error_message: Option<String>,
}

impl ActionResponse {
    /// Constructs an [`ActionResponse`], based on the response from the [`ticket_to_ride::manager::Manager`].
    pub(crate) fn new(manager_action_result: ManagerActionResult) -> Self {
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
