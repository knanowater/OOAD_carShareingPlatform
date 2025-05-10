use crate::auth::AuthToken;
use crate::models::user::User;
use crate::repositories::user_repository::{MySqlUserRepository, UserRepository};
use rocket::State;
use rocket::get;
use rocket::http::Status;
use rocket::serde::json::Json;
use sqlx::MySqlPool;

#[get("/api/mypage")]
pub async fn api_mypage(
    auth_token: AuthToken,
    pool: &State<MySqlPool>,
) -> Result<Json<User>, (Status, String)> {
    let user_id = auth_token.0.sub.parse::<i32>().map_err(|_| {
        eprintln!("Failed to parse user ID from token subject");
        (Status::Unauthorized, "Invalid token".to_string())
    })?;
    let user_repo = MySqlUserRepository::new(pool.inner().clone());
    user_repo
        .get_user_by_id(user_id)
        .await
        .map(|user_opt| {
            user_opt
                .map(Json)
                .ok_or((Status::NotFound, "User not found".to_string()))
        })
        .map_err(|e| {
            eprintln!("Database error: {}", e);
            (Status::InternalServerError, "Database error".to_string())
        })
        .and_then(std::convert::identity)
}
