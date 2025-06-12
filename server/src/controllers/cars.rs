use crate::auth::AuthToken;
use crate::models::car::{CarForm, CarQuery};
use crate::services::car_service::CarService;
use rocket::State;
use rocket::form::Form;
use rocket::http::Status;
use rocket::serde::json::{Json, json};
use rocket::{delete, get, post};
use sqlx::MySqlPool;

// GRASP - Controller 패턴: HTTP 요청/응답만 처리, 비즈니스 로직은 서비스에 위임
#[post("/api/add_car", data = "<form>")]
pub async fn api_add_car(
    form: Form<CarForm<'_>>,
    auth_token: AuthToken,
    pool: &State<MySqlPool>,
) -> Result<String, (Status, String)> {
    // 사용자 ID 추출
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| (Status::Unauthorized, "Invalid token".into()))?;

    // GRASP - Low Coupling: 서비스 레이어를 통한 비즈니스 로직 처리
    let car_service = CarService::new(pool.inner().clone());
    car_service.create_car(form, user_id).await.map_err(|e| {
        eprintln!("Error adding car: {:?}", e);
        (Status::InternalServerError, e)
    })
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[post("/api/update_car", data = "<form>")]
pub async fn api_update_car(
    form: Form<CarForm<'_>>,
    pool: &State<MySqlPool>,
) -> Result<String, (Status, String)> {
    let car_service = CarService::new(pool.inner().clone());
    car_service.update_car(form).await.map_err(|e| {
        eprintln!("Error updating car: {:?}", e);
        (Status::InternalServerError, e)
    })
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[delete("/api/car/<car_id>")]
pub async fn api_delete_car(
    car_id: i32,
    auth_token: AuthToken,
    pool: &State<MySqlPool>,
) -> Result<String, (Status, String)> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| (Status::Unauthorized, "Invalid token".into()))?;

    let car_service = CarService::new(pool.inner().clone());
    car_service
        .delete_car(car_id, user_id)
        .await
        .map_err(|(status, message)| {
            if message.contains("차량이 예약중입니다") {
                (status, json!({"message": message}).to_string())
            } else {
                (status, message)
            }
        })
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 응답 변환
#[get("/api/cars?<query..>")]
pub async fn api_get_cars(
    pool: &State<MySqlPool>,
    query: CarQuery,
) -> Result<Json<crate::models::car::CarListResponse>, (Status, String)> {
    let car_service = CarService::new(pool.inner().clone());
    car_service
        .get_cars(query)
        .await
        .map(Json)
        .map_err(|e| (Status::InternalServerError, e))
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 응답 변환
#[get("/api/cars/<id>")]
pub async fn api_get_car_by_id(
    pool: &State<MySqlPool>,
    id: i32,
) -> Result<Json<crate::models::car::CarInfo>, (Status, String)> {
    let car_service = CarService::new(pool.inner().clone());
    car_service
        .get_car_by_id(id)
        .await
        .map_err(|e| (Status::InternalServerError, e))
        .and_then(|car_opt| {
            car_opt
                .map(Json)
                .ok_or((Status::NotFound, format!("Car with ID {} not found", id)))
        })
}
