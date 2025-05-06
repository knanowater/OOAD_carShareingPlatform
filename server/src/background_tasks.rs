use chrono::Utc;
use rocket::Rocket;
use rocket::tokio;
use rocket::tokio::time::{Duration, sleep};
use sqlx::MySqlPool;

async fn update_overdue_reservations(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let now = Utc::now().naive_utc();
    let mut conn = pool.acquire().await?;

    // 먼저 업데이트가 필요한 예약 ID들을 조회
    let overdue_reservations = sqlx::query!(
        "SELECT reservation_id FROM reservation 
        WHERE return_date < ? 
        AND reservation_status IN ('scheduled', 'in_use')",
        now
    )
    .fetch_all(&mut *conn)
    .await?;

    // reservation 테이블 업데이트
    for reservation in &overdue_reservations {
        sqlx::query!(
            "UPDATE reservation
            SET reservation_status = 'overdue'
            WHERE reservation_id = ?
            AND reservation_status IN ('scheduled', 'in_use')",
            reservation.reservation_id
        )
        .execute(&mut *conn)
        .await?;
    }

    // reservation_log 테이블 업데이트
    for reservation in &overdue_reservations {
        sqlx::query!(
            "UPDATE reservation_log
            SET reservation_status = 'overdue'
            WHERE reservation_id = ?
            AND reservation_status IN ('scheduled', 'in_use')",
            reservation.reservation_id
        )
        .execute(&mut *conn)
        .await?;
    }

    println!(
        "Updated {} overdue reservations at {}",
        overdue_reservations.len(),
        now
    );
    Ok(())
}

pub fn start_background_tasks(rocket: Rocket<rocket::Build>) -> Rocket<rocket::Build> {
    let pool = rocket
        .state::<MySqlPool>()
        .expect("Database pool not configured.");
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await; // 시간 조절!!!
            if let Err(e) = update_overdue_reservations(&pool_clone).await {
                eprintln!("Error updating overdue reservations: {}", e);
            }
        }
    });

    // 백그라운드 작업 시작 후 Rocket 인스턴스를 그대로 반환 (Build 상태)
    rocket
}
