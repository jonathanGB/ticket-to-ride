//! Integration tests for the _Ticket To Ride_ web server.

use crate::authenticator::Identifier;
use crate::authenticator::COOKIE_IDENTIFIER_NAME;
use crate::controller::GameIdManagerMapping;
use crate::request_types::*;
use crate::response_types::ActionResponse;
use crate::rocket;
use crate::router::*;
use crate::STATIC_FILES_PATH;

use regex::Regex;
use rocket::http::Cookie;
use rocket::{
    http::{ContentType, Status},
    local::blocking::Client,
};
use std::fs::{read, read_to_string};
use std::path::Path;
use ticket_to_ride::{
    manager::{GamePhase, Manager},
    player::PlayerColor,
};
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
    validate_state_if(state, game_id, |game_manager| {
        assert_eq!(game_manager.num_players(), expected_num_players);
    });
}

fn validate_state_if<F>(state: &GameIdManagerMapping, game_id: &Uuid, predicate: F)
where
    F: FnOnce(&Manager),
{
    let game_manager = state.get(game_id);
    assert!(game_manager.is_some());
    let game_manager = game_manager.unwrap();

    predicate(&*game_manager);
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

fn create_game(client: &Client) -> Uuid {
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

    game_id.unwrap()
}

#[test]
fn router_change_player_name() {
    let client = Client::untracked(rocket()).expect("valid rocket");
    let game_id = create_game(&client);

    // Load five unique players.

    let cookies: Vec<_> = (1..=5)
        .map(|_| {
            let res = client.get(uri!(load_game(game_id))).dispatch();
            assert_eq!(res.status(), Status::Ok);

            let cookie = res.cookies().get_private(COOKIE_IDENTIFIER_NAME);
            assert!(cookie.is_some());
            cookie.unwrap()
        })
        .collect();
    assert_eq!(cookies.len(), 5);

    let state = client.rocket().state::<GameIdManagerMapping>().unwrap();
    validate_state_num_of_players(state, &game_id, 5);

    // Change the name of the first player.
    let change_name_request = ChangeNameRequest {
        new_name: String::from("Bob"),
    };
    let res = client
        .put(uri!(change_player_name(game_id)))
        .private_cookie(cookies[0].clone())
        .json(&change_name_request)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let res_json = res.into_json();
    assert!(res_json.is_some());
    let res_json: ActionResponse = res_json.unwrap();
    assert!(res_json.success);
    assert!(res_json.error_message.is_none());

    // Changing the name of the first player to the same name should fail.
    let res = client
        .put(uri!(change_player_name(game_id)))
        .private_cookie(cookies[0].clone())
        .json(&change_name_request)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let res_json = res.into_json();
    assert!(res_json.is_some());
    let res_json: ActionResponse = res_json.unwrap();
    assert_eq!(res_json.success, false);
    assert!(res_json.error_message.is_some());

    // Change the name of the second player.
    let change_name_request = ChangeNameRequest {
        new_name: String::from("Alice"),
    };
    let res = client
        .put(uri!(change_player_name(game_id)))
        .private_cookie(cookies[1].clone())
        .json(&change_name_request)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let res_json = res.into_json();
    assert!(res_json.is_some());
    let res_json: ActionResponse = res_json.unwrap();
    assert!(res_json.success);
    assert!(res_json.error_message.is_none());

    // Change the name of the third player to an existing name fails.
    let res = client
        .put(uri!(change_player_name(game_id)))
        .private_cookie(cookies[2].clone())
        .json(&change_name_request)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let res_json = res.into_json();
    assert!(res_json.is_some());
    let res_json: ActionResponse = res_json.unwrap();
    assert_eq!(res_json.success, false);
    assert!(res_json.error_message.is_some());

    // Validate final state.
    validate_state_if(state, &game_id, |game_manager| {
        let game_state = game_manager.get_state(0);
        assert_eq!(game_state.phase, GamePhase::InLobby);

        let players_state = game_state.players_state;
        assert_eq!(
            players_state[0].public_player_state.name,
            String::from("Bob")
        );
        assert_eq!(
            players_state[1].public_player_state.name,
            String::from("Alice")
        );
        assert_ne!(
            players_state[2].public_player_state.name,
            String::from("Alice")
        );
    });
}

#[test]
fn router_change_player_name_unauthenticated() {
    let client = Client::untracked(rocket()).expect("valid rocket");
    let game_id = create_game(&client);

    let state = client.rocket().state::<GameIdManagerMapping>().unwrap();
    validate_state_num_of_players(state, &game_id, 0);

    // Change the name, but no cookies provided to authenticate.
    let change_name_request = ChangeNameRequest {
        new_name: String::from("Bob"),
    };
    let res = client
        .put(uri!(change_player_name(game_id)))
        .json(&change_name_request)
        .dispatch();

    assert_eq!(res.status(), Status::Unauthorized);
}

#[test]
fn router_change_player_name_unauthorized() {
    let client = Client::untracked(rocket()).expect("valid rocket");
    let game_id = create_game(&client);
    let wrong_game_id = Uuid::new_v4();
    let player_id = 0;
    let identifier = Identifier::new(wrong_game_id, player_id);
    let cookie = Cookie::new(COOKIE_IDENTIFIER_NAME, identifier.to_string());

    let state = client.rocket().state::<GameIdManagerMapping>().unwrap();
    validate_state_num_of_players(state, &game_id, 0);

    // Load one player into the game. This player has the same ID as `player_id`.
    let res = client.get(uri!(load_game(game_id))).dispatch();
    assert_eq!(res.status(), Status::Ok);

    // Change the name, but cookie authorizes for a different game ID.
    let change_name_request = ChangeNameRequest {
        new_name: String::from("Bob"),
    };
    let res = client
        .put(uri!(change_player_name(game_id)))
        .private_cookie(cookie)
        .json(&change_name_request)
        .dispatch();

    assert_eq!(res.status(), Status::Unauthorized);
}

#[test]
fn router_change_player_name_game_not_found() {
    let client = Client::untracked(rocket()).expect("valid rocket");
    let game_id = Uuid::new_v4();
    let player_id = 0;
    let identifier = Identifier::new(game_id, player_id);
    let cookie = Cookie::new(COOKIE_IDENTIFIER_NAME, identifier.to_string());

    // Change the name with a valid cookie, but no such game exists.
    let change_name_request = ChangeNameRequest {
        new_name: String::from("Bob"),
    };
    let res = client
        .put(uri!(change_player_name(game_id)))
        .private_cookie(cookie)
        .json(&change_name_request)
        .dispatch();

    assert_eq!(res.status(), Status::NotFound);
}

#[test]
fn router_change_player_color() {
    let client = Client::untracked(rocket()).expect("valid rocket");
    let game_id = create_game(&client);

    // Load five unique players.

    let cookies: Vec<_> = (1..=5)
        .map(|_| {
            let res = client.get(uri!(load_game(game_id))).dispatch();
            assert_eq!(res.status(), Status::Ok);

            let cookie = res.cookies().get_private(COOKIE_IDENTIFIER_NAME);
            assert!(cookie.is_some());
            cookie.unwrap()
        })
        .collect();
    assert_eq!(cookies.len(), 5);

    let state = client.rocket().state::<GameIdManagerMapping>().unwrap();
    validate_state_num_of_players(state, &game_id, 5);

    // Change the color of the first player.
    let change_color_request = ChangeColorRequest {
        new_color: PlayerColor::Red,
    };
    let res = client
        .put(uri!(change_player_color(game_id)))
        .private_cookie(cookies[0].clone())
        .json(&change_color_request)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let res_json = res.into_json();
    assert!(res_json.is_some());
    let res_json: ActionResponse = res_json.unwrap();
    assert!(res_json.success);
    assert!(res_json.error_message.is_none());

    // Changing the color of the first player to the same color should fail.
    let res = client
        .put(uri!(change_player_color(game_id)))
        .private_cookie(cookies[0].clone())
        .json(&change_color_request)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let res_json = res.into_json();
    assert!(res_json.is_some());
    let res_json: ActionResponse = res_json.unwrap();
    assert_eq!(res_json.success, false);
    assert!(res_json.error_message.is_some());

    // Change the color of the second player.
    let change_color_request = ChangeColorRequest {
        new_color: PlayerColor::Yellow,
    };
    let res = client
        .put(uri!(change_player_color(game_id)))
        .private_cookie(cookies[1].clone())
        .json(&change_color_request)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let res_json = res.into_json();
    assert!(res_json.is_some());
    let res_json: ActionResponse = res_json.unwrap();
    assert!(res_json.success);
    assert!(res_json.error_message.is_none());

    // Change the color of the third player to an existing color fails.
    let res = client
        .put(uri!(change_player_color(game_id)))
        .private_cookie(cookies[2].clone())
        .json(&change_color_request)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let res_json = res.into_json();
    assert!(res_json.is_some());
    let res_json: ActionResponse = res_json.unwrap();
    assert_eq!(res_json.success, false);
    assert!(res_json.error_message.is_some());

    // Validate final state.
    validate_state_if(state, &game_id, |game_manager| {
        let game_state = game_manager.get_state(0);
        assert_eq!(game_state.phase, GamePhase::InLobby);

        let players_state = game_state.players_state;
        assert_eq!(players_state[0].public_player_state.color, PlayerColor::Red);
        assert_eq!(
            players_state[1].public_player_state.color,
            PlayerColor::Yellow
        );
        assert_ne!(
            players_state[2].public_player_state.color,
            PlayerColor::Yellow
        );
    });
}

#[test]
fn router_set_player_ready() {
    let client = Client::untracked(rocket()).expect("valid rocket");
    let game_id = create_game(&client);

    // Load five unique players.

    let cookies: Vec<_> = (1..=5)
        .map(|_| {
            let res = client.get(uri!(load_game(game_id))).dispatch();
            assert_eq!(res.status(), Status::Ok);

            let cookie = res.cookies().get_private(COOKIE_IDENTIFIER_NAME);
            assert!(cookie.is_some());
            cookie.unwrap()
        })
        .collect();
    assert_eq!(cookies.len(), 5);

    let state = client.rocket().state::<GameIdManagerMapping>().unwrap();
    validate_state_num_of_players(state, &game_id, 5);
    validate_state_if(state, &game_id, |game_manager| {
        assert!(game_manager
            .get_state(0)
            .players_state
            .iter()
            .all(|player| !player.public_player_state.is_ready));
    });

    // Set all players (except the first one) as ready.
    cookies.iter().skip(1).for_each(|cookie| {
        let set_player_ready_request = SetPlayerReadyRequest { is_ready: true };
        let res = client
            .put(uri!(set_player_ready(game_id)))
            .private_cookie(cookie.clone())
            .json(&set_player_ready_request)
            .dispatch();

        assert_eq!(res.status(), Status::Ok);
        let res_json = res.into_json();
        assert!(res_json.is_some());
        let res_json: ActionResponse = res_json.unwrap();
        assert!(res_json.success);
        assert!(res_json.error_message.is_none());
    });

    validate_state_if(state, &game_id, |game_manager| {
        assert_eq!(game_manager.get_state(0).phase, GamePhase::InLobby);
    });

    // Setting a player as ready (without a cookie) should fail.
    let set_player_ready_request = SetPlayerReadyRequest { is_ready: true };
    let res = client
        .put(uri!(set_player_ready(game_id)))
        .json(&set_player_ready_request)
        .dispatch();

    assert_eq!(res.status(), Status::Unauthorized);

    // Set the first player as not ready should change nothing.
    let set_player_ready_request = SetPlayerReadyRequest { is_ready: false };
    let res = client
        .put(uri!(set_player_ready(game_id)))
        .private_cookie(cookies[0].clone())
        .json(&set_player_ready_request)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let res_json = res.into_json();
    assert!(res_json.is_some());
    let res_json: ActionResponse = res_json.unwrap();
    assert!(res_json.success);
    assert!(res_json.error_message.is_none());

    // Validate that we are still in the lobby.
    validate_state_if(state, &game_id, |game_manager| {
        assert_eq!(game_manager.get_state(0).phase, GamePhase::InLobby);
    });

    // Set the first player as ready now.
    let set_player_ready_request = SetPlayerReadyRequest { is_ready: true };
    let res = client
        .put(uri!(set_player_ready(game_id)))
        .private_cookie(cookies[0].clone())
        .json(&set_player_ready_request)
        .dispatch();

    assert_eq!(res.status(), Status::Ok);
    let res_json = res.into_json();
    assert!(res_json.is_some());
    let res_json: ActionResponse = res_json.unwrap();
    assert!(res_json.success);
    assert!(res_json.error_message.is_none());

    // Validate that we have moved out of the lobby.
    validate_state_if(state, &game_id, |game_manager| {
        assert_eq!(game_manager.get_state(0).phase, GamePhase::Starting);
    });
}
