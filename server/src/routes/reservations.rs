use crate::auth::AuthToken;
use crate::models::reservation::*;
use crate::utils::generate_reservation_id;
use chrono::{Local, NaiveDateTime, Utc};
use rocket::http::Status;
use rocket::serde::json::{Json, json};
use rocket::{State, delete, get, post};
use sqlx::{Acquire, MySqlPool};

#[post("/api/reservations/request", data = "<reservation_data>")]
pub async fn api_reservation_request(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    reservation_data: Json<CreateReservationRequest>,
) -> Result<Json<serde_json::Value>, (Status, String)> {
    // 기존 main.rs의 api_reservation_request 로직 전체 이동
    let user_id = auth_token.0.sub.parse::<i32>().map_err(|_| {
        eprintln!("Failed to parse user ID from token");
        (
            Status::Unauthorized,
            "토큰에서 사용자 ID를 파싱하는 데 실패했습니다.".into(),
        )
    })?;

    let conn_result = pool.acquire().await;
    let mut conn = match conn_result {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to acquire connection from pool: {}", e);
            return Err((
                Status::InternalServerError,
                format!("데이터베이스 연결 오류: {}", e),
            ));
        }
    };

    let data = reservation_data.into_inner();

    // 트랜잭션 시작 (선택 사항이지만 권장)
    let tx_result = conn.begin().await;
    let mut tx = match tx_result {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to begin transaction: {}", e);
            return Err((Status::InternalServerError, "트랜잭션 시작 오류".into()));
        }
    };

    let car_status_result: Result<Option<String>, sqlx::Error> =
        sqlx::query_scalar("SELECT status FROM cars WHERE id = ? FOR UPDATE") // 비관적 락킹
            .bind(data.car_id)
            .fetch_optional(&mut *tx) // 트랜잭션 사용
            .await;

    let car_status = match car_status_result {
        Ok(status) => status,
        Err(e) => {
            eprintln!("Failed to fetch car status: {}", e);
            tx.rollback().await.ok(); // 롤백 시도
            return Err((
                Status::InternalServerError,
                format!("차량 상태를 확인하는 데 실패했습니다: {}", e),
            ));
        }
    };

    match car_status {
        Some(status) if status.to_lowercase() != "available" => {
            tx.rollback().await.ok(); // 롤백 시도
            return Err((
                Status::BadRequest,
                "선택한 차량은 현재 예약 불가능합니다.".into(),
            ));
        }
        None => {
            tx.rollback().await.ok(); // 롤백 시도
            return Err((Status::NotFound, "선택한 차량을 찾을 수 없습니다.".into()));
        }
        Some(_) => {
            // 예약 생성 로직
            let reservation_timestamp = Local::now().naive_local();
            let new_reservation_id = generate_reservation_id();
            let insert_result = sqlx::query!(
                r#"
                INSERT INTO reservation (id, user_id, car_id, reservation_timestamp, rental_date, return_date, total_price, request, reservation_status, reservation_id)
                VALUES (NULL, ?, ?, ?, ?, ?, ?, ?, 'pending', ?)
                "#,
                user_id,
                data.car_id,
                reservation_timestamp,
                data.rental_date, // 타입 변환 필요 시 주의
                data.return_date, // 타입 변환 필요 시 주의
                data.total_price,
                data.request,
                new_reservation_id,
            )
            .execute(&mut *tx) // 트랜잭션 사용
            .await;

            if let Err(e) = insert_result {
                eprintln!("Failed to insert reservation: {}", e);
                tx.rollback().await.ok(); // 롤백 시도
                return Err((
                    Status::InternalServerError,
                    format!("예약 요청에 실패했습니다: {}", e),
                ));
            }

            // 차량 상태 업데이트
            let update_car_result = sqlx::query!(
                "UPDATE cars SET status = 'Unavailable' WHERE id = ?",
                data.car_id
            )
            .execute(&mut *tx) // 트랜잭션 사용
            .await;

            match update_car_result {
                Ok(result) if result.rows_affected() > 0 => {
                    tx.commit().await.map_err(|e| {
                        // 커밋
                        eprintln!("Failed to commit transaction: {}", e);
                        (Status::InternalServerError, "트랜잭션 커밋 오류".into())
                    })?;
                    Ok(Json(json!({ "reservation_id": new_reservation_id })))
                }
                Ok(_) | Err(_) => {
                    tx.rollback().await.ok(); // 롤백 시도
                    eprintln!(
                        "Warning: Car status update not applied for car ID {}",
                        data.car_id
                    );
                    // 이 경우에도 예약 ID를 반환할지, 에러를 반환할지 정책 결정 필요
                    Err((
                        Status::InternalServerError,
                        "차량 상태 업데이트 실패".into(),
                    ))
                }
            }
        }
    }
}

