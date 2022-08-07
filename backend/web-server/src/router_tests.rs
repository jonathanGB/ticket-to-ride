//! Integration tests for the _Ticket To Ride_ web server.

use crate::rocket;
use crate::router::*;
use crate::STATIC_FILES_PATH;

use rocket::{
    http::{ContentType, Status},
    local::blocking::Client,
};
use std::fs::{read, read_to_string};
use std::path::Path;

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
