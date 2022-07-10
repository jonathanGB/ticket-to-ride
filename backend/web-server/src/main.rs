// Launches a HTTP server, and routes incoming requests to the Controller.

#[macro_use]
extern crate rocket;

mod authenticator;
mod controller;
mod model;

use controller::{Controller, GameState};
use dashmap::DashMap;
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

#[derive(Responder)]
enum LoadGameError {
    NoFile(std::io::Error),
    NoGame(Redirect),
    Unauthorized(Redirect),
}

#[get("/game/<game_id>")]
async fn load_game(
    game_id: Uuid,
    cookies: &CookieJar<'_>,
    origin: &Origin<'_>,
    state: &State<DashMap<Uuid, Vec<usize>>>,
) -> Result<NamedFile, LoadGameError> {
    match state.get_mut(&game_id) {
        Some(game_id_and_state) => {
            if !Controller::new(game_id_and_state).load_game(cookies, origin) {
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

#[post("/create")]
fn create_game(state: &State<DashMap<Uuid, Vec<usize>>>) -> Redirect {
    let game_id = Controller::create_game(state);

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
    let h = DashMap::<Uuid, GameState>::new();
    rocket::build()
        .mount(
            "/",
            routes![root, index, robots, create_game, load_game, get_game_state],
        )
        .mount("/static", FileServer::from(STATIC_FILES_PATH))
        .manage(h)
}
