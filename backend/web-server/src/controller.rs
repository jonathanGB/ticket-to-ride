use crate::authenticator::Authenticator;
use rocket::http::{uri::Origin, CookieJar};
use uuid::Uuid;

// Main entrypoint of incoming requests to the server, after routing.
// The controller is in charge of most of the business logic on the server.
// It delegates specific complexity to the Authenticator, the Model, and the GameManager.
pub struct Controller {
  game_id: Uuid,
}

impl Controller {
  pub fn new(game_id: Uuid) -> Self {
    Controller { game_id }
  }

  pub fn create_game() -> Uuid {
    let game_id = Uuid::new_v4();
    // TODO: Create a game state in the database.

    game_id
  }

  pub fn load_game(&self, cookies: &CookieJar, origin: &Origin) -> String {
    if let Some(player_id) = Authenticator::validate_and_get_player_id(cookies) {
      format!(
        "Loaded game with ID = {}, player_id is = {}",
        self.game_id, player_id
      )
    } else {
      // TODO: generate a real player ID, not zero.
      Authenticator::authenticate(cookies, &origin.path(), 0);
      format!(
        "Loaded game with ID = {}, but not authenticated yet.",
        self.game_id
      )
    }
  }
}
