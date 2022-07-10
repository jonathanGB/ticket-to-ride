use crate::authenticator::Authenticator;
use dashmap::{mapref::one::RefMut, DashMap};
use rocket::http::{uri::Origin, CookieJar};
use uuid::Uuid;

// TODO: Eventually, this should be exported from the `ticket-to-ride` crate.
pub type GameState = Vec<usize>;

// Main entrypoint of incoming requests to the server, after routing.
// The controller is in charge of most of the business logic on the server.
// It delegates specific complexity to the Authenticator, and the GameManager.
pub struct Controller<'a> {
    game_id_and_state: RefMut<'a, Uuid, GameState>,
}

impl<'a> Controller<'a> {
    pub fn new(game_id_and_state: RefMut<'a, Uuid, GameState>) -> Controller<'a> {
        Controller { game_id_and_state }
    }

    fn game_id(&self) -> &Uuid {
        self.game_id_and_state.key()
    }

    fn game_state(&self) -> &GameState {
        self.game_id_and_state.value()
    }

    fn game_state_mut(&mut self) -> &mut GameState {
        self.game_id_and_state.value_mut()
    }

    pub fn create_game(state: &DashMap<Uuid, GameState>) -> Uuid {
        let game_id = Uuid::new_v4();

        state.insert(game_id, GameState::new());

        game_id
    }

    pub fn load_game(&mut self, cookies: &CookieJar, origin: &Origin) -> bool {
        if let Some(player_id) = Authenticator::validate_and_get_player_id(cookies) {
            println!(
                "Loaded game with ID = {}, player_id is = {}",
                self.game_id(),
                player_id
            );

            return true;
        }

        let num_players = self.game_state().len();
        if num_players == 5 {
            return false;
        }

        let player_id = num_players;
        self.game_state_mut().push(player_id);

        Authenticator::authenticate(cookies, &origin.path(), player_id);
        println!(
            "Loaded game with ID = {}, now authenticated as {}.",
            self.game_id(),
            player_id
        );

        true
    }
}
