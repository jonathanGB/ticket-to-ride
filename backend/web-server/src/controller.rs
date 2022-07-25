use crate::authenticator::{Authenticator, Identifier};
use crate::request_types::*;
use crate::response_types::*;

use dashmap::{mapref::one::Ref, mapref::one::RefMut, DashMap};
use rocket::http::{uri::Origin, CookieJar};
use rocket::serde::json::Json;
use uuid::Uuid;

use ticket_to_ride::manager::Manager;

pub type GameIdManagerMapping = DashMap<Uuid, Manager>;

/// Main entrypoint of read-only requests to the server, after routing.
///
/// The controller is in charge of most of the business logic on the server.
/// It delegates specific complexity to the [`Authenticator`], and the [`Manager`].
///
/// It is different from [`WriteController`] in that it has a shared reference to the [`Manager`],
/// rather than a mutable reference.
pub struct ReadController<'a> {
    game_id_and_manager: Ref<'a, Uuid, Manager>,
    cookies: &'a CookieJar<'a>,
}

impl<'a> ReadController<'a> {
    pub fn new(
        game_id_and_manager: Ref<'a, Uuid, Manager>,
        cookies: &'a CookieJar<'a>,
    ) -> ReadController<'a> {
        Self {
            game_id_and_manager,
            cookies,
        }
    }

    fn game_id(&self) -> &Uuid {
        self.game_id_and_manager.key()
    }

    fn manager(&self) -> &Manager {
        self.game_id_and_manager.value()
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
    cookies: &'a CookieJar<'a>,
}

impl<'a> WriteController<'a> {
    pub fn new(
        game_id_and_manager: RefMut<'a, Uuid, Manager>,
        cookies: &'a CookieJar<'a>,
    ) -> WriteController<'a> {
        Self {
            game_id_and_manager,
            cookies,
        }
    }

    fn game_id(&self) -> &Uuid {
        self.game_id_and_manager.key()
    }

    fn manager(&mut self) -> &mut Manager {
        self.game_id_and_manager.value_mut()
    }

    pub fn create_game(state: &DashMap<Uuid, Manager>) -> Uuid {
        let game_id = Uuid::new_v4();

        state.insert(game_id, Manager::new());

        game_id
    }

    pub fn load_game(&mut self, origin: &Origin) -> bool {
        if let Some(player_id) =
            Authenticator::validate_and_get_player_id(self.cookies, self.game_id())
        {
            println!(
                "Loaded game with ID = {}, player_id is = {}",
                self.game_id(),
                player_id
            );

            return true;
        }

        let player_id = match self.manager().add_player() {
            Some(player_id) => player_id,
            None => return false,
        };

        Authenticator::authenticate(
            self.cookies,
            &origin.path(),
            Identifier::new(self.game_id().clone(), player_id),
        );

        println!(
            "Loaded game with ID = {}, now authenticated as {}.",
            self.game_id(),
            player_id
        );

        true
    }

    pub fn change_player_name(
        &mut self,
        change_name_request: ChangeNameRequest,
    ) -> ChangeNameResponse {
        match Authenticator::validate_and_get_player_id(self.cookies, self.game_id()) {
            Some(player_id) => {
                if self
                    .manager()
                    .change_player_name(player_id, change_name_request.new_name)
                {
                    ChangeNameResponse::Success
                } else {
                    ChangeNameResponse::AlreadyUsed
                }
            }
            None => ChangeNameResponse::Unauthorized,
        }
    }
}
