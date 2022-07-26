// Launches a HTTP server, and routes incoming requests to the Controller.

#[macro_use]
extern crate rocket;

mod authenticator;
mod controller;
mod model;
mod request_types;
mod response_types;
mod router;

use crate::router::*;

use controller::GameIdManagerMapping;
use rocket::fs::FileServer;

const STATIC_FILES_PATH: &str = "../../frontend/build/static";

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
