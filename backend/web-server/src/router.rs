//! All the routes handled by the web server.
//!
//! Most of the handlers' logic is handled by the [`ReadController`] and [`WriteController`],
//! which themselves delegate to the [`crate::authenticator::Authenticator`] and to the
//! [`ticket_to_ride::manager::Manager`].

use crate::controller::{GameIdManagerMapping, ReadController, WriteController};
use crate::request_types::*;
use crate::response_types::*;

use rocket::response::content::RawJson;
use rocket::serde::json::Json;
use rocket::{
    fs::NamedFile,
    http::{uri::Origin, CookieJar},
    response::Redirect,
    serde::uuid::Uuid,
    State,
};
use std::path::Path;

/// Path to the frontend build directory.
/// This directory contains the frontend app that needs to be served to clients.
pub(crate) const BUILD_FILES_PATH: &str = "../../frontend/build";

#[inline]
/// Helper to redirect to [`root()`].
fn redirect_to_root() -> Redirect {
    Redirect::to(uri!(root()))
}

/// Serves the frontend app.
#[get("/")]
pub async fn root() -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new(BUILD_FILES_PATH).join("index.html")).await
}

/// Redirects permanently to [`root()`].
#[get("/index.html")]
pub fn index() -> Redirect {
    Redirect::permanent(uri!(root()))
}

/// Serves the robots file, for crawlers.
#[get("/robots.txt")]
pub async fn robots() -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new(BUILD_FILES_PATH).join("robots.txt")).await
}

/// Creates a game, and redirects to [`load_game()`].
#[post("/create")]
pub fn create_game(state: &State<GameIdManagerMapping>) -> Redirect {
    let game_id = WriteController::create_game(state);

    Redirect::to(uri!(load_game(game_id)))
}

/// Authenticates the player, and serves the frontend app.
///
/// If no games are found for that ID, redirects to [`root()`].
///
/// Otherwise, validates whether the player is authenticated for that game:
///   * If they are, then simply serves the frontend app.
///   * If they are not, but we can add a player (see [`ticket_to_ride::manager::Manager::add_player`]),
///     then we add the player to the game, store a cookie, and serve the frontend app.
///   * If they are not and we can't add a player, redirects to [`root()`].
#[get("/game/<game_id>")]
pub async fn load_game(
    game_id: Uuid,
    cookies: &CookieJar<'_>,
    origin: &Origin<'_>,
    state: &State<GameIdManagerMapping>,
) -> Result<NamedFile, LoadGameError> {
    match state.get_mut(&game_id) {
        Some(game_id_and_state) => {
            if !WriteController::load_game(game_id_and_state, cookies, origin) {
                return Err(LoadGameError::Unauthorized(redirect_to_root()));
            }

            match NamedFile::open(Path::new(BUILD_FILES_PATH).join("index.html")).await {
                Ok(file) => Ok(file),
                Err(e) => Err(LoadGameError::NoFile(e)),
            }
        }
        None => Err(LoadGameError::NoGame(redirect_to_root())),
    }
}

/// Tries to change the player's name. The player must be authenticated to do so.
///
/// More details in [`ticket_to_ride::manager::Manager::change_player_name`].
#[put(
    "/game/<_>/player/name",
    format = "json",
    data = "<change_name_request>"
)]
pub fn change_player_name(
    mut write_controller: WriteController,
    change_name_request: Json<ChangeNameRequest>,
) -> Json<ActionResponse> {
    Json(write_controller.change_player_name(change_name_request.into_inner()))
}

/// Tries to change the player's color. The player must be authenticated to do so.
///
/// More details in [`ticket_to_ride::manager::Manager::change_player_color`].
#[put(
    "/game/<_>/player/color",
    format = "json",
    data = "<change_color_request>"
)]
pub fn change_player_color(
    mut write_controller: WriteController,
    change_color_request: Json<ChangeColorRequest>,
) -> Json<ActionResponse> {
    Json(write_controller.change_player_color(change_color_request.into_inner()))
}

/// Sets the player as ready, or not. The player must be authenticated to do so.
///
/// More details in [`ticket_to_ride::manager::Manager::set_ready`].
#[put(
    "/game/<_>/player/is_ready",
    format = "json",
    data = "<set_player_ready_request>"
)]
pub fn set_player_ready(
    mut write_controller: WriteController,
    set_player_ready_request: Json<SetPlayerReadyRequest>,
) -> Json<ActionResponse> {
    Json(write_controller.set_player_ready(set_player_ready_request.into_inner()))
}

/// Allows a player to select which _pending_ destination cards they want to fulfill.
/// The player must be authenticated to do so.
///
/// More details in [`ticket_to_ride::manager::Manager::select_destination_cards`].
#[put(
    "/game/<_>/player/select_destination_cards",
    format = "json",
    data = "<select_destination_cards_request>"
)]
pub fn select_destination_cards(
    mut write_controller: WriteController,
    select_destination_cards_request: Json<SelectDestinationCardsRequest>,
) -> Json<ActionResponse> {
    Json(write_controller.select_destination_cards(select_destination_cards_request.into_inner()))
}

/// Allows a player to draw destination cards.
/// The player must be authenticated to do so.
///
/// More details in [`ticket_to_ride::manager::Manager::draw_destination_cards`].
#[post("/game/<_>/player/draw_destination_cards")]
pub fn draw_destination_cards(mut write_controller: WriteController) -> Json<ActionResponse> {
    Json(write_controller.draw_destination_cards())
}

/// Allows a player to draw one train card from the open-faced deck.
/// The player must be authenticated to do so.
///
/// More details in [`ticket_to_ride::manager::Manager::draw_open_train_card`].
#[post(
    "/game/<_>/player/draw_open_train_card",
    format = "json",
    data = "<draw_open_train_card_request>"
)]
pub fn draw_open_train_card(
    mut write_controller: WriteController,
    draw_open_train_card_request: Json<DrawOpenTrainCardRequest>,
) -> Json<ActionResponse> {
    Json(write_controller.draw_open_train_card(draw_open_train_card_request.into_inner()))
}

/// Allows a player to draw one train card from the close-faced deck.
/// The player must be authenticated to do so.
///
/// More details in [`ticket_to_ride::manager::Manager::draw_close_train_card`].
#[post("/game/<_>/player/draw_close_train_card")]
pub fn draw_close_train_card(mut write_controller: WriteController) -> Json<ActionResponse> {
    Json(write_controller.draw_close_train_card())
}

// TODO: Add integration tests.
/// Retrieves the game state. The player must be authenticated to do so.
///
/// More details in [`ticket_to_ride::manager::Manager::get_state`].
#[get("/game/<_>/state")]
pub fn get_game_state(read_controller: ReadController) -> RawJson<String> {
    RawJson(
        serde_json::to_string(&read_controller.get_game_state())
            .expect("Game state should never fail serializing as JSON"),
    )
}
