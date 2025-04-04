#[macro_use]
extern crate rocket;

mod auth;

use auth::{login, logout, signup};
use dotenvy::dotenv;
use rocket::fs::NamedFile;
use rocket::serde::json::Json;
use server::{CarInfo, add_car, update_car};
use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use std::path::Path;

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
        .mount(
            "/",
            routes![
                index,
                login_page,
                signup_page,
                list_page,
                add_car_endpoint,
                update_car_endpoint,
                signup,
                login,
                logout
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
