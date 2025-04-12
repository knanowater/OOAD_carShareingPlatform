#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

mod auth;
mod background_tasks;
mod payment;

use auth::{AuthToken, is_admin, login, logout, signup};
use chrono::{NaiveDateTime, Utc};
use dotenvy::dotenv;
use payment::process_payment;
use rocket::State;
use rocket::form::FromForm;
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Rocket, get, routes};
use serde::{Deserialize, Serialize};
use server::{CarInfo, add_car, update_car};
use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use std::path::Path;
static_response_handler! {
    "/favicon.ico" => favicon => "favicon",
}
#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().expect(".env file not found");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Successfully connected to the database!");

    let rocket = rocket::build()
        .attach(static_resources_initializer!("favicon" => "../favicon.ico",))
        .mount(
            "/",
            routes![
                favicon,
                index,
                login_page,
                signup_page,
                list_page,
                reservation_page,
                reservation_success_page,
                mypage_page,
                mypage_reservations_page,
                overdue_fee_page,
                admin_dashboard_page,
                car_management_page,
                add_car_endpoint,
                update_car_endpoint,
                signup,
                login,
                logout,
                is_admin,
                get_cars,
                get_car_by_id,
                reservation_request,
                api_mypage,
                api_reservations,
                api_return_car,
                api_cancel_reservation,
                api_overdue_fee_info,
                process_payment,
            ],
        )
        .mount(
            "/scripts",
            rocket::fs::FileServer::from("../client/scripts"),
        )
        .manage(pool.clone());

    let rocket_with_tasks: Rocket<rocket::Build> = background_tasks::start_background_tasks(rocket);

    rocket_with_tasks
        .launch()
        .await
        .expect("Rocket failed to launch");

    Ok(())
}
#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/index.html"))
        .await
        .ok()
}

#[get("/login")]
async fn login_page() -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/login.html"))
        .await
        .ok()
}

#[get("/signup")]
async fn signup_page() -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/signup.html"))
        .await
        .ok()
}

#[get("/list")]
async fn list_page() -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/list.html")).await.ok()
}

#[get("/reservation?<id>")]
async fn reservation_page(id: Option<i32>) -> Option<NamedFile> {
    if let Some(id) = id {
        println!("Reservation page requested with id: {:?}", id);
        NamedFile::open(Path::new("../client/reservation.html"))
            .await
            .ok()
    } else {
        None
    }
}

#[get("/overdue_fee")]
async fn overdue_fee_page() -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/mypage/overdue_fee.html"))
        .await
        .ok()
}

#[get("/reservation/success?<id>")]
async fn reservation_success_page(id: Option<i32>) -> Option<NamedFile> {
    if let Some(id) = id {
        println!("Reservation success page requested with id: {:?}", id);
        NamedFile::open(Path::new("../client/reservation-success.html"))
            .await
            .ok()
    } else {
        None
    }
}

#[post("/api/add_car", format = "json", data = "<car_info>")]
async fn add_car_endpoint(car_info: Json<CarInfo>, pool: &rocket::State<MySqlPool>) -> String {
    match add_car(pool, car_info.into_inner()).await {
        Ok(message) => message,
        Err(error) => error,
    }
}

#[post("/api/update_car", format = "json", data = "<car_info>")]
async fn update_car_endpoint(car_info: Json<CarInfo>, pool: &rocket::State<MySqlPool>) -> String {
    match update_car(pool, car_info.into_inner()).await {
        Ok(message) => message,
        Err(error) => error,
    }
}

#[derive(Serialize)]
pub struct CarListResponse {
    total: usize,
    cars: Vec<CarInfo>,
}

#[derive(FromForm, Deserialize)]
pub struct CarQuery {
    start: Option<usize>,
    sort: Option<String>,
    min_daily_rate: Option<i32>,
    max_daily_rate: Option<i32>,
    car_type: Option<String>,
    fuel_type: Option<String>,
    transmission: Option<String>,
    status: Option<String>,
}

