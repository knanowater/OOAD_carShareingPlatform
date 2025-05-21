use crate::auth::AuthToken;
use crate::models::car::{CarForm, CarInfo, CarListResponse, CarQuery};
// GRASP - Low Coupling 패턴: 인터페이스를 통해 구체적인 구현체와 분리
use crate::repositories::car_repository::{CarRepository, MySqlCarRepository};
use rocket::State;
use rocket::form::Form;
use rocket::http::Status;
use rocket::serde::json::{Json, json};
use rocket::{get, post};
use sqlx::MySqlPool;

// GRASP - Controller 패턴: API 엔드포인트가 요청을 받아 적절한 도메인 객체로 위임
#[post("/api/add_car", data = "<form>")]
pub async fn api_add_car(
    mut form: Form<CarForm<'_>>,
    auth_token: AuthToken,
    pool: &State<MySqlPool>,
) -> Result<String, (Status, String)> {
    // GRASP - Creator 패턴: 차량 데이터 관련 객체 생성 책임을 가진 Repository 생성
    let car_repo = MySqlCarRepository::new(pool.inner().clone());

    // 사용자 ID 가져오기
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| (Status::Unauthorized, "Invalid token".into()))?;

    // GRASP - Information Expert 패턴: CarInfo 객체가 자신의 데이터를 관리하는 책임을 가짐
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
    car_info.set_owner(Some(user_id)); // 소유자 ID 설정
    car_info.set_color(Some(form.color.clone()));
    let images = std::mem::take(&mut form.images);
    // GRASP - Indirection 패턴: 컨트롤러가 직접 DB에 접근하지 않고 Repository를 통해 간접적으로 접근
    car_repo.add_car(car_info, images).await.map_err(|e| {
        eprintln!("Error adding car: {:?}", e);
        (Status::InternalServerError, e.to_string())
    })
}

// GRASP - Controller 패턴: 차량 업데이트 요청 처리를 담당
#[post("/api/update_car", data = "<form>")]
pub async fn api_update_car(
    mut form: Form<CarForm<'_>>,
    pool: &State<MySqlPool>,
) -> Result<String, (Status, String)> {
    // GRASP - Pure Fabrication 패턴: Repository가 도메인 객체와 직접 관련이 없지만 역할 분담을 위해 생성됨
    let car_repo = MySqlCarRepository::new(pool.inner().clone());

    // GRASP - Creator 패턴: 차량 정보를 캡슐화하는 객체 생성
    let mut car_info = CarInfo::new();
    car_info.set_id(
        form.id
            .ok_or_else(|| (Status::BadRequest, "차량 ID가 필요합니다".to_string()))?,
    );
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
    car_info.set_deleted_images(form.deleted_images.clone());
    car_info.set_color(Some(form.color.clone()));
    let images = std::mem::take(&mut form.images);
    car_repo.update_car(car_info, images).await.map_err(|e| {
        eprintln!("Error updating car: {:?}", e);
        (Status::InternalServerError, e.to_string())
    })
}

// GRASP - Controller 패턴: 차량 삭제 요청을 처리하는 컨트롤러
#[delete("/api/car/<car_id>")]
pub async fn api_delete_car(
    car_id: i32,
    auth_token: AuthToken,
    pool: &State<MySqlPool>,
) -> Result<String, (Status, String)> {
    let repo = MySqlCarRepository::new(pool.inner().clone());
    // GRASP - Information Expert 패턴: 차량 정보를 조회하는 책임을 Repository에 위임
    let car = repo
        .get_car_by_id(car_id)
        .await
        .map_err(|_| (Status::InternalServerError, "Error retrieving car".into()))?
        .ok_or((
            Status::NotFound,
            format!("Car with ID {} not found", car_id),
        ))?;

    // GRASP - Protected Variations 패턴: 사용자 인증 토큰 검증을 통한 보호
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| (Status::Unauthorized, "Invalid token".into()))?;

    // GRASP - Information Expert 패턴: 차량 소유권 검사 로직
    if car.owner().unwrap() != user_id {
        return Err((
            Status::Forbidden,
            "You do not have permission to delete this car.".into(),
        ));
    }

    // GRASP - Low Coupling 패턴: 컨트롤러는 실제 삭제 로직을 구현하지 않고 Repository에 위임
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

// GRASP - Controller 패턴: 차량 목록 조회 요청 처리
#[get("/api/cars?<query..>")]
pub async fn api_get_cars(
    pool: &State<MySqlPool>,
    query: CarQuery,
) -> Result<Json<CarListResponse>, (Status, String)> {
    // GRASP - Pure Fabrication & Creator 패턴: Repository 객체 생성
    let car_repo = MySqlCarRepository::new(pool.inner().clone());
    // GRASP - Indirection 패턴: 컨트롤러가 직접 DB에 접근하지 않고 Repository를 통해 간접 접근
    car_repo
        .get_cars(query)
        .await
        .map(|response| Json(response))
        .map_err(|e| (Status::InternalServerError, e.to_string()))
}

// GRASP - Controller 패턴: 특정 차량 조회 요청 처리
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
