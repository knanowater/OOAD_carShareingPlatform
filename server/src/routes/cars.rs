use crate::auth::AuthToken;
use crate::models::car::{CarForm, CarInfo, CarListResponse, CarQuery};
use crate::repositories::car_repository::{CarRepository, MySqlCarRepository};
use rocket::State;
use rocket::form::Form;
use rocket::http::Status;
use rocket::serde::json::{Json, json};
use rocket::{get, post};
use sqlx::MySqlPool;

#[post("/api/add_car", data = "<form>")]
pub async fn api_add_car(
    mut form: Form<CarForm<'_>>,
    pool: &State<MySqlPool>,
) -> Result<String, (Status, String)> {
    let car_repo = MySqlCarRepository::new(pool.inner().clone());

    // CarForm -> CarInfo 변환
    let mut car_info = CarInfo::new();
    car_info.set_plate_number(form.plate_number.clone());
    car_info.set_manufacturer(form.manufacturer.clone());
    car_info.set_name(form.name.clone());
    car_info.set_year(form.year);
    car_info.set_car_type(form.car_type.clone());
    car_info.set_fuel_type(form.fuel_type.clone());
    car_info.set_transmission(form.transmission.clone());
    car_info.set_seat_num(form.seat_num);
    car_info.set_car_trim(Some(form.car_trim.clone()));
    car_info.set_daily_rate(form.daily_rate);
    car_info.set_location(form.location.clone());
    car_info.set_rating(0.0);
    car_info.set_description(Some(form.description.clone()));
    car_info.set_status("Available".to_string());

    // images 소유권 이동
    let images = std::mem::take(&mut form.images);

    car_repo.add_car(car_info, images).await.map_err(|e| {
        eprintln!("Error adding car: {:?}", e); // 에러 로그 출력
        (Status::InternalServerError, e.to_string())
    })
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

#[delete("/api/car/<car_id>")]
pub async fn api_delete_car(
    car_id: i32,
    auth_token: AuthToken,
    pool: &State<MySqlPool>,
) -> Result<String, (Status, String)> {
    let repo = MySqlCarRepository::new(pool.inner().clone());
    // 차량 정보 조회
    let car = repo
        .get_car_by_id(car_id)
        .await
        .map_err(|_| (Status::InternalServerError, "Error retrieving car".into()))?
        .ok_or((
            Status::NotFound,
            format!("Car with ID {} not found", car_id),
        ))?;

    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| (Status::Unauthorized, "Invalid token".into()))?;

    if car.owner().unwrap() != user_id {
        return Err((
            Status::Forbidden,
            "You do not have permission to delete this car.".into(),
        ));
    }

    match repo.delete_car(car).await {
        Ok(msg) => Ok(format!(
            "{{\"status\": \"success\", \"message\": \"{}\"}}",
            msg
        )),
        Err(e) => {
            let err_msg = e.to_string();
            let user_msg = if err_msg.contains("차량이 예약중입니다") {
                "차량이 예약중입니다".to_string()
            } else {
                err_msg
            };
            Err((
                Status::InternalServerError,
                json!({"message": user_msg}).to_string(),
            ))
        }
    }
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