#[get("/api/cars?<query..>")]
pub async fn get_cars(pool: &State<MySqlPool>, query: CarQuery) -> Json<CarListResponse> {
    let start_index = query.start.unwrap_or(0);
    let limit = 6;
    let sort_by = query.sort.unwrap_or_else(|| "plate_number ASC".to_string());
    let min_price = query.min_daily_rate;
    let max_price = query.max_daily_rate;
    let car_types: Option<Vec<String>> = query
        .car_type
        .map(|s| s.split(',').map(String::from).collect());
    let fuel_types: Option<Vec<String>> = query
        .fuel_type
        .map(|s| s.split(',').map(String::from).collect());
    let transmissions: Option<Vec<String>> = query
        .transmission
        .map(|s| s.split(',').map(String::from).collect());
    let status: Option<String> = query.status;

    let mut where_clauses = Vec::new();
    let mut query_params: Vec<String> = Vec::new(); // 구체적인 String 타입 사용

    if let Some(min) = min_price {
        where_clauses.push("daily_rate >= ?".to_string());
        query_params.push(min.to_string());
    }
    if let Some(max) = max_price {
        where_clauses.push("daily_rate <= ?".to_string());
        query_params.push(max.to_string());
    }

    if let Some(types) = &car_types {
        if !types.is_empty() {
            let placeholders = vec!["?"; types.len()].join(",");
            where_clauses.push(format!("car_type IN ({})", placeholders));
            query_params.extend(types.iter().cloned());
        }
    }
    if let Some(fuels) = &fuel_types {
        if !fuels.is_empty() {
            let placeholders = vec!["?"; fuels.len()].join(",");
            where_clauses.push(format!("fuel_type IN ({})", placeholders));
            query_params.extend(fuels.iter().cloned());
        }
    }
    if let Some(trans) = &transmissions {
        if !trans.is_empty() {
            let placeholders = vec!["?"; trans.len()].join(",");
            where_clauses.push(format!("transmission IN ({})", placeholders));
            query_params.extend(trans.iter().cloned());
        }
    }

    if let Some(status) = &status {
        where_clauses.push(format!("status IN (?)",)); // ? 플레이스홀더 사용
        query_params.push(status.clone());
    }

    let where_clause = if !where_clauses.is_empty() {
        format!("WHERE {}", where_clauses.join(" AND "))
    } else {
        String::new()
    };

    let count_sql = format!("SELECT COUNT(*) FROM cars {}", where_clause);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for param in &query_params {
        count_query = count_query.bind(param);
    }
    let count_result = count_query.fetch_one(pool.inner()).await;

    let order_by_clause = match sort_by.as_str() {
        "daily_rate_asc" => "ORDER BY daily_rate ASC",
        "daily_rate_desc" => "ORDER BY daily_rate DESC",
        "rating_desc" => "ORDER BY rating DESC",
        "rating_asc" => "ORDER BY rating ASC",
        _ => "ORDER BY name ASC",
    };

    let sql = format!(
        "SELECT id, plate_number, manufacture, name, year, car_type, fuel_type, transmission, seat_num, daily_rate, location, rating, description, status, connected_with, image_url FROM cars {} {} LIMIT ? OFFSET ?",
        where_clause, order_by_clause
    );

    let mut cars_query = sqlx::query_as::<_, CarInfo>(&sql);
    for param in &query_params {
        cars_query = cars_query.bind(param);
    }
    cars_query = cars_query.bind(limit as u64).bind(start_index as u64);
    let cars_result = cars_query.fetch_all(pool.inner()).await;

    match (count_result, cars_result) {
        (Ok(total), Ok(cars)) => Json(CarListResponse {
            total: total as usize,
            cars,
        }),
        (Err(e), _) => {
            eprintln!("Error fetching car count: {}", e);
            Json(CarListResponse {
                total: 0,
                cars: Vec::new(),
            })
        }
        (_, Err(e)) => {
            eprintln!("Error fetching cars: {}", e);
            Json(CarListResponse {
                total: 0,
                cars: Vec::new(),
            })
        }
    }
}

