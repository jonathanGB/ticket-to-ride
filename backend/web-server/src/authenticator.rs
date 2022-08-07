//! All things related to authenticating incoming HTTP requests.

use parse_display::{Display, FromStr};
use rocket::http::{uri::Path, Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use uuid::Uuid;

/// The cookie's name for the [`Identifier`], which we authenticate against.
const COOKIE_IDENTIFIER_NAME: &str = "identifier";

/// Identifier of the request, which is contained in a private cookie sent by the players.
///
/// We make it contain both the `game_id` and the `player_id`, even though only the `player_id`
/// is strictly necessary, in order to prevent players from acting as another player by copying
/// the cookie to another game.
///
/// As the cookie hashes and encrypts the identifier, the mangling of the `game_id` results in
/// different cookies across games.
#[derive(Clone, Copy, Debug, Display, FromStr, PartialEq)]
#[display("{game_id}/{player_id}")]
pub(crate) struct Identifier {
    /// The game ID attached to the request.
    game_id: Uuid,
    /// The player ID attached to the request.
    player_id: usize,
}

impl Identifier {
    /// Constructs an [`Identifier`].
    pub fn new(game_id: Uuid, player_id: usize) -> Self {
        Identifier { game_id, player_id }
    }
}

/// Types of error when authenticating a request.
#[derive(Debug, PartialEq)]
pub enum AuthenticatorError {
    GameIdMismatch,
    InvalidUrl,
    Unauthenticated,
    UnparsableCookie,
}

/// Authenticates incoming HTTP requests, encapsulating the corresponding player and game IDs.
///
/// Implements [`rocket::request::FromRequest`], so it can be used as a request guard.
/// In fact, as there is no public constructor, an [`Authenticator`] can only be
/// instantiated via a request guard.
#[derive(Debug, PartialEq)]
pub(crate) struct Authenticator {
    identifier: Identifier,
}

impl Authenticator {
    /// Validates the given request.
    ///
    /// If the request is authenticated, it returns the player's ID.
    ///
    /// Otherwise, it returns `None`.
    pub(crate) fn validate_and_get_player_id(cookies: &CookieJar, game_id: Uuid) -> Option<usize> {
        match Self::authentication_outcome(cookies, game_id) {
            Outcome::Success(authenticator) => Some(authenticator.player_id()),
            _ => None,
        }
    }

    /// Given an [`Identifier`] and the game path, writes a new private cookie to authenticate
    /// subsequent requests coming from this browser.
    ///
    /// Considering that we set the path in the cookie, this means that the same browser
    /// will not re-use the same cookie when starting a new game.
    pub(crate) fn authenticate(cookies: &CookieJar, game_path: &Path, identifier: Identifier) {
        cookies.add_private(
            Cookie::build(COOKIE_IDENTIFIER_NAME, identifier.to_string())
                .path(game_path.to_string())
                .finish(),
        );
    }

    /// Returns the authenticated player ID.
    pub(crate) fn player_id(&self) -> usize {
        self.identifier.player_id
    }

    /// Returns the authenticated game ID.
    pub(crate) fn game_id(&self) -> &Uuid {
        &self.identifier.game_id
    }

    fn authentication_outcome(
        cookies: &CookieJar,
        game_id: Uuid,
    ) -> Outcome<Self, AuthenticatorError> {
        if let Some(identifier_cookie) = cookies.get_private(COOKIE_IDENTIFIER_NAME) {
            match identifier_cookie.value().parse::<Identifier>() {
                Ok(identifier) if &identifier.game_id == &game_id => {
                    Outcome::Success(Authenticator { identifier })
                }
                Ok(_) => {
                    Outcome::Failure((Status::Unauthorized, AuthenticatorError::GameIdMismatch))
                }
                _ => Outcome::Failure((Status::Unauthorized, AuthenticatorError::UnparsableCookie)),
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
        // The path should be `/game/<game_id>`.
        if !req.uri().path().starts_with("/game/") {
            return Outcome::Failure((Status::NotFound, AuthenticatorError::InvalidUrl));
        }

        match req.param::<Uuid>(1) {
            Some(Ok(game_id)) => Self::authentication_outcome(req.cookies(), game_id),
            _ => Outcome::Failure((Status::NotFound, AuthenticatorError::InvalidUrl)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::local::blocking::Client;
    type AsyncClient = rocket::local::asynchronous::Client;

    fn new_authenticator() -> Authenticator {
        Authenticator {
            identifier: new_identifier(),
        }
    }

    fn new_identifier() -> Identifier {
        Identifier {
            player_id: 6,
            game_id: Uuid::new_v4(),
        }
    }

    // Tests for `Identifier`.

    #[test]
    fn identifier_display() {
        let identifier = new_identifier();

        assert_eq!(
            identifier.to_string(),
            format!("{}/{}", identifier.game_id, identifier.player_id)
        );
    }

    // Tests for `Authenticator`.

    #[test]
    fn authenticator_accessors() {
        let authenticator = new_authenticator();
        assert_eq!(authenticator.game_id(), &authenticator.identifier.game_id);
        assert_eq!(
            authenticator.player_id(),
            authenticator.identifier.player_id
        );
    }

    #[test]
    fn authenticator_authentication_outcome_no_cookies() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).expect("valid rocket");
        let Identifier { game_id, .. } = new_identifier();

        assert_eq!(
            Authenticator::authentication_outcome(&client.cookies(), game_id),
            Outcome::Failure((Status::Unauthorized, AuthenticatorError::Unauthenticated))
        );
        assert!(Authenticator::validate_and_get_player_id(&client.cookies(), game_id).is_none());
    }

    #[test]
    fn authenticator_authentication_outcome_invalid_cookie() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).expect("valid rocket");
        let req = client.get("/");
        let Identifier { game_id, .. } = new_identifier();

        let req = req.private_cookie(Cookie::new(COOKIE_IDENTIFIER_NAME, "invalid_cookie"));
        let cookies = req.inner().cookies();

        let outcome = Authenticator::authentication_outcome(cookies, game_id);
        assert_eq!(
            outcome.failed(),
            Some((Status::Unauthorized, AuthenticatorError::UnparsableCookie))
        );
        assert!(Authenticator::validate_and_get_player_id(&client.cookies(), game_id).is_none());
    }

    #[test]
    fn authenticator_authentication_outcome_wrong_game_id() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).expect("valid rocket");
        let req = client.get("/");
        let identifier = new_identifier();
        let wrong_game_id = Uuid::new_v4();

        let req = req.private_cookie(Cookie::new(COOKIE_IDENTIFIER_NAME, identifier.to_string()));
        let cookies = req.inner().cookies();

        let outcome = Authenticator::authentication_outcome(cookies, wrong_game_id);
        assert_eq!(
            outcome.failed(),
            Some((Status::Unauthorized, AuthenticatorError::GameIdMismatch))
        );
        assert!(Authenticator::validate_and_get_player_id(cookies, wrong_game_id).is_none());
    }

    #[test]
    fn authenticator_authentication_outcome_success() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).expect("valid rocket");
        let req = client.get("/");
        let identifier = new_identifier();

        let req = req.private_cookie(Cookie::new(COOKIE_IDENTIFIER_NAME, identifier.to_string()));
        let cookies = req.inner().cookies();

        let outcome = Authenticator::authentication_outcome(cookies, identifier.game_id);
        assert_eq!(outcome.succeeded(), Some(Authenticator { identifier }));
        assert_eq!(
            Authenticator::validate_and_get_player_id(cookies, identifier.game_id),
            Some(identifier.player_id)
        );
    }

    #[test]
    fn authenticator_authenticate() {
        let rocket = rocket::build();
        let client = Client::tracked(rocket).expect("valid rocket");
        let identifier = new_identifier();
        let path = format!("/game/{}", identifier.game_id);
        let req = client.get(&path);

        assert_eq!(req.uri().path().as_str(), path);

        let cookies = req.inner().cookies();

        let authenticated_cookie = cookies.get_pending(COOKIE_IDENTIFIER_NAME);
        assert!(authenticated_cookie.is_none());

        Authenticator::authenticate(cookies, &req.uri().path(), identifier);

        let authenticated_cookie = cookies.get_pending(COOKIE_IDENTIFIER_NAME);
        assert!(authenticated_cookie.is_some());
        let authenticated_cookie = authenticated_cookie.unwrap();

        assert_eq!(authenticated_cookie.value(), identifier.to_string());
        assert_eq!(authenticated_cookie.path(), Some(path.as_str()));
    }

    #[rocket::async_test]
    async fn authenticator_from_request_invalid_path() {
        let rocket = rocket::build();
        let client = AsyncClient::tracked(rocket).await.expect("valid rocket");
        let identifier = new_identifier();
        let path = format!("/wrong_path/{}", identifier.game_id);
        let req = client.get(&path);

        let outcome = Authenticator::from_request(req.inner()).await;
        assert_eq!(
            outcome.failed(),
            Some((Status::NotFound, AuthenticatorError::InvalidUrl))
        );
    }

    #[rocket::async_test]
    async fn authenticator_from_request_invalid_uuid_in_path() {
        let rocket = rocket::build();
        let client = AsyncClient::tracked(rocket).await.expect("valid rocket");
        let path = String::from("/game/abc-123");
        let req = client.get(&path);

        let outcome = Authenticator::from_request(req.inner()).await;
        assert_eq!(
            outcome.failed(),
            Some((Status::NotFound, AuthenticatorError::InvalidUrl))
        );
    }

    #[rocket::async_test]
    async fn authenticator_from_request_authentication_failure() {
        let rocket = rocket::build();
        let client = AsyncClient::tracked(rocket).await.expect("valid rocket");
        let identifier = new_identifier();
        let path = format!("/game/{}", identifier.game_id);
        let req = client.get(&path);

        let outcome = Authenticator::from_request(req.inner()).await;
        assert_eq!(
            outcome.failed(),
            Some((Status::Unauthorized, AuthenticatorError::Unauthenticated))
        );
    }

    #[rocket::async_test]
    async fn authenticator_from_request_authentication_success() {
        let rocket = rocket::build();
        let client = AsyncClient::tracked(rocket).await.expect("valid rocket");
        let identifier = new_identifier();
        let path = format!("/game/{}", identifier.game_id);
        let req = client.get(&path);

        let req = req.private_cookie(Cookie::new(COOKIE_IDENTIFIER_NAME, identifier.to_string()));

        let outcome = Authenticator::from_request(req.inner()).await;
        assert_eq!(outcome.succeeded(), Some(Authenticator { identifier }));
    }
}
