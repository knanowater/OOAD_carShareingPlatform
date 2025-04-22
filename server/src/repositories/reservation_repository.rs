use crate::models::reservation::*;
use crate::utils::generate_reservation_id;
use chrono::{Datelike, Duration, Local, Utc};
use rocket::http::Status;
use rocket::serde::json::Json;
use sqlx::{Acquire, MySqlPool};

pub struct ReservationRepository<'a> {
    pub pool: &'a MySqlPool,
}

impl<'a> ReservationRepository<'a> {
    pub fn new(pool: &'a MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_reservation(
        &self,
        user_id: i32,
        data: CreateReservationRequest,
    ) -> Result<String, (Status, String)> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| (Status::InternalServerError, format!("DB 연결 실패: {}", e)))?;

        let mut tx = conn.begin().await.map_err(|e| {
            (
                Status::InternalServerError,
                format!("트랜잭션 시작 실패: {}", e),
            )
        })?;

        let car_status_result =
            sqlx::query_scalar::<_, String>("SELECT status FROM cars WHERE id = ? FOR UPDATE")
                .bind(data.car_id)
                .fetch_optional(&mut *tx)
                .await;
        let car_status = match car_status_result {
            Ok(status) => status,
            Err(e) => {
                tx.rollback().await.ok();
                return Err((
                    Status::InternalServerError,
                    format!("차량 상태 조회 실패: {}", e),
                ));
            }
        };
        match car_status {
            Some(status) if status.to_lowercase() != "available" => {
                tx.rollback().await.ok();
                return Err((Status::BadRequest, "차량 사용 불가".into()));
            }
            None => {
                tx.rollback().await.ok();
                return Err((Status::NotFound, "차량 없음".into()));
            }
            _ => {}
        }

        let reservation_id = generate_reservation_id();
        let now = Local::now().naive_local();

        let insert_res = sqlx::query!(
            r#"
            INSERT INTO reservation 
            (user_id, car_id, reservation_timestamp, rental_date, return_date, total_price, request, reservation_status, reservation_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', ?)
            "#,
            user_id,
            data.car_id,
            now,
            data.rental_date,
            data.return_date,
            data.total_price,
            data.request,
            reservation_id,
        )
        .execute(&mut *tx)
        .await;

        let insert_log_res = sqlx::query!(
            r#"
            INSERT INTO reservation_log 
            (user_id, car_id, reservation_timestamp, rental_date, return_date, total_price, request, reservation_status, reservation_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', ?)
            "#,
            user_id,
            data.car_id,
            now,
            data.rental_date,
            data.return_date,
            data.total_price,
            data.request,
            reservation_id,
        )
        .execute(&mut *tx)
        .await;

        if let Err(e) = insert_res {
            tx.rollback().await.ok();
            return Err((Status::InternalServerError, format!("예약 실패: {}", e)));
        }
        if let Err(e) = insert_log_res {
            tx.rollback().await.ok();
            return Err((
                Status::InternalServerError,
                format!("로그 저장 실패: {}", e),
            ));
        }
        let update_res = sqlx::query!(
            "UPDATE cars SET status = 'Unavailable' WHERE id = ?",
            data.car_id
        )
        .execute(&mut *tx)
        .await;

        if let Err(e) = update_res {
            tx.rollback().await.ok();
            return Err((
                Status::InternalServerError,
                format!("차 상태 업데이트 실패: {}", e),
            ));
        }

        match update_res {
            Ok(result) if result.rows_affected() > 0 => {
                tx.commit()
                    .await
                    .map_err(|e| (Status::InternalServerError, format!("커밋 실패: {}", e)))?;
                Ok(reservation_id)
            }
            _ => {
                tx.rollback().await.ok();
                Err((
                    Status::InternalServerError,
                    "차량 상태 업데이트 실패".into(),
                ))
            }
        }
    }
    pub async fn cancel_due_to_payment_failure(
        &self,
        reservation_id: String,
        user_id: i32,
    ) -> Result<Status, Status> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|_| Status::InternalServerError)?;
        let mut tx = conn
            .begin()
            .await
            .map_err(|_| Status::InternalServerError)?;

        let info = sqlx::query!(
            "SELECT user_id, car_id FROM reservation WHERE reservation_id = ?",
            reservation_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| Status::InternalServerError)?;

        match info {
            Some(info) => {
                if info.user_id != user_id {
                    tx.rollback().await.ok();
                    return Err(Status::Forbidden);
                }

                let delete_res = sqlx::query!(
                    "DELETE FROM reservation WHERE reservation_id = ?",
                    reservation_id
                )
                .execute(&mut *tx)
                .await;

                match delete_res {
                    Ok(_) => {}
                    Err(_) => {
                        tx.rollback().await.ok();
                        return Err(Status::InternalServerError);
                    }
                }

                let delete_log_res = sqlx::query!(
                    "DELETE FROM reservation_log WHERE reservation_id = ?",
                    reservation_id
                )
                .execute(&mut *tx)
                .await;

                match delete_log_res {
                    Ok(_) => {}
                    Err(_) => {
                        tx.rollback().await.ok();
                        return Err(Status::InternalServerError);
                    }
                }

                let update_res = sqlx::query!(
                    "UPDATE cars SET status = 'Available' WHERE id = ?",
                    info.car_id
                )
                .execute(&mut *tx)
                .await;

                match update_res {
                    Ok(_) => {}
                    Err(_) => {
                        tx.rollback().await.ok();
                        return Err(Status::InternalServerError);
                    }
                }

                tx.commit().await.map_err(|_| Status::InternalServerError)?;
                Ok(Status::Ok)
            }
            None => {
                tx.rollback().await.ok();
                Err(Status::NotFound)
            }
        }
    }

    pub async fn return_car(
        &self,
        user_id: i32,
        reservation_id: String,
    ) -> Result<Json<ReturnApiResponse>, Status> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|_| Status::InternalServerError)?;
        let mut tx = conn
            .begin()
            .await
            .map_err(|_| Status::InternalServerError)?;

        let reservation = sqlx::query!(
            "SELECT user_id, car_id, return_date FROM reservation WHERE reservation_id = ?",
            reservation_id
        )
        .fetch_optional(&mut *tx)
        .await;

        let reservation = match reservation {
            Ok(Some(reservation)) => reservation,
            Ok(None) => {
                // 롤백 후 NotFound 반환
                tx.rollback().await.ok();
                return Err(Status::NotFound);
            }
            Err(_) => {
                // 롤백 후 InternalServerError 반환
                tx.rollback().await.ok();
                return Err(Status::InternalServerError);
            }
        };

        if reservation.user_id != user_id {
            tx.rollback().await.ok();
            return Err(Status::Forbidden);
        }

        let now = Utc::now().naive_utc();
        let mut overdue_fee = 0;

        if now > reservation.return_date {
            let overdue_hours = (now - reservation.return_date).num_hours();

            if let Some(car) = sqlx::query!(
                "SELECT daily_rate FROM cars WHERE id = ?",
                reservation.car_id
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| Status::InternalServerError)?
            {
                let hourly_rate = car.daily_rate as f64 / 24.0;
                overdue_fee = (hourly_rate * 1.5 * overdue_hours as f64).round() as i32;
            }
        }

        sqlx::query!(
            "UPDATE reservation SET reservation_status = ?, return_date_actual = ?, overdue_fee = ? WHERE reservation_id = ?",
            if overdue_fee > 0 { "overdue" } else { "completed" },
            now,
            overdue_fee,
            reservation_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| Status::InternalServerError)?;

        sqlx::query!(
            "UPDATE cars SET status = 'Available' WHERE id = ?",
            reservation.car_id
        )
        .execute(&mut *tx)
        .await
        .ok(); // optional rollback

        tx.commit().await.map_err(|_| Status::InternalServerError)?;

        let message = if overdue_fee > 0 {
            "차량이 연체되었습니다. 연체료 결제 페이지로 이동합니다.".to_string()
        } else {
            "차량이 성공적으로 반납 처리되었습니다.".to_string()
        };

        Ok(Json(ReturnApiResponse {
            message,
            overdue_fee: if overdue_fee > 0 {
                Some(overdue_fee)
            } else {
                None
            },
        }))
    }

    pub async fn cancel_reservation(
        &self,
        user_id: i32,
        reservation_id: String,
    ) -> Result<Json<CancelApiResponse>, Status> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|_| Status::InternalServerError)?;
        let mut tx = conn
            .begin()
            .await
            .map_err(|_| Status::InternalServerError)?;

        let reservation_result = sqlx::query!(
            "SELECT user_id, car_id FROM reservation WHERE reservation_id = ?",
            reservation_id
        )
        .fetch_optional(&mut *tx)
        .await;

        let reservation = match reservation_result {
            Ok(Some(reservation)) => reservation,
            Ok(None) => {
                // 롤백 후 NotFound 반환
                tx.rollback().await.ok();
                return Err(Status::NotFound);
            }
            Err(_) => {
                // 롤백 후 InternalServerError 반환
                tx.rollback().await.ok();
                return Err(Status::InternalServerError);
            }
        };

        if reservation.user_id != user_id {
            tx.rollback().await.ok();
            return Err(Status::Forbidden);
        }

        sqlx::query!(
            "UPDATE reservation_log SET reservation_status = 'canceled' WHERE reservation_id = ?",
            reservation_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| Status::InternalServerError)?;

        // reservation에서 삭제
        sqlx::query!(
            "DELETE FROM reservation WHERE reservation_id = ?",
            reservation_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| Status::InternalServerError)?;

        sqlx::query!(
            "UPDATE cars SET status = 'Available' WHERE id = ?",
            reservation.car_id
        )
        .execute(&mut *tx)
        .await
        .ok();

        tx.commit().await.map_err(|_| Status::InternalServerError)?;

        Ok(Json(CancelApiResponse {
            message: "예약이 성공적으로 취소되었습니다.".to_string(),
        }))
    }

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
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|_| Status::InternalServerError)?;

        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(10);
        let offset = (page - 1) * limit;

        let mut where_clause = "WHERE r.user_id = ?".to_string();
        let mut params: Vec<String> = vec![user_id.to_string()];

        if let Some(s) = status {
            if s != "all" {
                where_clause.push_str(" AND r.reservation_status = ?");
                params.push(s);
            }
        }
        if let Some(start) = start_date {
            where_clause.push_str(" AND r.rental_date >= ?");
            params.push(start);
        }
        if let Some(end) = end_date {
            where_clause.push_str(" AND r.return_date <= ?");
            params.push(end);
        }
        if let Some(car) = car_type {
            if !car.is_empty() {
                where_clause.push_str(" AND c.car_type = ?");
                params.push(car);
            }
        }

        let count_query = format!(
            "SELECT COUNT(*) FROM reservation r JOIN cars c ON r.car_id = c.id {}",
            where_clause
        );
        let mut query = sqlx::query_scalar::<_, i64>(&count_query);
        for p in &params {
            query = query.bind(p);
        }
        let total_items = query
            .fetch_one(&mut *conn)
            .await
            .map_err(|_| Status::InternalServerError)?;
        let total_pages = (total_items as f64 / limit as f64).ceil() as u64;

        let select_query = format!(
            r#"
            SELECT r.reservation_id, c.image_url AS car_image_url, c.manufacturer AS car_manufacturer,
                c.name AS car_model, r.rental_date, r.return_date,
                DATEDIFF(r.return_date, r.rental_date) AS rental_period_days,
                COALESCE(c.location, '미정') AS pickup_location,
                COALESCE(r.total_price, 0) AS total_price,
                r.reservation_status
            FROM reservation r
            JOIN cars c ON r.car_id = c.id
            {}
            ORDER BY r.id DESC
            LIMIT ? OFFSET ?
            "#,
            where_clause
        );

        let mut sel = sqlx::query_as::<_, ReservationDetails>(&select_query);
        for p in &params {
            sel = sel.bind(p);
        }

        let reservations = sel
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&mut *conn)
            .await
            .map_err(|_| Status::InternalServerError)?;

        Ok(Json(ReservationsResponse {
            reservations,
            total_pages,
        }))
    }

    pub async fn get_overdue_fee_info(
        &self,
        user_id: i32,
        reservation_id: String,
    ) -> Result<Json<OverdueFeeInfo>, Status> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|_| Status::InternalServerError)?;

        let res = sqlx::query!(
            r#"
            SELECT r.reservation_id, c.manufacturer AS car_manufacturer, c.name AS car_model,
                r.rental_date, r.return_date, r.return_date_actual,
                r.overdue_fee, c.daily_rate
            FROM reservation r
            JOIN cars c ON r.car_id = c.id
            WHERE r.reservation_id = ? AND r.user_id = ? AND r.reservation_status = 'overdue'
            "#,
            reservation_id,
            user_id
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(|_| Status::InternalServerError)?;

        if let Some(res) = res {
            let overdue_hours = res
                .return_date_actual
                .map(|actual| (actual - res.return_date).num_hours())
                .unwrap_or(0);
            let hourly_rate = (res.daily_rate as f64 / 24.0).round() as i32;
            let base_fee = (hourly_rate as f64 * 1.5).round() as i32;

            Ok(Json(OverdueFeeInfo {
                reservation_id: res.reservation_id.unwrap(),
                car_info: format!("{} {}", res.car_manufacturer, res.car_model),
                rental_date: res.rental_date.date(),
                expected_return_date: res.return_date.date(),
                actual_return_date: res.return_date_actual.map(|dt| dt.date()),
                base_fee,
                overdue_hours,
                total_overdue_fee: res.overdue_fee.unwrap_or(0) as i32,
            }))
        } else {
            Err(Status::NotFound)
        }
    }

    pub async fn get_reservation_info_by_reservation_id_payment_id(
        &self,
        user_id: i32,
        reservation_id: String,
        payment_id: String,
    ) -> Result<Json<ReservationInfo>, Status> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|_| Status::InternalServerError)?;

        let reservation = sqlx::query_as!(
            ReservationInfo,
            r#"
            SELECT 
                r.car_id,
                CAST(c.year AS UNSIGNED) as year, -- 강제 형 변환
                c.manufacturer,
                c.name,
                c.image_url,
                r.rental_date,
                r.return_date,
                c.daily_rate,
                r.total_price,
                r.request,
                p.payment_date,
                p.reservation_id as payment_reservation_id,
                p.payment_method,
                p.amount,
                r.reservation_status
            FROM reservation r
            JOIN payment p ON r.reservation_id = p.reservation_id
            JOIN cars c ON r.car_id = c.id
            WHERE r.reservation_id = ? AND p.transaction_id = ? AND r.user_id = ?
            "#,
            reservation_id,
            payment_id,
            user_id
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| {
            eprintln!("Error fetching reservation info: {}", e);
            rocket::http::Status::InternalServerError
        })?;

        match reservation {
            Some(reservation) => Ok(rocket::serde::json::Json(reservation)),
            None => Err(rocket::http::Status::NotFound),
        }
    }

    pub async fn get_reservation_calendar(
        &self,
        car_id: i32,
        default_rental_date: MyDate,
        default_return_date: MyDate,
    ) -> Result<Json<ReservationCalendar>, Status> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|_| Status::InternalServerError)?;

        let start_date = default_rental_date.0;
        let end_date = default_return_date.0;

        let reservations = sqlx::query!(
            r#"
            SELECT rental_date, return_date 
            FROM reservation
            WHERE car_id = ? AND rental_date < ? AND return_date >= ?
            "#,
            car_id,
            end_date,
            start_date,
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(|_| Status::InternalServerError)?;

        let mut reserved_days = vec![];

        for res in reservations {
            let mut day = res.rental_date;
            while day < res.return_date {
                if day.month() == start_date.month() {
                    reserved_days.push(day.day() as u8);
                }
                day += Duration::days(1);
            }
        }

        reserved_days.sort_unstable();
        reserved_days.dedup();

        Ok(Json(ReservationCalendar { reserved_days }))
    }
}
