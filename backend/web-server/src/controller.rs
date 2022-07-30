use crate::authenticator::{Authenticator, AuthenticatorError, Identifier};
use crate::request_types::*;
use crate::response_types::*;

use dashmap::{mapref::one::Ref, mapref::one::RefMut, DashMap};
use rocket::http::{uri::Origin, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use uuid::Uuid;

use ticket_to_ride::manager::{GameState, Manager};

pub type GameIdManagerMapping = DashMap<Uuid, Manager>;

#[derive(Debug)]
pub enum ControllerGuardError {
    InvalidGameId,
    StateNotFound,
    AuthenticatorFailed(AuthenticatorError),
}

/// Main entrypoint of read-only requests to the server, after routing.
///
/// The controller is in charge of most of the business logic on the server.
/// It delegates specific complexity to the [`Authenticator`], and the [`Manager`].
///
/// It is different from [`WriteController`] in that it has a shared reference to the [`Manager`],
/// rather than a mutable reference.
pub struct ReadController<'a> {
    game_id_and_manager: Ref<'a, Uuid, Manager>,
    player_id: usize,
}

impl<'a> ReadController<'a> {
    pub fn new(
        game_id_and_manager: Ref<'a, Uuid, Manager>,
        player_id: usize,
    ) -> ReadController<'a> {
        Self {
            game_id_and_manager,
            player_id,
        }
    }

    fn manager(&self) -> &Manager {
        self.game_id_and_manager.value()
    }

    pub fn get_game_state(&self) -> GameState {
        self.manager().get_state(self.player_id)
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for ReadController<'a> {
    type Error = ControllerGuardError;

    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        match request.guard::<Authenticator>().await {
            Outcome::Success(authenticator) => {
                match request.guard::<&'a State<GameIdManagerMapping>>().await {
                    Outcome::Success(game_id_manager_mapping) => {
                        match game_id_manager_mapping.get(authenticator.game_id()) {
                            Some(game_id_and_manager) => Outcome::Success(Self::new(
                                game_id_and_manager,
                                authenticator.player_id(),
                            )),
                            None => Outcome::Failure((
                                Status::NotFound,
                                ControllerGuardError::InvalidGameId,
                            )),
                        }
                    }
                    _ => Outcome::Failure((
                        Status::InternalServerError,
                        ControllerGuardError::StateNotFound,
                    )),
                }
            }
            Outcome::Failure((status, e)) => {
                Outcome::Failure((status, ControllerGuardError::AuthenticatorFailed(e)))
            }
            Outcome::Forward(_) => unreachable!(),
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
pub struct WriteController<'a> {
    game_id_and_manager: RefMut<'a, Uuid, Manager>,
    player_id: usize,
}

impl<'a> WriteController<'a> {
    pub fn new(
        game_id_and_manager: RefMut<'a, Uuid, Manager>,
        player_id: usize,
    ) -> WriteController<'a> {
        Self {
            game_id_and_manager,
            player_id,
        }
    }

    fn manager(&mut self) -> &mut Manager {
        self.game_id_and_manager.value_mut()
    }

    pub fn create_game(state: &DashMap<Uuid, Manager>) -> Uuid {
        let game_id = Uuid::new_v4();

        state.insert(game_id, Manager::new());

        game_id
    }

    pub fn load_game(
        mut manager: RefMut<'a, Uuid, Manager>,
        cookies: &CookieJar,
        origin: &Origin,
    ) -> bool {
        let game_id = manager.key().clone();
        let manager = manager.value_mut();

        if let Some(player_id) = Authenticator::validate_and_get_player_id(cookies, game_id) {
            println!(
                "Loaded game with ID = {}, player_id is = {}",
                &game_id, player_id
            );

            return true;
        }

        let player_id = match manager.add_player() {
            Some(player_id) => player_id,
            None => return false,
        };

        Authenticator::authenticate(cookies, &origin.path(), Identifier::new(game_id, player_id));

        println!(
            "Loaded game with ID = {}, now authenticated as {}.",
            &game_id, player_id
        );

        true
    }

    pub fn change_player_name(&mut self, change_name_request: ChangeNameRequest) -> ActionResponse {
        let player_id = self.player_id;

        match self
            .manager()
            .change_player_name(player_id, change_name_request.new_name)
        {
            Ok(_) => ActionResponse::new_success(),
            Err(e) => ActionResponse::new_failure(e),
        }
    }

    pub fn change_player_color(
        &mut self,
        change_color_request: ChangeColorRequest,
    ) -> ActionResponse {
        let player_id = self.player_id;

        match self
            .manager()
            .change_player_color(player_id, change_color_request.new_color)
        {
            Ok(_) => ActionResponse::new_success(),
            Err(e) => ActionResponse::new_failure(e),
        }
    }

    pub fn set_player_ready(
        &mut self,
        set_player_ready_request: SetPlayerReadyRequest,
    ) -> ActionResponse {
        let player_id = self.player_id;

        match self
            .manager()
            .set_ready(player_id, set_player_ready_request.is_ready)
        {
            Ok(_) => ActionResponse::new_success(),
            Err(e) => ActionResponse::new_failure(e),
        }
    }
}

#[rocket::async_trait]
impl<'a> FromRequest<'a> for WriteController<'a> {
    type Error = ControllerGuardError;

    async fn from_request(request: &'a Request<'_>) -> Outcome<Self, Self::Error> {
        match request.guard::<Authenticator>().await {
            Outcome::Success(authenticator) => {
                match request.guard::<&'a State<GameIdManagerMapping>>().await {
                    Outcome::Success(game_id_manager_mapping) => {
                        match game_id_manager_mapping.get_mut(authenticator.game_id()) {
                            Some(game_id_and_manager) => Outcome::Success(Self::new(
                                game_id_and_manager,
                                authenticator.player_id(),
                            )),
                            None => Outcome::Failure((
                                Status::NotFound,
                                ControllerGuardError::InvalidGameId,
                            )),
                        }
                    }
                    _ => Outcome::Failure((
                        Status::InternalServerError,
                        ControllerGuardError::StateNotFound,
                    )),
                }
            }
            Outcome::Failure((status, e)) => {
                Outcome::Failure((status, ControllerGuardError::AuthenticatorFailed(e)))
            }
            Outcome::Forward(_) => unreachable!("The authenticator should never forward."),
        }
    }
}
