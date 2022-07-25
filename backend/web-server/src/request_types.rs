use rocket::serde::Deserialize;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ChangeNameRequest {
    pub new_name: String,
}
