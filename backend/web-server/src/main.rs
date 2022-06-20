// Launches a HTTP server, and routes incoming requests to the Controller.

#[macro_use]
extern crate rocket;

mod authenticator;
mod controller;

use controller::Controller;
use rocket::{
    fs::NamedFile,
    http::{uri::Origin, CookieJar},
    response::Redirect,
    serde::uuid::Uuid,
};
use std::path::{Path, PathBuf};

const STATIC_FILES_PATH: &str = "../../frontend/build";

#[get("/")]
async fn index() -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new(STATIC_FILES_PATH).join("index.html")).await
}

#[get("/<file..>")]
async fn serve_static_files(file: PathBuf) -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new(STATIC_FILES_PATH).join(file)).await
}

#[get("/game/<game_id>")]
fn load_game(game_id: Uuid, cookies: &CookieJar, origin: &Origin) -> String {
    Controller::new(game_id).load_game(cookies, origin)
}

#[post("/create")]
fn create_game() -> Redirect {
    let game_id = Controller::create_game();
    Redirect::to(uri!(load_game(game_id)))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes![index, serve_static_files, create_game, load_game],
    )
}
