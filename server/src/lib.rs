use bcrypt::{DEFAULT_COST, hash, verify};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlQueryResult;
use sqlx::{Error, MySqlPool};

#[derive(Serialize, Deserialize)]
pub enum FuelType {
    Gasoline,
    Diesel,
    Hybrid,
    Electric,
}
#[derive(Serialize, Deserialize)]
pub enum CarStatus {
    Available,
    Unavailable,
}

#[derive(Serialize, Deserialize)]
pub struct CarInfo {
    plate_number: String,
    manufacture: String,
    car_name: String,
    fuel_type: FuelType,
    seat_num: u8,
    price: u32,
    status: CarStatus,
}

impl FuelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FuelType::Gasoline => "Gasoline",
            FuelType::Diesel => "Diesel",
            FuelType::Electric => "Electric",
            FuelType::Hybrid => "Hybrid",
        }
    }
}

impl CarStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CarStatus::Available => "Available",
            CarStatus::Unavailable => "Unavailable",
        }
    }
}

impl CarInfo {
    fn get_plate_number(&self) -> &str {
        &self.plate_number
    }
    fn get_manufacture(&self) -> &str {
        &self.manufacture
    }
    fn get_car_name(&self) -> &str {
        &self.car_name
    }
    fn get_fuel_type(&self) -> &str {
        self.fuel_type.as_str()
    }
    fn get_seat_num(&self) -> u8 {
        self.seat_num
    }
    fn get_price(&self) -> u32 {
        self.price
    }
    fn get_status(&self) -> &str {
        self.status.as_str()
    }
}

pub async fn add_car(pool: &MySqlPool, info: CarInfo) -> Result<String, String> {
    let check_query = "SELECT plate_number FROM cars WHERE plate_number = ?";
    let existing_car = sqlx::query(check_query)
        .bind(&info.plate_number)
        .fetch_optional(pool)
        .await;

    match existing_car {
        Ok(Some(_)) => {
            return Err(format!(
                "Car with plate number '{}' already exists!",
                info.plate_number
            ));
        }
        Ok(None) => {
            // plate_number가 존재하지 않으므로 차량 추가 진행
            let query = r#"
                INSERT INTO cars (plate_number, manufacture, car_name, fuel_type, seat_num, price, status)
                VALUES (?, ?, ?, ?, ?, ?, ?)
            "#;

            let result = sqlx::query(query)
                .bind(&info.plate_number)
                .bind(&info.manufacture)
                .bind(&info.car_name)
                .bind(info.fuel_type.as_str())
                .bind(info.seat_num)
                .bind(info.price)
                .bind(info.status.as_str())
                .execute(pool)
                .await;

            match result {
                Ok(_) => Ok(format!("Car '{}' added successfully!", info.car_name)),
                Err(e) => Err(format!("Failed to add car: {}", e)),
            }
        }
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

pub async fn update_car(pool: &MySqlPool, info: CarInfo) -> Result<String, String> {
    // plate_number를 기준으로 차량이 존재하는지 확인
    let check_query = "SELECT plate_number FROM cars WHERE plate_number = ?";
    let existing_car = sqlx::query(check_query)
        .bind(&info.plate_number)
        .fetch_optional(pool)
        .await;

    match existing_car {
        Ok(Some(_)) => {
            // 차량이 존재하므로 업데이트 수행
            let query = r#"
                UPDATE cars
                SET manufacture = ?, car_name = ?, fuel_type = ?, seat_num = ?, price = ?, status = ?
                WHERE plate_number = ?
            "#;

            let result = sqlx::query(query)
                .bind(&info.manufacture)
                .bind(&info.car_name)
                .bind(info.fuel_type.as_str())
                .bind(info.seat_num)
                .bind(info.price)
                .bind(info.status.as_str())
                .bind(&info.plate_number)
                .execute(pool)
                .await;

            match result {
                Ok(_) => Ok(format!(
                    "Car with plate number '{}' updated successfully!",
                    info.plate_number
                )),
                Err(e) => Err(format!("Failed to update car: {}", e)),
            }
        }
        Ok(None) => Err(format!(
            "Car with plate number '{}' not found!",
            info.plate_number
        )),
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
}

// 응답 구조체 (auth.rs 에서 사용)
#[derive(serde::Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
    pub data: Option<String>,
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

pub async fn find_user_by_email(pool: &MySqlPool, email: &str) -> Result<Option<User>, Error> {
    // 사용자 메일로 사용자 검색
    let user = sqlx::query_as!(
        User,
        "SELECT name, id, email, password FROM users WHERE email = ?",
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    // 해시된 비밀번호와 입력된 비밀번호 비교
    verify(password, hashed_password).unwrap_or(false)
}
