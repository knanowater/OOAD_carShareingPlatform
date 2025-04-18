use crate::auth::AuthToken;
use crate::models::reservation::ReservationQuery;
use crate::models::reservation::*;
use crate::repositories::reservation_repository::ReservationRepository;
use rocket::http::Status;
use rocket::serde::json::{Json, json};
use rocket::{State, delete, get, post};
use sqlx::MySqlPool;

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
    let repo = ReservationRepository::new(pool);
    let reservation_id = repo
        .create_reservation(user_id, reservation_data.into_inner())
        .await?;
    Ok(Json(json!({ "reservation_id": reservation_id })))
}

#[delete("/api/reservations/cancel/<id>")]
pub async fn cancel_reservation_due_to_payment_failed(
    id: String,
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
) -> Result<Status, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;
    let repo = ReservationRepository::new(pool);
    repo.cancel_due_to_payment_failure(id, user_id).await
}

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
    let repo = ReservationRepository::new(pool);
    repo.return_car(user_id, return_request.into_inner().reservation_id)
        .await
}

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
    let repo = ReservationRepository::new(pool);
    repo.get_user_reservations(user_id, page, limit, status, start_date, end_date, car_type)
        .await
}

#[post("/api/cancel", data = "<cancel_request>")]
pub async fn api_cancel_reservation(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    cancel_request: Json<ReservationActionRequest>,
) -> Result<Json<CancelApiResponse>, Status> {
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;
    let repo = ReservationRepository::new(pool);
    repo.cancel_reservation(user_id, cancel_request.into_inner().reservation_id)
        .await
}

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
    let repo = ReservationRepository::new(pool);
    repo.get_overdue_fee_info(user_id, reservation_id).await
}

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
    let repo = ReservationRepository::new(pool);
    repo.get_reservation_info_by_reservation_id_payment_id(
        user_id,
        reservation_payment_query.reservation_id.clone(),
        reservation_payment_query.payment_id.clone(),
    )
    .await
}