#[get("/api/cars/<id>")]
pub async fn get_car_by_id(
    pool: &State<MySqlPool>,
    id: i32,
) -> Result<Json<CarInfo>, (Status, String)> {
    let sql = "SELECT id, plate_number, manufacture, name, year, car_type, fuel_type, transmission, seat_num, daily_rate, location, rating, description, status, connected_with, image_url FROM cars WHERE id = ?";

    let car_result = sqlx::query_as::<_, CarInfo>(sql)
        .bind(id)
        .fetch_optional(pool.inner())
        .await;

    match car_result {
        Ok(Some(car)) => Ok(Json(car)),
        Ok(None) => Err((Status::NotFound, format!("Car with ID {} not found", id))), // Corrected
        Err(e) => {
            eprintln!("Error fetching car with ID {}: {}", id, e);
            Err((
                Status::InternalServerError,
                "Failed to fetch car".to_string(),
            )) // Corrected
        }
    }
}

#[post("/api/reservations/request", data = "<reservation_data>")]
pub async fn reservation_request(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    reservation_data: Json<ReservationData>,
) -> Result<Json<serde_json::Value>, (Status, String)> {
    let user_id = auth_token.0.sub.parse::<i32>().map_err(|_| {
        eprintln!("Failed to parse user ID from token");
        (
            Status::Unauthorized,
            "토큰에서 사용자 ID를 파싱하는 데 실패했습니다.".into(),
        )
    })?;

    let mut conn = pool.acquire().await.map_err(|e| {
        eprintln!("Failed to acquire connection from pool: {}", e);
        (
            Status::InternalServerError,
            format!("데이터베이스 연결 오류: {}", e),
        )
    })?;

    let data = reservation_data.into_inner();

    let car_status: Option<String> = sqlx::query_scalar("SELECT status FROM cars WHERE id = ?")
        .bind(data.car_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| {
            eprintln!("Failed to fetch car status: {}", e);
            (
                Status::InternalServerError,
                format!("차량 상태를 확인하는 데 실패했습니다: {}", e),
            )
        })?;

    match car_status {
        Some(status) if status.to_lowercase() != "available" => {
            return Err((
                Status::BadRequest,
                "선택한 차량은 현재 예약 불가능합니다.".into(),
            ));
        }
        None => {
            return Err((Status::NotFound, "선택한 차량을 찾을 수 없습니다.".into()));
        }
        Some(_) => {
            let reservation_timestamp = chrono::Local::now().naive_local();
            let result = sqlx::query!(
                r#"
                INSERT INTO reservation (user_id, car_id, reservation_timestamp, rental_date, return_date, total_price, request, reservation_status )
                VALUES (?, ?, ?, ?, ?, ?, ?,'pending')
                "#,
                user_id,
                data.car_id,
                reservation_timestamp,
                data.rental_date,
                data.return_date,
                data.total_price,
                data.request
            )
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                eprintln!("Failed to insert reservation: {}", e);
                (
                    Status::InternalServerError,
                    format!("예약 요청에 실패했습니다: {}", e),
                )
            })?;

            let reservation_id = result.last_insert_id();
            // 예약 성공 시 차량 상태를 'Unavailable'로 업데이트
            let update_car_result = sqlx::query!(
                "UPDATE cars SET status = 'Unavailable' WHERE id = ?",
                data.car_id
            )
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                eprintln!("Failed to update car status: {}", e);
                // 차량 상태 업데이트 실패는 심각한 오류이므로 롤백 또는 추가 로깅 고려
                (
                    Status::InternalServerError,
                    format!("차량 상태를 업데이트하는 데 실패했습니다: {}", e),
                )
            })?;

            if update_car_result.rows_affected() > 0 {
                Ok(Json(
                    serde_json::json!({ "reservation_id": reservation_id }),
                ))
            } else {
                // 차량 상태 업데이트가 반영되지 않았을 경우 (드문 경우)
                eprintln!(
                    "Warning: Car status update not applied for car ID {}",
                    data.car_id
                );
                Ok(Json(
                    serde_json::json!({ "reservation_id": reservation_id }),
                ))
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ReservationData {
    car_id: i32,
    rental_date: String,
    return_date: String,
    request: String,
    total_price: f32,
}

#[get("/admin")]
pub async fn admin_dashboard_page() -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/admin/dashboard.html"))
        .await
        .ok()
}

