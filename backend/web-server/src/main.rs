#[macro_use]
extern crate rocket;

use rocket::{fs::NamedFile, response::Redirect, serde::uuid::Uuid, uri};
use std::path::{Path, PathBuf};

#[get("/")]
async fn index() -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new("../../frontend/build/index.html")).await
}

#[get("/<file..>")]
async fn build_dir(file: PathBuf) -> std::io::Result<NamedFile> {
    NamedFile::open(Path::new("../../frontend/build").join(file)).await
}

#[get("/game/<game_id>")]
fn load_game(game_id: Uuid) -> String {
    format!("Loaded game with ID = {}", game_id)
}

#[post("/create")]
fn create_game() -> Redirect {
    Redirect::to(uri!(load_game(Uuid::new_v4())))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, build_dir, create_game, load_game])
}
