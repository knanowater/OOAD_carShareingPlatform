#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

mod auth;

use auth::{login, logout, signup};
use dotenvy::dotenv;
use rocket::State;
use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use serde::Serialize;
use server::{CarInfo, add_car, update_car};
use sqlx::FromRow;
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
                add_car_endpoint,
                update_car_endpoint,
                signup,
                login,
                logout,
                get_cars
            ],
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

#[post("/add_car", format = "json", data = "<car_info>")]
async fn add_car_endpoint(car_info: Json<CarInfo>, pool: &rocket::State<MySqlPool>) -> String {
    match add_car(pool, car_info.into_inner()).await {
        Ok(message) => message,
        Err(error) => error,
    }
}

#[post("/update_car", format = "json", data = "<car_info>")]
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

#[get("/api/cars?<start>")]
pub async fn get_cars(pool: &State<MySqlPool>, start: Option<usize>) -> Json<CarListResponse> {
    let start_index = start.unwrap_or(0); // 시작 번호가 없으면 0부터 시작
    let limit = 6;

    let total_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM cars")
        .fetch_one(pool.inner())
        .await;

    let cars_result = sqlx::query_as::<_, CarInfo>(
        "SELECT plate_number, manufacture, name, year, fuel_type, transmission, seat_num, daily_rate, status, connected_with, image_url FROM cars LIMIT ? OFFSET ?"
    )
    .bind(limit as u64) // LIMIT는 u64 타입이어야 할 수 있습니다.
    .bind(start_index as u64) // OFFSET도 u64 타입이어야 할 수 있습니다.
    .fetch_all(pool.inner())
    .await;

    match (total_result, cars_result) {
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
