use crate::models::user::User;
use async_trait::async_trait;
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
