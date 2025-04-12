use lazy_static::lazy_static;
use rand::Rng as _;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rocket::State;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref RNG: Mutex<ChaCha8Rng> = {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Mutex::new(ChaCha8Rng::seed_from_u64(seed))
    };
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PaymentInfo {
    card_number: String,
    expiry_date: String, // MM/YY 형식
    cvc: String,
    cardholder_name: String,
    reservation_id: i32,    // 어떤 예약에 대한 결제인지 식별
    total_amount: i32,      // 결제할 총 금액
    payment_method: String, // 결제 방법 추가
    payment_type: String,   // 결제 유형 추가 ('reservation', 'overdue' 등)
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct PaymentResult {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    transaction_id: Option<String>, // 결제 성공 시 거래 ID
}

#[post("/api/pay", data = "<payment_info>")]
pub async fn process_payment(
    pool: &State<MySqlPool>,
    payment_info: Json<PaymentInfo>,
) -> Result<Json<PaymentResult>, Status> {
    let payment = payment_info.into_inner();

    // 간단한 유효성 검사
    if payment.card_number.len() < 16
        || payment.expiry_date.len() != 5
        || payment.cvc.len() < 3
        || payment.cardholder_name.is_empty()
        || payment.total_amount <= 0
        || payment.payment_method.is_empty()
        || payment.payment_type.is_empty()
    {
        return Ok(Json(PaymentResult {
            success: false,
            message: "결제 정보가 올바르지 않습니다.".to_string(),
            transaction_id: None,
        }));
    }

    let payment_successful = RNG.lock().unwrap().random_bool(0.8);
    let transaction_id = format!("TXN-{}", RNG.lock().unwrap().random::<u32>());

    println!(
        "가상 결제 처리: 카드 끝 4자리 ****-****-****-{}, 예약 ID {}, 금액 {}원, 방법 {}, 유형 {}, 성공: {}",
        payment.card_number.chars().skip(12).collect::<String>(),
        payment.reservation_id,
        payment.total_amount,
        payment.payment_method,
        payment.payment_type,
        payment_successful
    );

    let mut conn = pool.acquire().await.map_err(|e| {
        eprintln!("Failed to acquire connection: {}", e);
        Status::InternalServerError
    })?;

    if payment_successful {
        // payment 테이블에 결제 정보 저장
        let result_payment = sqlx::query!(
            r#"
            INSERT INTO payment (reservation_id, payment_method, amount, transaction_id, payment_status, payment_type)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            payment.reservation_id,
            payment.payment_method,
            payment.total_amount,
            transaction_id,
            "completed",
            payment.payment_type
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| {
            eprintln!("payment 테이블에 결제 정보 저장 실패: {}", e);
            Status::InternalServerError
        })?;

        if result_payment.rows_affected() > 0 {
            if payment.payment_type == "overdue" {
                sqlx::query!(
                    "UPDATE reservation SET reservation_status = 'completed' WHERE id = ?",
                    payment.reservation_id
                )
                .execute(&mut *conn)
                .await
                .map_err(|e| {
                    eprintln!("reservation 테이블 업데이트 실패: {}", e);
                    Status::InternalServerError
                })?;
            }

            Ok(Json(PaymentResult {
                success: true,
                message: "결제가 완료되었습니다.".to_string(),
                transaction_id: Some(transaction_id),
            }))
        } else {
            Ok(Json(PaymentResult {
                success: false,
                message: "결제는 성공했지만, 결제 정보를 저장하는 데 실패했습니다.".to_string(),
                transaction_id: Some(transaction_id),
            }))
        }
    } else {
        // payment 테이블에 실패 정보 저장
        let _ = sqlx::query!(
            r#"
            INSERT INTO payment (reservation_id, payment_method, amount, payment_status, payment_type)
            VALUES (?, ?, ?, ?, ?)
            "#,
            payment.reservation_id,
            payment.payment_method,
            payment.total_amount,
            "failed",
            payment.payment_type
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| eprintln!("payment 테이블에 결제 실패 정보 저장 실패: {}", e));

        Ok(Json(PaymentResult {
            success: false,
            message: "결제에 실패했습니다. 카드 정보를 확인하거나 다른 결제 수단을 이용해주세요."
                .to_string(),
            transaction_id: None,
        }))
    }
}
