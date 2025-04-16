use crate::auth::AuthToken;
use crate::models::user::User;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{State, get};
use sqlx::MySqlPool; // 사용자 모델 임포트

#[get("/api/mypage")]
pub async fn api_mypage(
    auth_token: AuthToken,
    pool: &State<MySqlPool>,
) -> Result<Json<User>, Status> {
    // 기존 main.rs의 api_mypage 로직 전체 이동
    let user_id = auth_token.0.sub.parse::<i32>().map_err(|_| {
        eprintln!("Failed to parse user ID from token subject");
        Status::Unauthorized
    })?;

    let mut conn = pool.acquire().await.map_err(|e| {
        eprintln!("Failed to acquire connection from pool: {}", e);
        Status::InternalServerError
    })?;

    let user = sqlx::query_as::<_, User>("SELECT id, name, email, role FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Status::NotFound,
            _ => {
                eprintln!("Failed to fetch user: {}", e); // 에러 로깅 추가
                Status::InternalServerError
            }
        })?;

    Ok(Json(user))
}
