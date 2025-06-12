use crate::models::reservation::*;
use crate::repositories::reservation_repository::{
    MySqlReservationRepository, ReservationRepository,
};
use rocket::http::Status;
use rocket::serde::json::Json;
use sqlx::MySqlPool;

pub struct ReservationService {
    pool: MySqlPool,
}

impl ReservationService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    // GRASP - Information Expert 패턴: 예약 생성 비즈니스 로직
    pub async fn create_reservation(
        &self,
        user_id: i32,
        reservation_data: CreateReservationRequest,
    ) -> Result<String, (Status, String)> {
        let reservation_repository = MySqlReservationRepository::new(&self.pool);
        reservation_repository
            .create_reservation(user_id, reservation_data)
            .await
    }

    // GRASP - Information Expert 패턴: 결제 실패로 인한 예약 취소
    pub async fn cancel_due_to_payment_failure(
        &self,
        payment_id: String,
        user_id: i32,
    ) -> Result<Status, Status> {
        let reservation_repository = MySqlReservationRepository::new(&self.pool);
        reservation_repository
            .cancel_due_to_payment_failure(payment_id, user_id)
            .await
    }

    // GRASP - Information Expert 패턴: 차량 반납 비즈니스 로직
    pub async fn return_car(
        &self,
        user_id: i32,
        reservation_id: String,
    ) -> Result<Json<ReturnApiResponse>, Status> {
        let reservation_repository = MySqlReservationRepository::new(&self.pool);
        reservation_repository
            .return_car(user_id, reservation_id)
            .await
    }

    // GRASP - Information Expert 패턴: 사용자 예약 목록 조회
    pub async fn get_user_reservations(
        &self,
        user_id: i32,
        page: Option<u64>,
        limit: Option<u64>,
        status: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
        car_type: Option<String>,
    ) -> Result<Json<ReservationsResponse>, Status> {
        let reservation_repository = MySqlReservationRepository::new(&self.pool);
        reservation_repository
            .get_user_reservations(user_id, page, limit, status, start_date, end_date, car_type)
            .await
    }

    // GRASP - Information Expert 패턴: 예약 취소 비즈니스 로직
    pub async fn cancel_reservation(
        &self,
        user_id: i32,
        reservation_id: String,
    ) -> Result<Json<ReservationActionResponse>, Status> {
        let reservation_repository = MySqlReservationRepository::new(&self.pool);
        reservation_repository
            .cancel_reservation(user_id, reservation_id)
            .await
    }

    // GRASP - Information Expert 패턴: 연체료 정보 조회
    pub async fn get_overdue_fee_info(
        &self,
        user_id: i32,
        reservation_id: String,
    ) -> Result<Json<OverdueFeeInfo>, Status> {
        let reservation_repository = MySqlReservationRepository::new(&self.pool);
        reservation_repository
            .get_overdue_fee_info(user_id, reservation_id)
            .await
    }

    // GRASP - Information Expert 패턴: 예약 정보 조회
    pub async fn get_reservation_info(
        &self,
        user_id: i32,
        reservation_id: String,
        payment_id: String,
    ) -> Result<Json<ReservationInfo>, Status> {
        let reservation_repository = MySqlReservationRepository::new(&self.pool);
        reservation_repository
            .get_reservation_info_by_reservation_id_payment_id(user_id, reservation_id, payment_id)
            .await
    }

    // GRASP - Information Expert 패턴: 예약 캘린더 조회
    pub async fn get_reservation_calendar(
        &self,
        car_id: i32,
        default_rental_date: MyDate,
    ) -> Result<Json<ReservationCalendar>, Status> {
        let reservation_repository = MySqlReservationRepository::new(&self.pool);
        reservation_repository
            .get_reservation_calendar(car_id, default_rental_date)
            .await
    }

    // GRASP - Information Expert 패턴: 호스트 예약 목록 조회
    pub async fn get_host_reservations(
        &self,
        host_id: i32,
        status: Option<String>,
    ) -> Result<Json<ReservationsResponse>, Status> {
        let reservation_repository = MySqlReservationRepository::new(&self.pool);
        reservation_repository
            .get_host_reservations(host_id, status)
            .await
    }

    // GRASP - Information Expert 패턴: 예약 수락 비즈니스 로직
    pub async fn accept_reservation(
        &self,
        host_id: i32,
        reservation_id: String,
    ) -> Result<ReservationActionResponse, (Status, String)> {
        let reservation_repository = MySqlReservationRepository::new(&self.pool);
        reservation_repository
            .accept_reservation(host_id, reservation_id)
            .await
            .map(|_| ReservationActionResponse {
                message: "예약이 수락되었습니다.".to_string(),
            })
    }

    // GRASP - Information Expert 패턴: 예약 거절 비즈니스 로직
    pub async fn reject_reservation(
        &self,
        host_id: i32,
        reservation_id: String,
    ) -> Result<ReservationActionResponse, (Status, String)> {
        let reservation_repository = MySqlReservationRepository::new(&self.pool);
        reservation_repository
            .reject_reservation(host_id, reservation_id)
            .await
            .map(|_| ReservationActionResponse {
                message: "예약이 거절되었습니다.".to_string(),
            })
    }
}
