use chrono::Utc;
use rocket::Rocket;
use rocket::tokio;
use rocket::tokio::time::{Duration, sleep};
use sqlx::MySqlPool;

async fn start_rental_if_due(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let now = Utc::now().naive_utc();
    let mut conn = pool.acquire().await?;

    let overdue_reservations = sqlx::query!(
        "SELECT reservation_id FROM reservation
        WHERE rental_date < ?
        AND reservation_status = 'scheduled'",
        now
    )
    .fetch_all(&mut *conn)
    .await?;

    for reservation in &overdue_reservations {
        sqlx::query!(
            "UPDATE reservation
            SET reservation_status = 'in_use'
            WHERE reservation_id = ?",
            reservation.reservation_id
        )
        .execute(&mut *conn)
        .await?;
    }

    for reservation in &overdue_reservations {
        sqlx::query!(
            "UPDATE reservation_log
            SET reservation_status = 'in_use'
            WHERE reservation_id = ?",
            reservation.reservation_id
        )
        .execute(&mut *conn)
        .await?;
    }

    println!("Started {} rentals at {}", overdue_reservations.len(), now);
    Ok(())
}

async fn cancel_rental_if_failed_to_host_confirm(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let now = Utc::now().naive_utc();
    let mut conn = pool.acquire().await?;

    let overdue_reservations = sqlx::query!(
        "SELECT reservation_id FROM reservation
        WHERE rental_date < ?
        AND reservation_status = 'pending'",
        now
    )
    .fetch_all(&mut *conn)
    .await?;

    for reservation in &overdue_reservations {
        sqlx::query!(
            "DELETE FROM reservation
            WHERE reservation_id = ?",
            reservation.reservation_id
        )
        .execute(&mut *conn)
        .await?;
    }

    for reservation in &overdue_reservations {
        sqlx::query!(
            "UPDATE reservation_log
            SET reservation_status = 'canceled'
            WHERE reservation_id = ?",
            reservation.reservation_id
        )
        .execute(&mut *conn)
        .await?;
    }

    println!("Canceled {} rentals at {}", overdue_reservations.len(), now);
    Ok(())
}

async fn update_overdue_reservations(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    let now = Utc::now().naive_utc();
    let mut conn = pool.acquire().await?;

    let overdue_reservations = sqlx::query!(
        "SELECT reservation_id FROM reservation 
        WHERE return_date < ? 
        AND reservation_status IN ('scheduled', 'in_use')",
        now
    )
    .fetch_all(&mut *conn)
    .await?;

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
    let update_overdue_reservations_pool = pool.clone();
    let start_rental_if_due_pool = pool.clone();
    let cancel_rental_if_failed_to_host_confirm_pool = pool.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await; // 시간 조절!!!
            if let Err(e) = update_overdue_reservations(&update_overdue_reservations_pool).await {
                eprintln!("Error updating overdue reservations: {}", e);
            }
        }
    });

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await; // 시간 조절!!!
            if let Err(e) = start_rental_if_due(&start_rental_if_due_pool).await {
                eprintln!("Error starting rentals: {}", e);
            }
        }
    });

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await; // 시간 조절!!!
            if let Err(e) = cancel_rental_if_failed_to_host_confirm(
                &cancel_rental_if_failed_to_host_confirm_pool,
            )
            .await
            {
                eprintln!("Error canceling rentals: {}", e);
            }
        }
    });

    rocket
}
