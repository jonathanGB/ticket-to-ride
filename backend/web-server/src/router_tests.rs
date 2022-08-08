//! Integration tests for the _Ticket To Ride_ web server.

use crate::authenticator::COOKIE_IDENTIFIER_NAME;
use crate::controller::GameIdManagerMapping;
use crate::rocket;
use crate::router::*;
use crate::STATIC_FILES_PATH;

use regex::Regex;
use rocket::{
    http::{ContentType, Status},
    local::blocking::Client,
};
use std::fs::{read, read_to_string};
use std::path::Path;
use uuid::Uuid;

// Tests for file-handling routes.

#[test]
fn router_root() {
    let client = Client::tracked(rocket()).expect("valid rocket");
    let res = client.get(uri!(root())).dispatch();

    assert_eq!(res.status(), Status::Ok);
    assert_eq!(res.content_type(), Some(ContentType::HTML));
    assert_eq!(res.cookies().iter().count(), 0);

    let res_str = res.into_string();
    assert!(res_str.is_some());
    let res_str = res_str.unwrap();

    let expected_res_str = read_to_string(Path::new(BUILD_FILES_PATH).join("index.html"));
    assert!(expected_res_str.is_ok());
    let expected_res_str = expected_res_str.unwrap();

    assert_eq!(res_str, expected_res_str);
}

#[test]
fn router_index_redirects_to_root() {
    let client = Client::tracked(rocket()).expect("valid rocket");
    let res = client.get(uri!(index())).dispatch();

    assert_eq!(res.status(), Status::PermanentRedirect);
    assert_eq!(res.cookies().iter().count(), 0);
    assert_eq!(res.headers().get_one("location"), Some("/"));
}

#[test]
fn router_robots() {
    let client = Client::tracked(rocket()).expect("valid rocket");
    let res = client.get(uri!(robots())).dispatch();

    assert_eq!(res.status(), Status::Ok);
    assert_eq!(res.content_type(), Some(ContentType::Text));
    assert_eq!(res.cookies().iter().count(), 0);

    let res_str = res.into_string();
    assert!(res_str.is_some());
    let res_str = res_str.unwrap();

    let expected_res_str = read_to_string(Path::new(BUILD_FILES_PATH).join("robots.txt"));
    assert!(expected_res_str.is_ok());
    let expected_res_str = expected_res_str.unwrap();

    assert_eq!(res_str, expected_res_str);
}

#[test]
fn router_static() {
    let client = Client::tracked(rocket()).expect("valid rocket");
    let res = client.get("/static/favicon.ico").dispatch();

    assert_eq!(res.status(), Status::Ok);
    assert_eq!(res.content_type(), Some(ContentType::Icon));
    assert_eq!(res.cookies().iter().count(), 0);

    let res_str = res.into_bytes();
    assert!(res_str.is_some());
    let res_str = res_str.unwrap();

    let expected_res_str = read(Path::new(STATIC_FILES_PATH).join("favicon.ico"));
    assert!(expected_res_str.is_ok());
    let expected_res_str = expected_res_str.unwrap();

    assert_eq!(res_str, expected_res_str);
}

fn validate_state_num_of_players(
    state: &GameIdManagerMapping,
    game_id: &Uuid,
    expected_num_players: usize,
) {
    let game_manager = state.get(game_id);
    assert!(game_manager.is_some());
    let game_manager = game_manager.unwrap();
    assert_eq!(game_manager.num_players(), expected_num_players);
}

