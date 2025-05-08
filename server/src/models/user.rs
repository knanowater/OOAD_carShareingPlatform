use rocket::serde::{Deserialize, Serialize}; // rocket의 Serialize/Deserialize 사용
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserWithPassword {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
}
