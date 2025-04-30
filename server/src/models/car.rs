use rocket::form::FromForm;
use rocket::fs::TempFile;
use serde::{Deserialize, Serialize};
use server::CarInfo;

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
    pub sort: Option<String>,
    pub rental_date: Option<String>,
    pub return_date: Option<String>,
    pub min_daily_rate: Option<i32>,
    pub max_daily_rate: Option<i32>,
    pub car_type: Option<String>,
    pub fuel_type: Option<String>,
    pub transmission: Option<String>,
    pub status: Option<String>,
}

#[derive(FromForm)]
pub struct CarForm<'r> {
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
    #[field(name = "images")]
    pub images: Vec<TempFile<'r>>,
}
