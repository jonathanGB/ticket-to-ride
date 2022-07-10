use rocket::http::{uri::Path, Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};

// Authenticates incoming HTTP requests to their corresponding player.
//
// Implements `FromRequest`, so it can be used as a request guard.
// In fact, as there is no public constructor, an `Authenticator` can only be
// instantiated via a request guard.
pub struct Authenticator {
    player_id: usize,
}

#[derive(Debug)]
pub enum AuthenticatorError {
    Unauthenticated,
    InvalidPlayerId,
}

const COOKIE_PLAYER_ID_NAME: &str = "player_id";

impl Authenticator {
    // Validates the given request.
    // If the request is authenticated, it returns the player's ID.
    // Otherwise, it returns None.
    pub fn validate_and_get_player_id(cookies: &CookieJar) -> Option<usize> {
        match Self::authentication_outcome(cookies) {
            Outcome::Success(authenticator) => Some(authenticator.player_id),
            _ => None,
        }
    }

    // Given the player ID and the game path, writes a new private cookie to authenticate
    // subsequent requests coming from this browser.
    // Considering that we set the path in the cookie, this means that the same browser
    // will not re-use the same cookie when starting a new game.
    pub fn authenticate(cookies: &CookieJar, game_path: &Path, player_id: usize) {
        cookies.add_private(
            Cookie::build(COOKIE_PLAYER_ID_NAME, player_id.to_string())
                .path(game_path.to_string())
                .finish(),
        );
    }

    pub fn player_id(&self) -> usize {
        self.player_id
    }

    fn authentication_outcome(cookies: &CookieJar) -> Outcome<Self, AuthenticatorError> {
        if let Some(player_id_cookie) = cookies.get_private(COOKIE_PLAYER_ID_NAME) {
            match player_id_cookie.value().parse::<usize>() {
                Ok(player_id) => Outcome::Success(Authenticator { player_id }),
                Err(_) => {
                    Outcome::Failure((Status::Unauthorized, AuthenticatorError::InvalidPlayerId))
                }
            }
        } else {
            Outcome::Failure((Status::Unauthorized, AuthenticatorError::Unauthenticated))
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authenticator {
    type Error = AuthenticatorError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        Self::authentication_outcome(req.cookies())
    }
}
