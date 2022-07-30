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

const BUILD_FILES_PATH: &str = "../../frontend/build";

#[inline]
fn redirect_to_root() -> Redirect {
    Redirect::to(uri!(root()))
}

#[get("/")]
pub async fn root() -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new(BUILD_FILES_PATH).join("index.html")).await
}

#[get("/index.html")]
pub fn index() -> Redirect {
    Redirect::permanent(uri!(root()))
}

#[get("/robots.txt")]
pub async fn robots() -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new(BUILD_FILES_PATH).join("robots.txt")).await
}

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

#[post("/create")]
pub fn create_game(state: &State<GameIdManagerMapping>) -> Redirect {
    let game_id = WriteController::create_game(state);

    Redirect::to(uri!(load_game(game_id)))
}

#[get("/game/<_>/state")]
pub fn get_game_state(read_controller: ReadController) -> RawJson<String> {
    RawJson(
        serde_json::to_string(&read_controller.get_game_state())
            .expect("Game state should never fail serializing as JSON"),
    )
}