#[get("/admin/vehicles")]
pub async fn car_management_page() -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/admin/vehicles.html"))
        .await
        .ok()
}

#[get("/mypage")]
pub async fn mypage_page() -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/mypage/mypage.html"))
        .await
        .ok()
}

#[get("/mypage/reservations")]
pub async fn mypage_reservations_page() -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/mypage/reservations.html"))
        .await
        .ok()
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[get("/api/mypage")]
pub async fn api_mypage(
    auth_token: AuthToken,
    pool: &State<MySqlPool>,
) -> Result<Json<User>, Status> {
    let user_id = auth_token.0.sub.parse::<i32>().map_err(|_| {
        eprintln!("Failed to parse user ID from token subject");
        Status::Unauthorized
    })?;

    let mut conn = pool.acquire().await.map_err(|e| {
        eprintln!("Failed to acquire connection from pool: {}", e);
        Status::InternalServerError
    })?;

    let user = sqlx::query_as::<_, User>("SELECT id, name, email, role FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Status::NotFound,
            _ => Status::InternalServerError,
        })?;

    Ok(Json(user))
}

#[derive(Serialize)]
pub struct ReservationInfo {
    reservation_id: i32,
    car_image_url: String,
    car_manufacture: String,
    car_model: String,
    rental_date: NaiveDateTime,
    return_date: NaiveDateTime,
    rental_period_days: i64,
    pickup_location: String,
    total_price: f32,
    reservation_status: String,
}

#[get("/api/reservations")]
pub async fn api_reservations(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
) -> Result<Json<Vec<ReservationInfo>>, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;

    let mut conn = pool.acquire().await.map_err(|e| {
        eprintln!("Failed to acquire connection: {}", e);
        Status::InternalServerError
    })?;

    let reservations = sqlx::query!(
        r#"
        SELECT
            r.id AS reservation_id,
            c.image_url AS car_image_url,
            c.manufacture AS car_manufacture,
            c.name AS car_model,
            r.rental_date,
            r.return_date,
            DATEDIFF(r.return_date, r.rental_date) AS rental_period_days,
            COALESCE(c.location, '미정') AS pickup_location, -- NULL인 경우 '미정'으로 처리
            COALESCE(r.total_price, 0) AS total_price,             -- NULL인 경우 0으로 처리
            r.reservation_status
        FROM reservation r
        JOIN cars c ON r.car_id = c.id
        WHERE r.user_id = ?
        ORDER BY r.id DESC
        "#,
        user_id
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch reservations: {}", e);
        Status::InternalServerError
    })?
    .into_iter()
    .map(|row| ReservationInfo {
        reservation_id: row.reservation_id,
        car_image_url: row.car_image_url.unwrap_or_default(),
        car_manufacture: row.car_manufacture,
        car_model: row.car_model,
        rental_date: row.rental_date,
        return_date: row.return_date,
        rental_period_days: row.rental_period_days.unwrap_or_default(), // Option<i64> -> i64 (None이면 0)
        pickup_location: row.pickup_location,
        total_price: row.total_price,
        reservation_status: row.reservation_status,
    })
    .collect();

    Ok(Json(reservations))
}

#[derive(Deserialize)]
struct Request {
    reservation_id: i32,
}

#[derive(Serialize)]
struct CancelApiResponse {
    message: String,
}

#[derive(Serialize)]
struct ReturnApiResponse {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    overdue_fee: Option<i32>, // 연체료 필드 (없으면 직렬화에서 제외)
}

