use bcrypt::{DEFAULT_COST, hash, verify as bcrypt_verify};
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlQueryResult;
use sqlx::mysql::MySqlRow;
use sqlx::{Error, MySqlPool};
use sqlx::{FromRow, Row};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CarInfo {
    id: Option<i32>,
    plate_number: String,
    manufacturer: String,
    name: String,
    year: u16,
    car_type: String,
    fuel_type: String,
    transmission: String,
    seat_num: u8,
    color: Option<String>,
    image_url: serde_json::Value,
    car_trim: Option<String>,
    daily_rate: f64,
    location: String,
    rating: f64,
    description: Option<String>,
    status: String,
}

impl CarInfo {
    pub fn new() -> Self {
        CarInfo {
            id: None,
            plate_number: String::new(),
            manufacturer: String::new(),
            name: String::new(),
            year: 0,
            car_type: String::new(),
            fuel_type: String::new(),
            transmission: String::new(),
            seat_num: 0,
            color: None,
            image_url: serde_json::Value::Null,
            car_trim: None,
            daily_rate: 0.0,
            location: String::new(),
            rating: 0.0,
            description: None,
            status: String::new(),
        }
    }
    pub fn id(&self) -> Option<i32> {
        self.id
    }
    pub fn set_plate_number(&mut self, plate_number: String) {
        self.plate_number = plate_number;
    }
    pub fn set_manufacturer(&mut self, manufacturer: String) {
        self.manufacturer = manufacturer;
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn set_year(&mut self, year: u16) {
        self.year = year;
    }
    pub fn set_car_type(&mut self, car_type: String) {
        self.car_type = car_type;
    }
    pub fn set_fuel_type(&mut self, fuel_type: String) {
        self.fuel_type = fuel_type;
    }
    pub fn set_transmission(&mut self, transmission: String) {
        self.transmission = transmission;
    }
    pub fn set_seat_num(&mut self, seat_num: u8) {
        self.seat_num = seat_num;
    }
    pub fn set_color(&mut self, color: Option<String>) {
        self.color = color;
    }
    pub fn set_image_url(&mut self, image_url: serde_json::Value) {
        self.image_url = image_url;
    }
    pub fn set_car_trim(&mut self, car_trim: Option<String>) {
        self.car_trim = car_trim;
    }
    pub fn set_daily_rate(&mut self, daily_rate: f64) {
        self.daily_rate = daily_rate;
    }
    pub fn set_location(&mut self, location: String) {
        self.location = location;
    }
    pub fn set_rating(&mut self, rating: f64) {
        self.rating = rating;
    }
    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }
    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }
    pub fn plate_number(&self) -> &str {
        &self.plate_number
    }
    pub fn manufacturer(&self) -> &str {
        &self.manufacturer
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn year(&self) -> u16 {
        self.year
    }
    pub fn car_type(&self) -> &str {
        &self.car_type
    }
    pub fn fuel_type(&self) -> &str {
        &self.fuel_type
    }
    pub fn transmission(&self) -> &str {
        &self.transmission
    }
    pub fn seat_num(&self) -> u8 {
        self.seat_num
    }
    pub fn color(&self) -> &Option<String> {
        &self.color
    }
    pub fn car_trim(&self) -> &Option<String> {
        &self.car_trim
    }
    pub fn daily_rate(&self) -> f64 {
        self.daily_rate
    }
    pub fn location(&self) -> &str {
        &self.location
    }
    pub fn rating(&self) -> f64 {
        self.rating
    }
    pub fn description(&self) -> &Option<String> {
        &self.description
    }
    pub fn status(&self) -> &str {
        &self.status
    }
}

impl FromRow<'_, MySqlRow> for CarInfo {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        let mut car_info = CarInfo {
            id: row.try_get("id").ok(),
            plate_number: String::new(),
            manufacturer: String::new(),
            name: String::new(),
            year: 0,
            car_type: String::new(),
            fuel_type: String::new(),
            transmission: String::new(),
            seat_num: 0,
            color: None,
            image_url: serde_json::Value::Null,
            car_trim: None,
            daily_rate: 0.0,
            location: String::new(),
            rating: 0.0,
            description: None,
            status: String::new(),
        };

        car_info.set_plate_number(row.try_get("plate_number")?);
        car_info.set_manufacturer(row.try_get("manufacturer")?);
        car_info.set_name(row.try_get("name")?);
        car_info.set_year(row.try_get("year")?);
        car_info.set_car_type(row.try_get("car_type")?);
        car_info.set_fuel_type(row.try_get("fuel_type")?);
        car_info.set_transmission(row.try_get("transmission")?);
        car_info.set_seat_num(row.try_get("seat_num")?);
        car_info.set_color(row.try_get("color").ok());
        car_info.set_image_url(row.try_get("image_url")?);
        car_info.set_car_trim(row.try_get("car_trim").ok());
        car_info.set_daily_rate(row.try_get("daily_rate")?);
        car_info.set_location(row.try_get("location")?);
        car_info.set_rating(row.try_get("rating")?);
        car_info.set_description(row.try_get("description")?);
        car_info.set_status(row.try_get("status")?);
        Ok(car_info)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
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
