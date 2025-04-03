use bcrypt::{DEFAULT_COST, hash, verify};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlQueryResult;
use sqlx::{Error, MySqlPool};

trait Car {
    fn get_plate_number(&self) -> &str;
    fn get_manufacture(&self) -> &str;
    fn get_car_name(&self) -> &str;
    fn get_fuel_type(&self) -> &str;
    fn get_seat_num(&self) -> u8;
    fn get_price(&self) -> u32;
    fn get_status(&self) -> &str;
}

impl Car for CarInfo {
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

#[derive(Serialize, Deserialize)]
pub enum FuelType {
    Gasoline,
    Diesel,
    Electric,
    Hybrid,
    LPG,
    Hydrogen,
    Other,
}
#[derive(Serialize, Deserialize)]
pub enum CarStatus {
    Available,
    Rented,
    UnderMaintenance,
    Reserved,
    OutOfService,
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
            FuelType::LPG => "LPG",
            FuelType::Hydrogen => "Hydrogen",
            FuelType::Other => "Other",
        }
    }
}

impl CarStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CarStatus::Available => "Available",
            CarStatus::Rented => "Rented",
            CarStatus::UnderMaintenance => "UnderMaintenance",
            CarStatus::Reserved => "Reserved",
            CarStatus::OutOfService => "OutOfService",
        }
    }
}

pub async fn add_car(pool: &MySqlPool, info: CarInfo) -> Result<String, String> {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
}

// 응답 구조체 (auth.rs 에서 사용)
#[derive(serde::Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}

pub async fn create_user(
    pool: &MySqlPool,
    email: &str,
    password: &str,
) -> Result<MySqlQueryResult, Error> {
    // 비밀번호 해싱
    let hashed_password = hash(password, DEFAULT_COST).expect("Failed to hash password");

    // 데이터베이스에 사용자 추가
    let result = sqlx::query!(
        "INSERT INTO users (email, password) VALUES (?, ?)",
        email,
        hashed_password
    )
    .execute(pool)
    .await?;

    Ok(result)
}

pub async fn find_user_by_email(pool: &MySqlPool, email: &str) -> Result<Option<User>, Error> {
    // 사용자 이름으로 사용자 검색
    let user = sqlx::query_as!(
        User,
        "SELECT id, email, password FROM users WHERE email = ?",
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
