use crate::auth::AuthToken;
use crate::models::user::User;
use crate::services::user_service::UserService;
use rocket::State;
use rocket::get;
use rocket::http::Status;
use rocket::serde::json::Json;
use sqlx::MySqlPool;

// GRASP - Controller 패턴: HTTP 요청/응답만 처리, 비즈니스 로직은 서비스에 위임
#[get("/api/mypage")]
pub async fn api_mypage(
    auth_token: AuthToken,
    pool: &State<MySqlPool>,
) -> Result<Json<User>, (Status, String)> {
    let user_id = auth_token.0.sub.parse::<i32>().map_err(|_| {
        eprintln!("Failed to parse user ID from token subject");
        (Status::Unauthorized, "Invalid token".to_string())
    })?;

    // GRASP - Low Coupling: 서비스 레이어를 통한 비즈니스 로직 처리
    let user_service = UserService::new(pool.inner().clone());
    user_service
        .get_user_by_id(user_id)
        .await
        .map_err(|e| (Status::InternalServerError, e))
        .and_then(|user_opt| {
            user_opt
                .map(Json)
                .ok_or((Status::NotFound, "User not found".to_string()))
        })
}