#[post("/api/return", data = "<return_request>")]
async fn api_return_car(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    return_request: Json<Request>,
) -> Result<Json<ReturnApiResponse>, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;
    let reservation_id = return_request.into_inner().reservation_id;

    // 데이터베이스 연결 가져오기
    let mut conn = pool.acquire().await.map_err(|e| {
        eprintln!("Failed to acquire connection: {}", e);
        Status::InternalServerError
    })?;

    // 예약 정보 확인 및 사용자 ID 일치 여부 확인, car_id, 반납 예정 시간 가져오기
    let reservation_result = sqlx::query!(
        "SELECT user_id, car_id, return_date FROM reservation WHERE id = ?",
        reservation_id
    )
    .fetch_optional(&mut *conn) // fetch_optional 사용
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch reservation: {}", e);
        Status::InternalServerError
    })?;

    let reservation_info = match reservation_result {
        Some(info) => info,
        None => return Err(Status::NotFound),
    };

    let owner_user_id = reservation_info.user_id;
    let car_id = reservation_info.car_id;
    let return_deadline: Option<NaiveDateTime> = Some(reservation_info.return_date); // 명시적 타입 지정

    if user_id != owner_user_id {
        return Err(Status::Forbidden); // 예약한 사용자와 반납 요청자가 다르면 403 Forbidden
    }

    let now = Utc::now().naive_utc(); // 현재 시간을 UTC naive datetime으로 가져옴

    let mut overdue_fee = 0;
    if let Some(deadline) = return_deadline {
        if now > deadline {
            let overdue_duration = now - deadline;
            let overdue_hours = overdue_duration.num_hours();

            let car_info_result = sqlx::query!("SELECT daily_rate FROM cars WHERE id = ?", car_id)
                .fetch_optional(&mut *conn) // fetch_optional 사용
                .await
                .map_err(|e| {
                    eprintln!("Failed to fetch car info: {}", e);
                    Status::InternalServerError
                })?;

            if let Some(car) = car_info_result {
                let daily_rate = car.daily_rate;
                let hourly_rate = daily_rate as f64 / 24.0;
                overdue_fee = (hourly_rate * 1.5 * overdue_hours as f64).round() as i32;
            }
        }
    }

    // 예약 상태 업데이트 (연체료와 실제 반납 시간 포함)
    let update_reservation_result = sqlx::query!(
         "UPDATE reservation SET reservation_status = ?, return_date_actual = ?, overdue_fee = ? WHERE id = ?",
         if overdue_fee > 0 { "overdue" } else { "completed" }, // 연체료 있으면 'overdue' 상태로 변경
         now, // 실제 반납 시간 저장
         overdue_fee,
         reservation_id
     )
     .execute(&mut *conn)
     .await
     .map_err(|e| {
         eprintln!("Failed to update reservation status and return time: {}", e);
         Status::InternalServerError
     })?;

    // 차량 반납 성공 후 차량 상태를 'Available'로 업데이트 (결과 확인 없이 시도)
    let update_car_result =
        sqlx::query!("UPDATE cars SET status = 'Available' WHERE id = ?", car_id)
            .execute(&mut *conn)
            .await;

    if let Err(e) = update_car_result {
        eprintln!(
            "Warning: Failed to update car status for car ID {}: {}",
            car_id, e
        );
    }

    if update_reservation_result.rows_affected() > 0 {
        let message = if overdue_fee > 0 {
            "차량이 연체되었습니다. 연체료 결제 페이지로 이동합니다.".to_string()
        } else {
            "차량이 성공적으로 반납 처리되었습니다.".to_string()
        };
        Ok(Json(ReturnApiResponse {
            message,
            overdue_fee: Some(overdue_fee),
        }))
    } else {
        Err(Status::InternalServerError) // 예약 업데이트 실패
    }
}