#[test]
fn router_create_and_load_game() {
    let client = Client::untracked(rocket()).expect("valid rocket");
    let res = client.post(uri!(create_game())).dispatch();

    assert_eq!(res.status(), Status::SeeOther);
    assert_eq!(res.cookies().iter().count(), 0);

    let game_path_str = res.headers().get_one("location");
    assert!(game_path_str.is_some());
    let game_path_str = game_path_str.unwrap();

    let captures = Regex::new(r"^/game/([0-9a-f-]+)$")
        .unwrap()
        .captures(game_path_str);
    assert!(captures.is_some());
    let captures = captures.unwrap();

    assert_eq!(captures.len(), 2);
    let game_id = Uuid::parse_str(captures.get(1).unwrap().as_str());
    assert!(game_id.is_ok());
    let game_id = game_id.unwrap();

    // Now, let's try to load the game.

    let state = client.rocket().state::<GameIdManagerMapping>().unwrap();
    validate_state_num_of_players(state, &game_id, 0);

    let res = client.get(uri!(load_game(game_id))).dispatch();
    assert_eq!(res.status(), Status::Ok);
    assert_eq!(res.content_type(), Some(ContentType::HTML));
    assert_eq!(res.cookies().iter().count(), 1);

    let cookie = res.cookies().get_private(COOKIE_IDENTIFIER_NAME);
    assert!(cookie.is_some());
    let cookie = cookie.unwrap();
    assert_eq!(cookie.value(), format!("{}/0", game_id));
    assert_eq!(cookie.path(), Some(game_path_str));

    let res_str = res.into_string();
    assert!(res_str.is_some());
    let res_str = res_str.unwrap();

    let expected_res_str = read_to_string(Path::new(BUILD_FILES_PATH).join("index.html"));
    assert!(expected_res_str.is_ok());
    let expected_res_str = expected_res_str.unwrap();

    assert_eq!(res_str, expected_res_str);

    validate_state_num_of_players(state, &game_id, 1);

    // Issue an idempotent request, which should not update the state.

    let res = client
        .get(uri!(load_game(game_id)))
        .private_cookie(cookie)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    assert_eq!(res.content_type(), Some(ContentType::HTML));
    // No new cookies are added.
    assert_eq!(res.cookies().iter().count(), 0);

    // Verify that we still have a single player in the state.
    // The reason is that the last request was already authenticated.
    validate_state_num_of_players(state, &game_id, 1);

    // Now, let's have a second client loading the game.

    let res = client.get(uri!(load_game(game_id))).dispatch();

    assert_eq!(res.status(), Status::Ok);
    assert_eq!(res.content_type(), Some(ContentType::HTML));
    assert_eq!(res.cookies().iter().count(), 1);

    let cookie = res.cookies().get_private(COOKIE_IDENTIFIER_NAME);
    assert!(cookie.is_some());
    let cookie = cookie.unwrap();
    assert_eq!(cookie.value(), format!("{}/1", game_id));
    assert_eq!(cookie.path(), Some(game_path_str));

    validate_state_num_of_players(state, &game_id, 2);
}

#[test]
fn router_load_game_not_found() {
    let client = Client::untracked(rocket()).expect("valid rocket");
    let game_id = Uuid::new_v4();
    let res = client.get(uri!(load_game(game_id))).dispatch();

    assert_eq!(res.status(), Status::SeeOther);
    assert_eq!(res.cookies().iter().count(), 0);
}

#[test]
fn router_load_game_too_many_players() {
    let client = Client::untracked(rocket()).expect("valid rocket");
    let res = client.post(uri!(create_game())).dispatch();

    assert_eq!(res.status(), Status::SeeOther);
    assert_eq!(res.cookies().iter().count(), 0);

    let game_path_str = res.headers().get_one("location");
    assert!(game_path_str.is_some());
    let game_path_str = game_path_str.unwrap();

    let captures = Regex::new(r"^/game/([0-9a-f-]+)$")
        .unwrap()
        .captures(game_path_str);
    assert!(captures.is_some());
    let captures = captures.unwrap();

    assert_eq!(captures.len(), 2);
    let game_id = Uuid::parse_str(captures.get(1).unwrap().as_str());
    assert!(game_id.is_ok());
    let game_id = game_id.unwrap();

    let state = client.rocket().state::<GameIdManagerMapping>().unwrap();
    validate_state_num_of_players(state, &game_id, 0);

    // Load five unique players.

    for i in 1..=5 {
        let res = client.get(uri!(load_game(game_id))).dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert_eq!(res.content_type(), Some(ContentType::HTML));
        assert_eq!(res.cookies().iter().count(), 1);

        let cookie = res.cookies().get_private(COOKIE_IDENTIFIER_NAME);
        assert!(cookie.is_some());
        let cookie = cookie.unwrap();
        assert_eq!(cookie.value(), format!("{}/{}", game_id, i - 1));
        assert_eq!(cookie.path(), Some(game_path_str));

        validate_state_num_of_players(state, &game_id, i);
    }

    // The 6th player to join should fail.
    let res = client.get(uri!(load_game(game_id))).dispatch();
    assert_eq!(res.status(), Status::SeeOther);
    assert_eq!(res.cookies().iter().count(), 0);
}
