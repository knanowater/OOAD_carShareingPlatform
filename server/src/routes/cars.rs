use crate::models::car::{CarListResponse, CarQuery};
use crate::repositories::car_repository::{CarRepository, MySqlCarRepository};
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use server::CarInfo;
use sqlx::MySqlPool;

#[post("/api/add_car", format = "json", data = "<car_info>")]
pub async fn api_add_car(
    car_info: Json<CarInfo>,
    pool: &State<MySqlPool>,
) -> Result<String, (Status, String)> {
    let car_repo = MySqlCarRepository::new(pool.inner().clone());
    car_repo
        .add_car(car_info.into_inner())
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))
}

#[post("/api/update_car", format = "json", data = "<car_info>")]
pub async fn api_update_car(
    car_info: Json<CarInfo>,
    pool: &State<MySqlPool>,
) -> Result<String, (Status, String)> {
    let car_repo = MySqlCarRepository::new(pool.inner().clone());
    car_repo
        .update_car(car_info.into_inner())
        .await
        .map_err(|e| (Status::InternalServerError, e.to_string()))
}

#[get("/api/cars?<query..>")]
pub async fn api_get_cars(
    pool: &State<MySqlPool>,
    query: CarQuery,
) -> Result<Json<CarListResponse>, (Status, String)> {
    let car_repo = MySqlCarRepository::new(pool.inner().clone());
    car_repo
        .get_cars(query)
        .await
        .map(|response| Json(response))
        .map_err(|e| (Status::InternalServerError, e.to_string()))
}

#[get("/api/cars/<id>")]
pub async fn api_get_car_by_id(
    pool: &State<MySqlPool>,
    id: i32,
) -> Result<Json<CarInfo>, (Status, String)> {
    let car_repo = MySqlCarRepository::new(pool.inner().clone());
    car_repo
        .get_car_by_id(id)
        .await
        .map(|car_opt| {
            car_opt
                .map(Json)
                .ok_or((Status::NotFound, format!("Car with ID {} not found", id)))
        })
        .map_err(|e| (Status::InternalServerError, e.to_string()))
        .and_then(std::convert::identity) // Result<Result<T, E>, E> 를 Result<T, E> 로 변환
}
