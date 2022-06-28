// Launches a HTTP server, and routes incoming requests to the Controller.

#[macro_use]
extern crate rocket;

mod authenticator;
mod controller;
mod model;

use controller::Controller;
use dashmap::DashMap;
use rocket::serde::{json::Json, Serialize};
use rocket::{
    fs::NamedFile,
    http::{uri::Origin, CookieJar},
    response::Redirect,
    serde::uuid::Uuid,
    State,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

const BUILD_FILES_PATH: &str = "../../frontend/build";
const STATIC_FILES_PATH: &str = "../../frontend/build/static";

#[get("/")]
async fn root(state: &State<DashMap<Uuid, u8>>) -> std::io::Result<NamedFile> {
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

#[get("/static/<file..>")]
async fn serve_static_files(file: PathBuf) -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new(STATIC_FILES_PATH).join(file)).await
}

#[get("/game/<game_id>")]
fn load_game(
    game_id: Uuid,
    cookies: &CookieJar,
    origin: &Origin,
    state: &State<DashMap<Uuid, u8>>,
) -> String {
    let mut h = HashMap::<Uuid, u8>::new();
    h.insert(game_id, 10);

    let start = Instant::now();
    let val = state.get(&game_id);

    println!("Found {:?}:{:?} in {:?}", game_id, val, start.elapsed());

    let start = Instant::now();
    let val = h.get(&game_id);
    println!(
        "Found (in hashmap) {:?}:{:?} in {:?}",
        game_id,
        val,
        start.elapsed()
    );

    Controller::new(game_id).load_game(cookies, origin)
}

#[post("/create")]
fn create_game(state: &State<DashMap<Uuid, u8>>) -> Redirect {
    let game_id = Controller::create_game();
    let start = Instant::now();
    state.insert(game_id, 9);
    println!("Duration: {:?}", start.elapsed());
    Redirect::to(uri!(load_game(game_id)))
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Foo {
    pub foo: usize,
    pub bar: bool,
    pub game_id: Uuid,
}

#[get("/game/<game_id>/state")]
fn get_game_state(game_id: Uuid, cookies: &CookieJar) -> Json<Foo> {
    Json(Foo {
        foo: 5,
        bar: true,
        game_id,
    })
}

#[launch]
fn rocket() -> _ {
    let h = DashMap::<Uuid, u8>::new();
    rocket::build()
        .mount(
            "/",
            routes![
                root,
                index,
                robots,
                serve_static_files,
                create_game,
                load_game,
                get_game_state
            ],
        )
        .manage(h)
}