#[delete("/api/reservations/cancel/<id>")]
pub async fn cancel_reservation_due_to_payment_failed(
    id: String,
    pool: &State<MySqlPool>,
    auth_token: AuthToken, // 인증 토큰 추가
) -> Result<Status, Status> {
    // 기존 main.rs의 cancel_reservation_due_to_payment_failed 로직 전체 이동
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;
    let conn_result = pool.acquire().await;
    let mut conn = match conn_result {
        Ok(c) => c,
        Err(_) => return Err(Status::InternalServerError),
    };

    // 트랜잭션 시작
    let tx_result = conn.begin().await;
    let mut tx = match tx_result {
        Ok(t) => t,
        Err(_) => return Err(Status::InternalServerError),
    };

    // 예약 정보 및 소유자 확인
    let reservation_info_result = sqlx::query!(
        "SELECT user_id, car_id FROM reservation WHERE reservation_id = ?",
        id
    )
    .fetch_optional(&mut *tx) // 트랜잭션 사용
    .await;

    let reservation_info = match reservation_info_result {
        Ok(info) => info,
        Err(_) => return Err(Status::InternalServerError),
    };

    match reservation_info {
        Some(info) => {
            if info.user_id == user_id {
                // 예약 삭제
                let delete_reservation_result =
                    sqlx::query!("DELETE FROM reservation WHERE reservation_id = ?", id)
                        .execute(&mut *tx)
                        .await;
                if let Err(_) = delete_reservation_result {
                    tx.rollback().await.ok();
                    return Err(Status::InternalServerError);
                }

                if delete_reservation_result.unwrap().rows_affected() > 0 {
                    // 차량 상태 업데이트
                    let update_car_result = sqlx::query!(
                        "UPDATE cars SET status = 'Available' WHERE id = ?",
                        info.car_id
                    )
                    .execute(&mut *tx)
                    .await;
                    if let Err(_) = update_car_result {
                        tx.rollback().await.ok();
                        return Err(Status::InternalServerError);
                    }

                    // 트랜잭션 커밋
                    let commit_result = tx.commit().await;
                    if let Err(_) = commit_result {
                        return Err(Status::InternalServerError);
                    }

                    if update_car_result.unwrap().rows_affected() == 0 {
                        eprintln!("Warning: Car status not updated for car ID {}", info.car_id);
                    }
                    Ok(Status::Ok)
                } else {
                    tx.rollback().await.ok();
                    Err(Status::NotFound) // 삭제할 예약 없음
                }
            } else {
                tx.rollback().await.ok();
                Err(Status::Forbidden) // 권한 없음
            }
        }
        None => {
            tx.rollback().await.ok();
            Err(Status::NotFound) // 예약 없음
        }
    }
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
    // 기존 main.rs의 api_reservations 로직 전체 이동
    let user_id_str = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?
        .to_string();

    let current_page = page.unwrap_or(1);
    let items_per_page = limit.unwrap_or(10);
    let offset = (current_page - 1) * items_per_page;

    let mut conn = pool.acquire().await.map_err(|e| {
        eprintln!("Failed to acquire connection: {}", e);
        Status::InternalServerError
    })?;

    let mut where_clause = "WHERE r.user_id = ?".to_string();
    let mut conditions: Vec<String> = Vec::new();
    // String으로 변경하여 소유권 문제 해결
    let mut query_params_values: Vec<String> = Vec::new();
    query_params_values.push(user_id_str); // user_id_str 소유권 이동

    if let Some(ref s) = status {
        if s != "all" {
            conditions.push("r.reservation_status = ?".to_string());
            query_params_values.push(s.clone()); // status 소유권 복제
        }
    }

    if let Some(ref start) = start_date {
        conditions.push("r.rental_date >= ?".to_string());
        query_params_values.push(start.clone()); // start_date 소유권 복제
    }

    if let Some(ref end) = end_date {
        conditions.push("r.return_date <= ?".to_string());
        query_params_values.push(end.clone()); // end_date 소유권 복제
    }

    if let Some(ref car) = car_type {
        if !car.is_empty() {
            conditions.push("c.car_type = ?".to_string());
            query_params_values.push(car.clone()); // car_type 소유권 복제
        }
    }

    if !conditions.is_empty() {
        where_clause.push_str(&format!(" AND {}", conditions.join(" AND ")));
    }

    let count_query_str = format!(
        r#"
        SELECT COUNT(*)
        FROM reservation r
        JOIN cars c ON r.car_id = c.id
        {}
        "#,
        where_clause
    );

    // 바인딩 로직 수정
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_query_str);
    for param in &query_params_values {
        count_query = count_query.bind(param);
    }

    let total_items = count_query.fetch_one(&mut *conn).await.map_err(|e| {
        eprintln!("Failed to count reservations: {}", e);
        Status::InternalServerError
    })?;

    let total_pages = (total_items as f64 / items_per_page as f64).ceil() as u64;

    let select_query_str = format!(
        r#"
        SELECT
            r.reservation_id AS reservation_id,
            c.image_url AS car_image_url,
            c.manufacturer AS car_manufacturer,
            c.name AS car_model,
            r.rental_date,
            r.return_date,
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

    // 바인딩 로직 수정
    let mut select_query = sqlx::query_as::<_, ReservationDetails>(&select_query_str);
    for param in &query_params_values {
        select_query = select_query.bind(param);
    }
    let reservations = select_query
        .bind(items_per_page as i64)
        .bind(offset as i64)
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| {
            eprintln!("Failed to fetch paginated reservations: {}", e);
            Status::InternalServerError
        })?;

    Ok(Json(ReservationsResponse {
        reservations,
        total_pages,
    }))
}

