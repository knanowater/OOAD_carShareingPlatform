use rocket::form::FromForm;
use rocket::fs::TempFile;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, Row};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")] // Json 응답을 위해 필요할 수 있음
pub struct CarListResponse {
    pub total: usize,
    pub cars: Vec<CarInfo>, // server::CarInfo 사용
}

#[derive(FromForm, Deserialize)]
#[serde(crate = "rocket::serde")] // 쿼리 파라미터 디시리얼라이즈를 위해 필요할 수 있음
pub struct CarQuery {
    pub start: Option<usize>,
    pub limit: Option<usize>,
    pub sort: Option<String>,
    pub rental_date: Option<String>,
    pub return_date: Option<String>,
    pub min_daily_rate: Option<i32>,
    pub max_daily_rate: Option<i32>,
    pub car_type: Option<String>,
    pub fuel_type: Option<String>,
    pub transmission: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<i32>,
}

#[derive(FromForm)]
pub struct CarForm<'r> {
    pub id: Option<i32>,
    pub plate_number: String,
    pub manufacturer: String,
    pub name: String,
    pub year: u16,
    pub car_type: String,
    pub fuel_type: String,
    pub transmission: String,
    pub seat_num: u8,
    pub car_trim: String,
    pub daily_rate: f64,
    pub location: String,
    pub description: String,
    pub deleted_images: Option<String>,
    #[field(name = "images")]
    pub images: Vec<TempFile<'r>>,
}

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
    owner: Option<i32>,
    deleted_images: Option<String>,
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
            owner: None,
            deleted_images: None,
        }
    }

    pub fn id(&self) -> Option<i32> {
        self.id
    }

    pub fn set_id(&mut self, id: i32) {
        self.id = Some(id);
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
    pub fn set_owner(&mut self, owner: Option<i32>) {
        self.owner = owner;
    }
    pub fn set_deleted_images(&mut self, deleted_images: Option<String>) {
        self.deleted_images = deleted_images;
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
    pub fn owner(&self) -> &Option<i32> {
        &self.owner
    }
    pub fn deleted_images(&self) -> Option<&str> {
        self.deleted_images.as_deref()
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
            owner: None,
            deleted_images: None,
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
        car_info.set_owner(row.try_get("owner").ok());
        car_info.set_deleted_images(row.try_get("deleted_images").ok());
        Ok(car_info)
    }
}
