//! Launches a HTTP server, and routes incoming requests to the Controller.

#[macro_use]
extern crate rocket;

mod authenticator;
mod controller;
mod request_types;
mod response_types;
mod router;

#[cfg(test)]
mod router_tests;

use crate::router::*;

use controller::GameIdManagerMapping;
use rocket::fs::FileServer;

/// Path to static files.
const STATIC_FILES_PATH: &str = "../../frontend/build/static";

#[launch]
/// Launches the web server.
fn rocket() -> _ {
    let game_id_manager_mapping = GameIdManagerMapping::new();
    rocket::build()
        .mount(
            "/",
            routes![
                change_player_color,
                change_player_name,
                create_game,
                draw_destination_cards,
                draw_open_train_card,
                get_game_state,
                index,
                load_game,
                robots,
                root,
                select_destination_cards,
                set_player_ready,
            ],
        )
        .mount("/static", FileServer::from(STATIC_FILES_PATH))
        .manage(game_id_manager_mapping)
}
