use std::i32;

use crate::auth::AuthToken;
use crate::models::reservation::ReservationQuery;
use crate::models::reservation::*;
use crate::services::reservation_service::ReservationService;
use rocket::http::Status;
use rocket::serde::json::{Json, json};
use rocket::{State, delete, get, post};
use sqlx::MySqlPool;

// GRASP - Controller 패턴: HTTP 요청/응답만 처리, 비즈니스 로직은 서비스에 위임
#[post("/api/reservations/request", data = "<reservation_data>")]
pub async fn api_reservation_request(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    reservation_data: Json<CreateReservationRequest>,
) -> Result<Json<serde_json::Value>, (Status, String)> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| (Status::Unauthorized, "Invalid token".into()))?;

    // GRASP - Low Coupling: 서비스 레이어를 통한 비즈니스 로직 처리
    let reservation_service = ReservationService::new(pool.inner().clone());
    let reservation_id = reservation_service
        .create_reservation(user_id, reservation_data.into_inner())
        .await
        .map_err(|(status, msg)| (status, msg))?;

    Ok(Json(json!({ "reservation_id": reservation_id })))
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[delete("/api/reservations/cancel/<id>")]
pub async fn cancel_reservation_due_to_payment_failed(
    id: &str,
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
) -> Result<Status, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;

    let reservation_service = ReservationService::new(pool.inner().clone());
    reservation_service
        .cancel_due_to_payment_failure(id.to_string(), user_id)
        .await
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[post("/api/return", data = "<return_request>")]
pub async fn api_return_car(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    return_request: Json<ReservationActionRequest>,
) -> Result<Json<ReturnApiResponse>, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;

    let reservation_service = ReservationService::new(pool.inner().clone());
    reservation_service
        .return_car(user_id, return_request.into_inner().reservation_id)
        .await
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[get("/api/reservations?<page>&<limit>&<status>&<start_date>&<end_date>&<car_type>")]
pub async fn api_reservations(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    page: Option<u64>,
    limit: Option<u64>,
    status: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    car_type: Option<String>,
) -> Result<Json<ReservationsResponse>, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;

    let reservation_service = ReservationService::new(pool.inner().clone());
    reservation_service
        .get_user_reservations(user_id, page, limit, status, start_date, end_date, car_type)
        .await
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[post("/api/cancel", data = "<cancel_request>")]
pub async fn api_cancel_reservation(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    cancel_request: Json<ReservationActionRequest>,
) -> Result<Json<ReservationActionResponse>, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;

    let reservation_service = ReservationService::new(pool.inner().clone());
    reservation_service
        .cancel_reservation(user_id, cancel_request.into_inner().reservation_id)
        .await
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[get("/api/overdue_fee_info/<reservation_id>")]
pub async fn api_overdue_fee_info(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    reservation_id: String,
) -> Result<Json<OverdueFeeInfo>, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;

    let reservation_service = ReservationService::new(pool.inner().clone());
    reservation_service
        .get_overdue_fee_info(user_id, reservation_id)
        .await
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[get("/api/reservation?<reservation_payment_query..>")]
pub async fn api_get_reservation_info_by_reservation_id_payment_id(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    reservation_payment_query: ReservationQuery,
) -> Result<Json<ReservationInfo>, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;

    let reservation_service = ReservationService::new(pool.inner().clone());
    reservation_service
        .get_reservation_info(
            user_id,
            reservation_payment_query.reservation_id.clone(),
            reservation_payment_query.payment_id.clone(),
        )
        .await
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[get("/api/reservation/calendar?<car_id>&<default_rental_date>")]
pub async fn api_get_reservation_calendar(
    pool: &State<MySqlPool>,
    car_id: i32,
    default_rental_date: MyDate,
) -> Result<Json<ReservationCalendar>, Status> {
    let reservation_service = ReservationService::new(pool.inner().clone());
    reservation_service
        .get_reservation_calendar(car_id, default_rental_date)
        .await
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[get("/api/host/reservations?<status>")]
pub async fn api_get_host_reservations(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    status: Option<String>,
) -> Result<Json<ReservationsResponse>, Status> {
    let host_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;

    let reservation_service = ReservationService::new(pool.inner().clone());
    reservation_service
        .get_host_reservations(host_id, status)
        .await
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[post("/api/host/reservations/<reservation_id>/accept")]
pub async fn api_accept_reservation(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    reservation_id: &str,
) -> Result<Json<ReservationActionResponse>, (Status, String)> {
    let host_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|e| (Status::Unauthorized, e.to_string()))?;

    let reservation_service = ReservationService::new(pool.inner().clone());
    let response = reservation_service
        .accept_reservation(host_id, reservation_id.to_string())
        .await
        .map_err(|(status, message)| (status, message))?;

    Ok(Json(response))
}

// GRASP - Controller 패턴: HTTP 요청 처리 및 서비스 호출
#[post("/api/host/reservations/<reservation_id>/reject")]
pub async fn api_reject_reservation(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    reservation_id: &str,
) -> Result<Json<ReservationActionResponse>, (Status, String)> {
    let host_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|e| (Status::Unauthorized, e.to_string()))?;

    let reservation_service = ReservationService::new(pool.inner().clone());
    let response = reservation_service
        .reject_reservation(host_id, reservation_id.to_string())
        .await
        .map_err(|(status, message)| (status, message))?;

    Ok(Json(response))
}
