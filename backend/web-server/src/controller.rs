//! The middleman between the [`crate::router`] handlers, [`Authenticator`], and [`ticket_to_ride::manager::Manager`].

use crate::authenticator::{Authenticator, AuthenticatorError, Identifier};
use crate::request_types::*;
use crate::response_types::*;

use dashmap::{mapref::one::Ref, mapref::one::RefMut, DashMap};
use rocket::http::{uri::Origin, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use uuid::Uuid;

use ticket_to_ride::manager::{GameState, Manager};

/// Maps a game ID to a manager in a shared concurrent hash map.
pub type GameIdManagerMapping = DashMap<Uuid, Manager>;

/// Types of error when creating a controller.
#[derive(Debug)]
pub enum ControllerGuardError {
    InvalidGameId,
    /// Should never occur!
    ///
    /// This only happens if we try to guard a controller for a request that does not
    /// guard against the [`GameIdManagerMapping`] state.
    StateNotFound,
    AuthenticatorFailed(AuthenticatorError),
}

/// Main entrypoint of read-only requests to the server, after routing.
///
/// The controller is in charge of most of the business logic on the server, coming from the [`crate::router`] handlers.
/// It delegates specific complexity to the [`Authenticator`], and the [`Manager`].
///
/// It is different from [`WriteController`] in that it has a shared reference to the [`Manager`],
/// rather than a mutable reference.
///
/// Implements [`rocket::request::FromRequest`], so it can be used as a request guard.
/// In fact, as there is no public constructor, it can only be instantiated via a request guard.
pub struct ReadController<'a> {
    /// Shared reference to the game ID, and to the [`Manager`] of that game.
    game_id_and_manager: Ref<'a, Uuid, Manager>,
    /// The player initiating the read-only request.
    player_id: usize,
}

impl<'a> ReadController<'a> {
    #[inline]
    fn manager(&self) -> &Manager {
        self.game_id_and_manager.value()
    }

    #[inline]
    pub(crate) fn get_game_state(&self) -> GameState {
        self.manager().get_state(self.player_id)
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for ReadController<'a> {
    type Error = ControllerGuardError;

    #[inline]
    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        Self::controller_from_request(request).await
    }
}

impl<'a> Controller<'a> for ReadController<'a> {
    fn controller_from_request_internal(
        game_id_manager_mapping: &'a State<GameIdManagerMapping>,
        authenticator: Authenticator,
    ) -> Outcome<Self, ControllerGuardError> {
        match game_id_manager_mapping.get(authenticator.game_id()) {
            Some(game_id_and_manager) => Outcome::Success(Self {
                game_id_and_manager,
                player_id: authenticator.player_id(),
            }),
            None => Outcome::Failure((Status::NotFound, ControllerGuardError::InvalidGameId)),
        }
    }
}

/// Main entrypoint of write requests to the server, after routing.
///
/// The controller is in charge of most of the business logic on the server.
/// It delegates specific complexity to the [`Authenticator`], and the [`Manager`].
///
/// It is different from [`ReadController`] in that it has a mutable reference to the [`Manager`],
/// rather than a shared reference.
///
/// Implements [`rocket::request::FromRequest`], so it can be used as a request guard.
/// In fact, as there is no public constructor, it can only be instantiated via a request guard.
pub struct WriteController<'a> {
    /// Mutable reference to the game ID, and to the [`Manager`] of that game.
    game_id_and_manager: RefMut<'a, Uuid, Manager>,
    /// The player initiating the write request.
    player_id: usize,
}

impl<'a> WriteController<'a> {
    #[inline]
    fn manager(&mut self) -> &mut Manager {
        self.game_id_and_manager.value_mut()
    }

    pub(crate) fn create_game(state: &DashMap<Uuid, Manager>) -> Uuid {
        let game_id = Uuid::new_v4();

        state.insert(game_id, Manager::new());

        game_id
    }

