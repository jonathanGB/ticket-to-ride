// Launches a HTTP server, and routes incoming requests to the Controller.

#[macro_use]
extern crate rocket;

mod authenticator;
mod controller;
mod model;
mod request_types;
mod response_types;

use controller::{GameIdManagerMapping, WriteController};
use request_types::*;
use response_types::*;
use rocket::serde::{json::Json, Serialize};
use rocket::{
    fs::{FileServer, NamedFile},
    http::{uri::Origin, CookieJar},
    response::Redirect,
    serde::uuid::Uuid,
    State,
};
use std::path::Path;

const BUILD_FILES_PATH: &str = "../../frontend/build";
const STATIC_FILES_PATH: &str = "../../frontend/build/static";

#[inline]
fn redirect_to_root() -> Redirect {
    Redirect::to(uri!(root()))
}

#[get("/")]
async fn root() -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new(BUILD_FILES_PATH).join("index.html")).await
}

#[get("/index.html")]
fn index() -> Redirect {
    Redirect::permanent(uri!(root()))
}

#[get("/robots.txt")]
async fn robots() -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new(BUILD_FILES_PATH).join("robots.txt")).await
}

#[get("/game/<game_id>")]
async fn load_game(
    game_id: Uuid,
    cookies: &CookieJar<'_>,
    origin: &Origin<'_>,
    state: &State<GameIdManagerMapping>,
) -> Result<NamedFile, LoadGameError> {
    match state.get_mut(&game_id) {
        Some(game_id_and_state) => {
            if !WriteController::new(game_id_and_state, cookies).load_game(origin) {
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
    "/game/<game_id>/player/name",
    format = "json",
    data = "<change_name_request>"
)]
fn change_player_name(
    game_id: Uuid,
    cookies: &CookieJar,
    state: &State<GameIdManagerMapping>,
    change_name_request: Json<ChangeNameRequest>,
) -> Option<Json<ChangeNameResponse>> {
    let game_id_and_state = state.get_mut(&game_id)?;

    Some(Json(
        WriteController::new(game_id_and_state, cookies)
            .change_player_name(change_name_request.into_inner()),
    ))
}

#[post("/create")]
fn create_game(state: &State<GameIdManagerMapping>) -> Redirect {
    let game_id = WriteController::create_game(state);

    Redirect::to(uri!(load_game(game_id)))
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Foo {
    pub foo: usize,
    pub bar: bool,
    pub game_id: Uuid,
    pub phase: String,
}

#[get("/game/<game_id>/state")]
fn get_game_state(game_id: Uuid, cookies: &CookieJar) -> Json<Foo> {
    Json(Foo {
        foo: 5,
        bar: true,
        game_id,
        phase: String::from("in_lobby"),
    })
}

#[launch]
fn rocket() -> _ {
    let game_id_manager_mapping = GameIdManagerMapping::new();
    rocket::build()
        .mount(
            "/",
            routes![
                change_player_name,
                create_game,
                get_game_state,
                index,
                load_game,
                robots,
                root,
            ],
        )
        .mount("/static", FileServer::from(STATIC_FILES_PATH))
        .manage(game_id_manager_mapping)
}
