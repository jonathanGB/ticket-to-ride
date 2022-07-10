use parse_display::{Display, FromStr};
use rocket::http::{uri::Path, Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use uuid::Uuid;

// Authenticates incoming HTTP requests to their corresponding player.
//
// Implements `FromRequest`, so it can be used as a request guard.
// In fact, as there is no public constructor, an `Authenticator` can only be
// instantiated via a request guard.

/// Identifier of the request, which is contained in a private cookie sent by the players.
/// We make it contain both the `game_id` and the `player_id`, even though only the `player_id`
/// is strictly necessary, in order to prevent players from acting as another player by copying
/// the cookie to another game.
///
/// As the cookie hashes and encrypts the identifier, the mangling of the `game_id` results in
/// different cookies across games.
#[derive(Debug, Display, FromStr)]
#[display("{game_id}/{player_id}")]
pub struct Identifier {
    game_id: Uuid,
    player_id: usize,
}

impl Identifier {
    pub fn new(game_id: Uuid, player_id: usize) -> Self {
        Identifier { game_id, player_id }
    }
}
pub struct Authenticator {
    player_id: usize,
}

#[derive(Debug)]
pub enum AuthenticatorError {
    Unauthenticated,
    InvalidPlayerId,
    InvalidUrl,
}

const COOKIE_IDENTIFIER_NAME: &str = "identifier";

impl Authenticator {
    // Validates the given request.
    // If the request is authenticated, it returns the player's ID.
    // Otherwise, it returns None.
    pub fn validate_and_get_player_id(cookies: &CookieJar, game_id: &Uuid) -> Option<usize> {
        match Self::authentication_outcome(cookies, game_id) {
            Outcome::Success(authenticator) => Some(authenticator.player_id),
            _ => None,
        }
    }

    // Given the player ID and the game path, writes a new private cookie to authenticate
    // subsequent requests coming from this browser.
    // Considering that we set the path in the cookie, this means that the same browser
    // will not re-use the same cookie when starting a new game.
    pub fn authenticate(cookies: &CookieJar, game_path: &Path, identifier: Identifier) {
        cookies.add_private(
            Cookie::build(COOKIE_IDENTIFIER_NAME, identifier.to_string())
                .path(game_path.to_string())
                .finish(),
        );
    }

    pub fn player_id(&self) -> usize {
        self.player_id
    }

    fn authentication_outcome(
        cookies: &CookieJar,
        game_id: &Uuid,
    ) -> Outcome<Self, AuthenticatorError> {
        if let Some(identifier_cookie) = cookies.get_private(COOKIE_IDENTIFIER_NAME) {
            match identifier_cookie.value().parse::<Identifier>() {
                Ok(identifier) if &identifier.game_id == game_id => {
                    Outcome::Success(Authenticator {
                        player_id: identifier.player_id,
                    })
                }
                _ => Outcome::Failure((Status::Unauthorized, AuthenticatorError::InvalidPlayerId)),
            }
        } else {
            println!("No cookie");
            Outcome::Failure((Status::Unauthorized, AuthenticatorError::Unauthenticated))
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Authenticator {
    type Error = AuthenticatorError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // The path should be `/game/<game_id>`.
        match req.param::<Uuid>(1) {
            Some(Ok(game_id)) => Self::authentication_outcome(req.cookies(), &game_id),
            _ => Outcome::Failure((Status::NotFound, AuthenticatorError::InvalidUrl)),
        }
    }
}