    pub(crate) fn load_game(
        mut manager: RefMut<'a, Uuid, Manager>,
        cookies: &CookieJar,
        origin: &Origin,
    ) -> bool {
        let game_id = manager.key().clone();
        let manager = manager.value_mut();

        if Authenticator::validate_and_get_player_id(cookies, game_id).is_some() {
            return true;
        }

        let player_id = match manager.add_player() {
            Some(player_id) => player_id,
            None => return false,
        };

        Authenticator::authenticate(cookies, &origin.path(), Identifier::new(game_id, player_id));
        true
    }

    #[inline]
    pub(crate) fn change_player_name(
        &mut self,
        change_name_request: ChangeNameRequest,
    ) -> ActionResponse {
        let player_id = self.player_id;

        ActionResponse::new(
            self.manager()
                .change_player_name(player_id, change_name_request.new_name),
        )
    }

    #[inline]
    pub(crate) fn change_player_color(
        &mut self,
        change_color_request: ChangeColorRequest,
    ) -> ActionResponse {
        let player_id = self.player_id;

        ActionResponse::new(
            self.manager()
                .change_player_color(player_id, change_color_request.new_color),
        )
    }

    #[inline]
    pub(crate) fn set_player_ready(
        &mut self,
        set_player_ready_request: SetPlayerReadyRequest,
    ) -> ActionResponse {
        let player_id = self.player_id;

        ActionResponse::new(
            self.manager()
                .set_ready(player_id, set_player_ready_request.is_ready),
        )
    }

    #[inline]
    pub(crate) fn select_destination_cards(
        &mut self,
        select_destination_cards_request: SelectDestinationCardsRequest,
    ) -> ActionResponse {
        let player_id = self.player_id;

        ActionResponse::new(self.manager().select_destination_cards(
            player_id,
            select_destination_cards_request.destination_cards_decisions,
        ))
    }

    #[inline]
    pub(crate) fn draw_destination_cards(&mut self) -> ActionResponse {
        let player_id = self.player_id;

        ActionResponse::new(self.manager().draw_destination_cards(player_id))
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for WriteController<'a> {
    type Error = ControllerGuardError;

    #[inline]
    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        Self::controller_from_request(request).await
    }
}

impl<'a> Controller<'a> for WriteController<'a> {
    fn controller_from_request_internal(
        game_id_manager_mapping: &'a State<GameIdManagerMapping>,
        authenticator: Authenticator,
    ) -> Outcome<Self, ControllerGuardError> {
        match game_id_manager_mapping.get_mut(authenticator.game_id()) {
            Some(game_id_and_manager) => Outcome::Success(Self {
                game_id_and_manager,
                player_id: authenticator.player_id(),
            }),
            None => Outcome::Failure((Status::NotFound, ControllerGuardError::InvalidGameId)),
        }
    }
}

#[rocket::async_trait]
trait Controller<'a>: Sized {
    fn controller_from_request_internal(
        game_id_manager_mapping: &'a State<GameIdManagerMapping>,
        authenticator: Authenticator,
    ) -> Outcome<Self, ControllerGuardError>;

    async fn controller_from_request(
        request: &'a Request<'_>,
    ) -> Outcome<Self, ControllerGuardError> {
        match request.guard::<Authenticator>().await {
            Outcome::Success(authenticator) => {
                match request.guard::<&'a State<GameIdManagerMapping>>().await {
                    Outcome::Success(game_id_manager_mapping) => {
                        Self::controller_from_request_internal(
                            game_id_manager_mapping,
                            authenticator,
                        )
                    }
                    _ => {
                        eprintln!(
                            "No `State<GameIdManagerMapping>` is set for the given handler.
                            Consider adding `ReadController` or `WriteController` as a request guard."
                        );

                        Outcome::Failure((
                            Status::InternalServerError,
                            ControllerGuardError::StateNotFound,
                        ))
                    }
                }
            }
            Outcome::Failure((status, e)) => {
                Outcome::Failure((status, ControllerGuardError::AuthenticatorFailed(e)))
            }
            Outcome::Forward(_) => unreachable!("The authenticator should never forward."),
        }
    }
}
