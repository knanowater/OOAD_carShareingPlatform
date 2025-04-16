use rocket::form::FromForm;
use serde::{Deserialize, Serialize};
use server::CarInfo; // server 모듈/크레이트의 CarInfo 사용

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
    pub min_daily_rate: Option<i32>,
    pub max_daily_rate: Option<i32>,
    pub car_type: Option<String>,
    pub fuel_type: Option<String>,
    pub transmission: Option<String>,
    pub status: Option<String>,
}

// 참고: server::CarInfo 구조체도 여기에 정의하거나,
// server 모듈을 그대로 사용한다면 이 파일에는 CarListResponse, CarQuery만 둡니다.
// 만약 server::CarInfo를 여기로 옮긴다면 server 모듈의 의존성을 제거해야 합니다.
