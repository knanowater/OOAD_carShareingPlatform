use jsonwebtoken::{EncodingKey, Header, encode};
use rocket::serde::json::Json;
use serde::Serialize;
use server::{ApiResponse, create_user, find_user_by_email, verify_password};
use sqlx::MySqlPool;
use std::env;

#[derive(Serialize)]
struct Claims {
    sub: String,   // 사용자 ID
    name: String,  // 사용자 이름
    email: String, // 사용자 이메일
    exp: usize,    // 만료 시간 (Unix timestamp)
}

// 회원 가입 요청을 처리하는 구조체
#[derive(serde::Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

// 로그인 요청을 처리하는 구조체
#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

// 회원 가입 엔드포인트
#[post("/api/signup", format = "json", data = "<user_info>")]
pub async fn signup(
    user_info: Json<CreateUserRequest>,
    pool: &rocket::State<MySqlPool>,
) -> rocket::serde::json::Json<ApiResponse> {
    let user_info = user_info.into_inner();

    // 사용자 이름이 이미 존재하는지 확인
    match find_user_by_email(pool, &user_info.email).await {
        Ok(Some(_)) => {
            return Json(ApiResponse {
                status: "error".to_string(),
                message: "Email already exists".to_string(),
                data: None,
            });
        }
        Ok(None) => {
            // 사용자를 데이터베이스에 추가
            match create_user(pool, &user_info.name, &user_info.email, &user_info.password).await {
                Ok(_) => {
                    return Json(ApiResponse {
                        status: "success".to_string(),
                        message: "User created successfully".to_string(),
                        data: None,
                    });
                }
                Err(e) => {
                    eprintln!("Failed to create user: {}", e);
                    return Json(ApiResponse {
                        status: "error".to_string(),
                        message: format!("Failed to create user: {}", e),
                        data: None,
                    });
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to check email: {}", e);
            return Json(ApiResponse {
                status: "error".to_string(),
                message: format!("Failed to check email: {}", e),
                data: None,
            });
        }
    }
}

// 로그인 엔드포인트
#[post("/api/login", format = "json", data = "<login_info>")]
pub async fn login(
    login_info: Json<LoginRequest>,
    pool: &rocket::State<MySqlPool>,
) -> rocket::serde::json::Json<ApiResponse> {
    let login_info = login_info.into_inner();
    match find_user_by_email(pool, &login_info.email).await {
        Ok(Some(user)) => {
            if verify_password(&login_info.password, &user.password) {
                // JWT 생성
                let expiration = chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::hours(12))
                    .expect("valid timestamp")
                    .timestamp() as usize;

                let claims = Claims {
                    sub: user.id.to_string(),
                    name: user.name,
                    email: user.email,
                    exp: expiration,
                };

                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(env::var("JWT_SECRET").unwrap().as_ref()),
                )
                .expect("Token creation failed");

                return Json(ApiResponse {
                    status: "success".to_string(),
                    message: "Login successful".to_string(),
                    data: Some(token),
                });
            } else {
                return Json(ApiResponse {
                    status: "error".to_string(),
                    message: "Invalid credentials".to_string(),
                    data: None,
                });
            }
        }
        Ok(None) => Json(ApiResponse {
            status: "error".to_string(),
            message: "User not found".to_string(),
            data: None,
        }),
        Err(_) => Json(ApiResponse {
            status: "error".to_string(),
            message: "Internal server error".to_string(),
            data: None,
        }),
    }
}

// 로그아웃 엔드포인트
#[get("/api/logout")]
pub async fn logout() -> rocket::serde::json::Json<ApiResponse> {
    // 로그아웃 로직은 필요에 따라 구현 (예: 세션 무효화)
    Json(ApiResponse {
        status: "success".to_string(),
        message: "Logout successful".to_string(),
        data: None,
    })
}