#[post("/api/cancel", data = "<cancel_request>")]
async fn api_cancel_reservation(
    pool: &State<MySqlPool>,
    auth_token: auth::AuthToken,
    cancel_request: Json<Request>,
) -> Result<Json<CancelApiResponse>, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;
    let reservation_id = cancel_request.into_inner().reservation_id;

    // 데이터베이스 연결 가져오기
    let mut conn = pool.acquire().await.map_err(|e| {
        eprintln!("Failed to acquire connection: {}", e);
        Status::InternalServerError
    })?;

    // 예약 정보 확인 및 사용자 ID 일치 여부 확인과 car_id 가져오기
    let reservation_info = sqlx::query!(
        "SELECT user_id, car_id FROM reservation WHERE id = ?",
        reservation_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch reservation: {}", e);
        Status::NotFound // 예약 정보가 없으면 404 Not Found
    })?;

    let owner_user_id = reservation_info.user_id;
    let car_id = reservation_info.car_id;

    if user_id != owner_user_id {
        return Err(Status::Forbidden); // 예약한 사용자와 취소 요청자가 다르면 403 Forbidden
    }

    let update_reservation_result = sqlx::query!(
        "UPDATE reservation SET reservation_status = 'canceled' WHERE id = ?",
        reservation_id
    )
    .execute(&mut *conn)
    .await
    .map_err(|e| {
        eprintln!("Failed to update reservation status: {}", e);
        Status::InternalServerError
    })?;

    if update_reservation_result.rows_affected() > 0 {
        // 예약 취소 성공 후 차량 상태를 'Available'로 업데이트 (결과 확인 없이 시도)
        let update_car_result =
            sqlx::query!("UPDATE cars SET status = 'Available' WHERE id = ?", car_id)
                .execute(&mut *conn)
                .await; // .map_err를 제거하여 오류를 Result로 처리

        if let Err(e) = update_car_result {
            eprintln!(
                "Warning: Failed to update car status for car ID {}: {}",
                car_id, e
            );
        }

        Ok(Json(CancelApiResponse {
            message: "예약이 성공적으로 취소되었습니다.".to_string(),
        }))
    } else {
        Err(Status::InternalServerError) // 예약 업데이트 실패
    }
}

#[get("/api/overdue_fee_info/<reservation_id>")]
async fn api_overdue_fee_info(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    reservation_id: i32,
) -> Result<Json<OverdueFeeInfo>, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;

    let mut conn = pool.acquire().await.map_err(|e| {
        eprintln!("Failed to acquire connection: {}", e);
        Status::InternalServerError
    })?;

    let reservation = sqlx::query!(
        r#"
        SELECT
            r.id AS reservation_id,
            c.manufacture AS car_manufacture,
            c.name AS car_model,
            r.rental_date,
            r.return_date,
            r.return_date_actual,
            r.overdue_fee,
            c.daily_rate -- 차량의 일일 요금을 가져옴
        FROM reservation r
        JOIN cars c ON r.car_id = c.id
        WHERE r.id = ? AND r.user_id = ? AND r.reservation_status = 'overdue'
        "#,
        reservation_id,
        user_id
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| {
        eprintln!("Failed to fetch overdue reservation info: {}", e);
        Status::InternalServerError
    })?;

    match reservation {
        Some(res) => {
            let rental_date = res.rental_date.and_utc();
            let return_date = res.return_date.and_utc();
            let return_date_actual = res.return_date_actual.map(|dt| dt.and_utc());

            let overdue_hours = return_date_actual
                .and_then(|actual| Some((actual - return_date).num_hours()))
                .unwrap_or(0);

            // 기본 연체료 계산 (시간당 요금 * 1.5, 정수 처리)
            let hourly_rate = (res.daily_rate as f64 / 24.0).round() as i32;
            let base_fee = (hourly_rate as f64 * 1.5).round() as i32;

            Ok(Json(OverdueFeeInfo {
                reservation_id: res.reservation_id,
                car_info: format!("{} {}", res.car_manufacture, res.car_model),
                rental_date: rental_date.naive_utc().date(), // 날짜만 유지
                expected_return_date: return_date.naive_utc().date(), // 날짜만 유지
                actual_return_date: return_date_actual.map(|dt| dt.naive_utc().date()), // 날짜만 유지
                base_fee,
                overdue_hours,
                total_overdue_fee: res.overdue_fee.unwrap_or(0) as i32,
            }))
        }
        None => Err(Status::NotFound),
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct OverdueFeeInfo {
    reservation_id: i32,
    car_info: String,
    rental_date: chrono::NaiveDate,
    expected_return_date: chrono::NaiveDate,
    actual_return_date: Option<chrono::NaiveDate>,
    base_fee: i32,
    overdue_hours: i64,
    total_overdue_fee: i32,
}
