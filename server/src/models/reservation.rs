use chrono::{NaiveDate, NaiveDateTime};
use rocket::serde::{Deserialize, Serialize}; // rocket의 Serialize/Deserialize 사용
use sqlx::FromRow;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateReservationRequest {
    pub car_id: i32,
    pub rental_date: String, // 날짜 형식 고려 (NaiveDateTime or String)
    pub return_date: String, // 날짜 형식 고려 (NaiveDateTime or String)
    pub request: String,
    pub total_price: f32,
}

#[derive(Serialize, FromRow)]
#[serde(crate = "rocket::serde")]
pub struct ReservationDetails {
    pub reservation_id: String,
    pub car_image_url: String,
    pub car_manufacturer: String,
    pub car_model: String,
    pub rental_date: NaiveDateTime,
    pub return_date: NaiveDateTime,
    pub rental_period_days: i32,
    pub pickup_location: String,
    pub total_price: f64, // DB 타입과 일치 확인 (DECIMAL -> f64 or Decimal type)
    pub reservation_status: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ReservationsResponse {
    pub reservations: Vec<ReservationDetails>,
    pub total_pages: u64,
}

// 이름 변경: Request -> ReservationActionRequest 또는 유사한 이름
#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ReservationActionRequest {
    pub reservation_id: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CancelApiResponse {
    pub message: String,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ReturnApiResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overdue_fee: Option<i32>,
}

#[derive(Serialize, FromRow)] // DB에서 직접 읽어올 경우 FromRow 필요
#[serde(crate = "rocket::serde")]
pub struct OverdueFeeInfo {
    pub reservation_id: String,
    pub car_info: String, // 조합된 문자열
    pub rental_date: NaiveDate,
    pub expected_return_date: NaiveDate,
    pub actual_return_date: Option<NaiveDate>,
    pub base_fee: i32,
    pub overdue_hours: i64,
    pub total_overdue_fee: i32,
}
