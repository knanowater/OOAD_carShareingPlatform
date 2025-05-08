use crate::models::user::{User, UserWithPassword};
use async_trait::async_trait;
use bcrypt::{DEFAULT_COST, hash, verify as bcrypt_verify};
use sqlx::mysql::MySqlQueryResult;
use sqlx::{Error, MySqlPool};

#[async_trait]
pub trait UserRepository {
    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, Error>;
}

pub struct MySqlUserRepository {
    pool: MySqlPool,
}

impl MySqlUserRepository {
    pub fn new(pool: MySqlPool) -> Self {
        MySqlUserRepository { pool }
    }
}

#[async_trait]
impl UserRepository for MySqlUserRepository {
    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, Error> {
        let sql = "SELECT id, name, email, role FROM users WHERE id = ?";
        sqlx::query_as::<_, User>(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }
}

pub async fn create_user(
    pool: &MySqlPool,
    name: &str,
    email: &str,
    password: &str,
) -> Result<MySqlQueryResult, Error> {
    // 비밀번호 해싱
    let hashed_password = hash(password, DEFAULT_COST).expect("Failed to hash password");

    // 데이터베이스에 사용자 추가
    let result = sqlx::query!(
        "INSERT INTO users (name, email, password) VALUES (?, ?, ?)",
        name,
        email,
        hashed_password
    )
    .execute(pool)
    .await?;

    Ok(result)
}

pub async fn find_user_by_email(
    pool: &MySqlPool,
    email: &str,
) -> Result<Option<UserWithPassword>, Error> {
    // 사용자 메일로 사용자 검색
    let user = sqlx::query_as!(
        UserWithPassword,
        "SELECT name, id, email, password, role FROM users WHERE email = ?",
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub fn verify_password(provided_password: &str, stored_password: &str) -> bool {
    bcrypt_verify(provided_password, stored_password).unwrap_or(false)
}
