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
    image_url: Option<String>,
    car_trim: Option<String>,
    daily_rate: f64,
    location: String,
    rating: f64,
    description: Option<String>,
    status: String,
}

impl CarInfo {
    pub fn id(&self) -> Option<i32> {
        self.id
    }
    fn set_plate_number(&mut self, plate_number: String) {
        self.plate_number = plate_number;
    }
    fn set_manufacturer(&mut self, manufacturer: String) {
        self.manufacturer = manufacturer;
    }
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
    fn set_year(&mut self, year: u16) {
        self.year = year;
    }
    fn set_car_type(&mut self, car_type: String) {
        self.car_type = car_type;
    }
    fn set_fuel_type(&mut self, fuel_type: String) {
        self.fuel_type = fuel_type;
    }
    fn set_transmission(&mut self, transmission: String) {
        self.transmission = transmission;
    }
    fn set_seat_num(&mut self, seat_num: u8) {
        self.seat_num = seat_num;
    }
    fn set_color(&mut self, color: Option<String>) {
        self.color = color;
    }
    fn set_image_url(&mut self, image_url: Option<String>) {
        self.image_url = image_url;
    }
    fn set_car_trim(&mut self, car_trim: Option<String>) {
        self.car_trim = car_trim;
    }
    fn set_daily_rate(&mut self, daily_rate: f64) {
        self.daily_rate = daily_rate;
    }
    fn set_location(&mut self, location: String) {
        self.location = location;
    }
    fn set_rating(&mut self, rating: f64) {
        self.rating = rating;
    }
    fn set_description(&mut self, description: Option<String>) {
        self.description = description;
    }
    fn set_status(&mut self, status: String) {
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
    pub fn image_url(&self) -> &Option<String> {
        &self.image_url
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
            image_url: None,
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
        car_info.set_image_url(row.try_get("image_url").ok());
        car_info.set_car_trim(row.try_get("car_trim").ok());
        car_info.set_daily_rate(row.try_get("daily_rate")?);
        car_info.set_location(row.try_get("location")?);
        car_info.set_rating(row.try_get("rating")?);
        car_info.set_description(row.try_get("description")?);
        car_info.set_status(row.try_get("status")?);
        Ok(car_info)
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
            return Err(format!("'{}' 차량은 이미 존재합니다!", info.plate_number));
        }
        Ok(None) => {
            // plate_number가 존재하지 않으므로 차량 추가 진행
            let query = r#"
                INSERT INTO cars (plate_number, manufacturer, name, year, car_type, fuel_type, transmission, seat_num, color, image_url, car_trim, daily_rate, location, rating, description, status)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#;

            let result = sqlx::query(query)
                .bind(&info.plate_number)
                .bind(&info.manufacturer)
                .bind(&info.name)
                .bind(&info.year)
                .bind(&info.car_type)
                .bind(&info.fuel_type)
                .bind(&info.transmission)
                .bind(&info.seat_num)
                .bind(&info.color)
                .bind(&info.image_url)
                .bind(&info.car_trim)
                .bind(&info.daily_rate)
                .bind(&info.location)
                .bind(&info.rating)
                .bind(&info.description)
                .bind(&info.status)
                .execute(pool)
                .await;

            match result {
                Ok(_) => Ok(format!("'{}' 이(가) 성공적으로 추가되었습니다!", info.name)),
                Err(e) => Err(format!("차량 추가에 실패했습니다 ! : {}", e)),
            }
        }
        Err(e) => Err(format!("DB에러! : {}", e)),
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
                SET manufacturer = ?, name = ?, year = ?, car_type = ?, fuel_type = ?, transmission = ?, seat_num = ?, daily_rate = ?, rating = ?, status = ?, image_url = ?
                WHERE plate_number = ?
            "#;

            let result = sqlx::query(query)
                .bind(&info.plate_number)
                .bind(&info.manufacturer)
                .bind(&info.name)
                .bind(&info.year)
                .bind(&info.car_type)
                .bind(&info.fuel_type)
                .bind(&info.transmission)
                .bind(&info.seat_num)
                .bind(&info.color)
                .bind(&info.image_url)
                .bind(&info.car_trim)
                .bind(&info.daily_rate)
                .bind(&info.location)
                .bind(&info.rating)
                .bind(&info.description)
                .bind(&info.status)
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
