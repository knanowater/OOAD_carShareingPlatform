#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

// 모듈 선언
mod auth;
mod background_tasks;
mod models;
mod payment;
mod repositories;
mod routes;
mod utils;

// 필요한 모듈 및 함수 임포트
use dotenvy::dotenv;
use rocket::fs::FileServer;
use rocket::{build, get, routes};
use sqlx::mysql::MySqlPoolOptions;
use std::env;
// 기존 모듈 임포트
use auth::{api_is_admin, api_login, api_logout, api_signup}; // AuthToken은 routes에서 사용
use payment::process_payment;

// 정적 리소스 핸들러 (main.rs에 두거나 별도 모듈로 분리 가능)
static_response_handler! {
    "/favicon.ico" => favicon => "favicon",
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 에러 타입 변경 (sqlx::Error + rocket::Error)
    dotenv().expect(".env file not found");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let pool = MySqlPoolOptions::new()
        .max_connections(5) // 설정값 조정 가능
        .connect(&database_url)
        .await?;
    println!("Successfully connected to the database!");

    let rocket_instance =
        build() // rocket::build() 대신 build() 사용
            .attach(static_resources_initializer!("favicon" => "../favicon.ico",))
            .mount(
                "/",
                routes![
                    favicon, // favicon 핸들러
                    // Page routes from routes::pages
                    routes::pages::index_page,
                    routes::pages::login_page,
                    routes::pages::signup_page,
                    routes::pages::list_page,
                    routes::pages::reservation_page,
                    routes::pages::reservation_success_page,
                    routes::pages::mypage_page,
                    routes::pages::mypage_reservations_page,
                    routes::pages::overdue_fee_page,
                    routes::pages::admin_dashboard_page,
                    routes::pages::car_management_page,
                    routes::pages::car_detail_page,
                    routes::pages::host_add_car_page,
                    routes::pages::host_management_page,
                    routes::pages::host_reservations_page,
                    // API routes from various modules
                    routes::cars::api_get_cars,
                    routes::cars::api_get_car_by_id,
                    routes::cars::api_add_car,
                    routes::cars::api_update_car,
                    routes::cars::api_delete_car,
                    routes::reservations::api_reservation_request,
                    routes::reservations::api_reservations,
                    routes::reservations::api_get_reservation_info_by_reservation_id_payment_id,
                    routes::reservations::api_return_car,
                    routes::reservations::api_cancel_reservation,
                    routes::reservations::api_get_reservation_calendar,
                    routes::reservations::cancel_reservation_due_to_payment_failed,
                    routes::reservations::api_overdue_fee_info,
                    routes::reservations::api_get_host_reservations,
                    routes::users::api_mypage,
                    // Auth routes (from auth module)
                    api_signup,
                    api_login,
                    api_logout,
                    api_is_admin,
                    // Payment routes (from payment module)
                    process_payment,
                ],
            )
            .mount(
                "/scripts",                            // 정적 파일 경로 확인
                FileServer::from("../client/scripts"), // FileServer 사용
            )
            .mount("/static", FileServer::from("../server/static")) // 정적 파일 경로 확인
            .manage(pool.clone()); // DB Pool 관리

    // 백그라운드 작업 시작
    let rocket_with_tasks = background_tasks::start_background_tasks(rocket_instance);

    // Rocket 실행
    rocket_with_tasks.launch().await.map_err(|e| {
        eprintln!("Rocket failed to launch: {:?}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    Ok(())
}
