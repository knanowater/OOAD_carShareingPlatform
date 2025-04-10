#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

mod auth;

use auth::{is_admin, login, logout, signup};
use dotenvy::dotenv;
use rocket::State;
use rocket::form::FromForm;
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::serde::json::Json;
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
    // .env 파일에서 환경 변수 로드
    dotenv().expect(".env file not found");

    // 환경 변수에서 데이터베이스 URL 읽기
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");

    // 데이터베이스 연결 풀 생성
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    println!("Successfully connected to the database!");

    // Rocket 서버 실행
    rocket::build()
        .attach(static_resources_initializer!("favicon" => "../favicon.ico",))
        .mount(
            "/",
            routes![
                favicon,
                index,
                login_page,
                signup_page,
                list_page,
                admin_dashboard,
                car_management,
                add_car_endpoint,
                update_car_endpoint,
                signup,
                login,
                logout,
                is_admin,
                get_cars,
                get_car_by_id,
            ],
        )
        .mount(
            "/scripts",
            rocket::fs::FileServer::from("../client/scripts"),
        )
        .manage(pool)
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
        "SELECT id, plate_number, manufacture, name, year, car_type, fuel_type, transmission, seat_num, daily_rate, rating, status, connected_with, image_url FROM cars {} {} LIMIT ? OFFSET ?",
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
    let sql = "SELECT id, plate_number, manufacture, name, year, car_type, fuel_type, transmission, seat_num, daily_rate, rating, description, status, connected_with, image_url FROM cars WHERE id = ?";

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

#[get("/admin")]
pub async fn admin_dashboard() -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/admin/dashboard.html"))
        .await
        .ok()
}

#[get("/admin/vehicles")]
pub async fn car_management() -> Option<NamedFile> {
    NamedFile::open(Path::new("../client/admin/vehicles.html"))
        .await
        .ok()
}
