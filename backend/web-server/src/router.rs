use crate::controller::{GameIdManagerMapping, WriteController};
use crate::request_types::*;
use crate::response_types::*;

use rocket::serde::{json::Json, Serialize};
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

#[post("/create")]
pub fn create_game(state: &State<GameIdManagerMapping>) -> Redirect {
    let game_id = WriteController::create_game(state);

    Redirect::to(uri!(load_game(game_id)))
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Foo {
    pub foo: usize,
    pub bar: bool,
    pub game_id: Uuid,
    pub phase: String,
}

#[get("/game/<game_id>/state")]
pub fn get_game_state(game_id: Uuid, cookies: &CookieJar) -> Json<Foo> {
    Json(Foo {
        foo: 5,
        bar: true,
        game_id,
        phase: String::from("in_lobby"),
    })
}