#[post("/api/return", data = "<return_request>")]
pub async fn api_return_car(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    return_request: Json<ReservationActionRequest>, // 이름 변경된 구조체 사용
) -> Result<Json<ReturnApiResponse>, Status> {
    // 기존 main.rs의 api_return_car 로직 전체 이동
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;
    let reservation_id_str = return_request.into_inner().reservation_id; // 필드 접근

    let conn_result = pool.acquire().await;
    let mut conn = match conn_result {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to acquire connection: {}", e);
            return Err(Status::InternalServerError);
        }
    };

    // 트랜잭션 시작
    let tx_result = conn.begin().await;
    let mut tx = match tx_result {
        Ok(t) => t,
        Err(_) => return Err(Status::InternalServerError),
    };

    let reservation_result = sqlx::query!(
        "SELECT user_id, car_id, return_date FROM reservation WHERE reservation_id = ?",
        reservation_id_str
    )
    .fetch_optional(&mut *tx) // 트랜잭션 사용
    .await;

    let reservation_info = match reservation_result {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Failed to fetch reservation: {}", e);
            tx.rollback().await.ok();
            return Err(Status::InternalServerError);
        }
    };

    let reservation = match reservation_info {
        Some(info) => info,
        None => {
            tx.rollback().await.ok();
            return Err(Status::NotFound);
        }
    };

    if user_id != reservation.user_id {
        tx.rollback().await.ok();
        return Err(Status::Forbidden);
    }

    let car_id = reservation.car_id;
    let return_deadline: Option<NaiveDateTime> = Some(reservation.return_date); // NaiveDateTime 타입 확인
    let now = Utc::now().naive_utc(); // NaiveDateTime으로 통일

    let mut overdue_fee = 0;
    if let Some(deadline) = return_deadline {
        if now > deadline {
            let overdue_duration = now - deadline;
            let overdue_hours = overdue_duration.num_hours();

            let car_info_result = sqlx::query!("SELECT daily_rate FROM cars WHERE id = ?", car_id)
                .fetch_optional(&mut *tx) // 트랜잭션 사용
                .await;

            let car_info = match car_info_result {
                Ok(info) => info,
                Err(e) => {
                    eprintln!("Failed to fetch car info: {}", e);
                    tx.rollback().await.ok();
                    return Err(Status::InternalServerError);
                }
            };

            if let Some(car) = car_info {
                let daily_rate = car.daily_rate;
                let hourly_rate = daily_rate as f64 / 24.0;
                // 연체료 계산 로직 검토 (시간당 요금 * 1.5 * 연체 시간)
                overdue_fee = (hourly_rate * 1.5 * overdue_hours as f64).round() as i32;
            }
        }
    }

    let update_reservation_result = sqlx::query!(
         "UPDATE reservation SET reservation_status = ?, return_date_actual = ?, overdue_fee = ? WHERE reservation_id = ?",
         if overdue_fee > 0 { "overdue" } else { "completed" },
         now, // 실제 반납 시간
         overdue_fee,
         reservation_id_str
     )
     .execute(&mut *tx) // 트랜잭션 사용
     .await;

    if let Err(e) = update_reservation_result {
        eprintln!("Failed to update reservation status and return time: {}", e);
        tx.rollback().await.ok();
        return Err(Status::InternalServerError);
    }

    if update_reservation_result.unwrap().rows_affected() > 0 {
        let update_car_result =
            sqlx::query!("UPDATE cars SET status = 'Available' WHERE id = ?", car_id)
                .execute(&mut *tx) // 트랜잭션 사용
                .await;

        if let Err(e) = update_car_result {
            // 차량 상태 업데이트 실패 시 롤백할지, 경고만 남길지 결정 필요
            eprintln!(
                "Warning: Failed to update car status for car ID {}: {}",
                car_id, e
            );
            // tx.rollback().await.ok();
            // return Err(Status::InternalServerError);
        }

        // 트랜잭션 커밋
        let commit_result = tx.commit().await;
        if let Err(e) = commit_result {
            eprintln!("트랜잭션 커밋 실패: {}", e);
            return Err(Status::InternalServerError);
        }

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
            }, // 0일때는 None
        }))
    } else {
        tx.rollback().await.ok();
        Err(Status::NotFound) // 업데이트할 예약 없음
    }
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
    let reservation_id_str = cancel_request.into_inner().reservation_id;

    let conn_result = pool.acquire().await;
    let mut conn = match conn_result {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to acquire connection: {}", e);
            return Err(Status::InternalServerError);
        }
    };

    let tx_result = conn.begin().await;
    let mut tx = match tx_result {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to begin transaction: {}", e);
            return Err(Status::InternalServerError);
        }
    };

    let reservation_info_result = sqlx::query!(
        "SELECT user_id, car_id FROM reservation WHERE reservation_id = ?",
        reservation_id_str
    )
    .fetch_optional(&mut *tx)
    .await;

    let reservation_info = match reservation_info_result {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Failed to fetch reservation: {}", e);
            tx.rollback().await.ok();
            return Err(Status::InternalServerError);
        }
    };

    let info = match reservation_info {
        Some(i) => i,
        None => {
            tx.rollback().await.ok();
            return Err(Status::NotFound);
        }
    };

    if user_id != info.user_id {
        tx.rollback().await.ok();
        return Err(Status::Forbidden);
    }

    let car_id = info.car_id;

    let update_reservation_result = sqlx::query!(
        "UPDATE reservation SET reservation_status = 'canceled' WHERE reservation_id = ?",
        reservation_id_str
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = update_reservation_result {
        eprintln!("Failed to update reservation status: {}", e);
        tx.rollback().await.ok();
        return Err(Status::InternalServerError);
    }

    if update_reservation_result.unwrap().rows_affected() > 0 {
        let update_car_result =
            sqlx::query!("UPDATE cars SET status = 'Available' WHERE id = ?", car_id)
                .execute(&mut *tx)
                .await;

        if let Err(e) = update_car_result {
            eprintln!(
                "Warning: Failed to update car status for car ID {}: {}",
                car_id, e
            );
        }

        let commit_result = tx.commit().await;
        if let Err(e) = commit_result {
            eprintln!("트랜잭션 커밋 실패: {}", e);
            return Err(Status::InternalServerError);
        }

        Ok(Json(CancelApiResponse {
            message: "예약이 성공적으로 취소되었습니다.".to_string(),
        }))
    } else {
        tx.rollback().await.ok();
        Err(Status::NotFound)
    }
}

