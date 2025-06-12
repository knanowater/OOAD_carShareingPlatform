#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_static_resources;

mod auth;
mod background_tasks;
mod controllers;
mod models;
mod payment;
mod repositories;
mod services;

use auth::{api_is_admin, api_login, api_logout, api_signup};
use dotenvy::dotenv;
use payment::process_payment;
use rocket::fs::FileServer;
use rocket::{build, get, routes};
use sqlx::mysql::MySqlPoolOptions;
use std::env;

static_response_handler! {
    "/favicon.ico" => favicon => "favicon",
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect(".env file not found");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    println!("Successfully connected to the database!");

    // GRASP - Controller 패턴
    let rocket_instance = build()
        .attach(static_resources_initializer!("favicon" => "../favicon.ico",))
        .mount(
            "/",
            routes![
                favicon,
                controllers::pages::index_page,
                controllers::pages::login_page,
                controllers::pages::signup_page,
                controllers::pages::list_page,
                controllers::pages::reservation_page,
                controllers::pages::reservation_success_page,
                controllers::pages::mypage_page,
                controllers::pages::mypage_reservations_page,
                controllers::pages::overdue_fee_page,
                controllers::pages::admin_dashboard_page,
                controllers::pages::car_management_page,
                controllers::pages::car_detail_page,
                controllers::pages::host_add_car_page,
                controllers::pages::host_edit_car_page,
                controllers::pages::host_management_page,
                controllers::pages::host_reservations_page,
                controllers::cars::api_get_cars,
                controllers::cars::api_get_car_by_id,
                controllers::cars::api_add_car,
                controllers::cars::api_update_car,
                controllers::cars::api_delete_car,
                controllers::reservations::api_reservation_request,
                controllers::reservations::api_reservations,
                controllers::reservations::api_get_reservation_info_by_reservation_id_payment_id,
                controllers::reservations::api_return_car,
                controllers::reservations::api_cancel_reservation,
                controllers::reservations::api_get_reservation_calendar,
                controllers::reservations::cancel_reservation_due_to_payment_failed,
                controllers::reservations::api_overdue_fee_info,
                controllers::reservations::api_get_host_reservations,
                controllers::reservations::api_accept_reservation,
                controllers::reservations::api_reject_reservation,
                controllers::users::api_mypage,
                // Auth controllers
                api_signup,
                api_login,
                api_logout,
                api_is_admin,
                // Payment controllers
                process_payment,
            ],
        )
        .mount("/scripts", FileServer::from("../client/scripts"))
        .mount("/static", FileServer::from("../server/static"))
        .manage(pool.clone()); // GRASP - Indirection 패턴

    // GRASP - Protected Variations 패턴
    let rocket_with_tasks = background_tasks::start_background_tasks(rocket_instance);

    rocket_with_tasks.launch().await.map_err(|e| {
        eprintln!("Rocket failed to launch: {:?}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    Ok(())
}
