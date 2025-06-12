use crate::models::user::User;
use crate::repositories::user_repository::{MySqlUserRepository, UserRepository};
use sqlx::MySqlPool;

pub struct UserService {
    user_repository: MySqlUserRepository,
}

impl UserService {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            user_repository: MySqlUserRepository::new(pool),
        }
    }

    // GRASP - Information Expert 패턴: 사용자 조회 비즈니스 로직
    pub async fn get_user_by_id(&self, user_id: i32) -> Result<Option<User>, String> {
        self.user_repository
            .get_user_by_id(user_id)
            .await
            .map_err(|e| {
                eprintln!("Database error: {}", e);
                "Database error".to_string()
            })
    }
}