#[get("/api/overdue_fee_info/<reservation_id>")]
pub async fn api_overdue_fee_info(
    pool: &State<MySqlPool>,
    auth_token: AuthToken,
    reservation_id: String,
) -> Result<Json<OverdueFeeInfo>, Status> {
    // 기존 main.rs의 api_overdue_fee_info 로직 전체 이동
    let user_id = auth_token
        .0
        .sub
        .parse::<i32>()
        .map_err(|_| Status::Unauthorized)?;

    let mut conn = pool.acquire().await.map_err(|e| {
        eprintln!("Failed to acquire connection: {}", e);
        Status::InternalServerError
    })?;

    // SQL 쿼리에서 reservation_id가 NULL일 수 있는지 확인 (unwrap_or_default() 제거 고려)
    let reservation = sqlx::query!(
        r#"
        SELECT
            r.reservation_id, -- 외부 ID (NOT NULL 가정)
            c.manufacturer AS car_manufacturer,
            c.name AS car_model,
            r.rental_date,
            r.return_date,
            r.return_date_actual,
            r.overdue_fee,
            c.daily_rate
        FROM reservation r
        JOIN cars c ON r.car_id = c.id
        WHERE r.reservation_id = ? AND r.user_id = ? AND r.reservation_status = 'overdue'
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
            // NaiveDateTime을 UTC로 변환하지 않고 그대로 사용하거나, DB 타임존 설정 확인
            let rental_date_naive = res.rental_date; // NaiveDateTime
            let return_date_naive = res.return_date; // NaiveDateTime
            let return_date_actual_naive = res.return_date_actual; // Option<NaiveDateTime>

            let overdue_hours = return_date_actual_naive
                .map(|actual| (actual - return_date_naive).num_hours()) // Option<i64>
                .unwrap_or(0); // 실제 반납 시간이 없으면 0시간

            let daily_rate = res.daily_rate;
            let hourly_rate = (daily_rate as f64 / 24.0).round() as i32;
            let base_fee = (hourly_rate as f64 * 1.5).round() as i32; // 시간당 연체 기본료

            Ok(Json(OverdueFeeInfo {
                reservation_id: res.reservation_id.unwrap(),
                car_info: format!("{} {}", res.car_manufacturer, res.car_model), // NULL 처리
                rental_date: rental_date_naive.date(),                           // NaiveDate
                expected_return_date: return_date_naive.date(),                  // NaiveDate
                actual_return_date: return_date_actual_naive.map(|dt| dt.date()), // Option<NaiveDate>
                base_fee,
                overdue_hours,
                total_overdue_fee: res.overdue_fee.unwrap_or(0) as i32, // NULL 처리
            }))
        }
        None => Err(Status::NotFound),
    }
}
