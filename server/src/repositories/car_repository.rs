use crate::models::car::{CarInfo, CarListResponse, CarQuery};
use async_trait::async_trait;
use rocket::fs::TempFile;
use sqlx::{Error, MySqlPool};

#[async_trait]
pub trait CarRepository: Sync + Send {
    async fn get_car_by_id(&self, id: i32) -> Result<Option<CarInfo>, Error>;
    async fn get_cars(&self, query: CarQuery) -> Result<CarListResponse, Error>;
    async fn add_car(&self, car_info: CarInfo, images: Vec<TempFile<'_>>) -> Result<String, Error>;
    async fn update_car(
        &self,
        car_info: CarInfo,
        images: Vec<TempFile<'_>>,
    ) -> Result<String, Error>;
    async fn delete_car(&self, car_info: CarInfo) -> Result<String, Error>;
}

pub struct MySqlCarRepository {
    pool: MySqlPool,
}

impl MySqlCarRepository {
    pub fn new(pool: MySqlPool) -> Self {
        MySqlCarRepository { pool }
    }
}

#[async_trait]
impl CarRepository for MySqlCarRepository {
    async fn get_car_by_id(&self, id: i32) -> Result<Option<CarInfo>, Error> {
        let sql = "SELECT id, plate_number, manufacturer, name, year, car_type, fuel_type, transmission, seat_num, color, image_url, car_trim, daily_rate, location, rating, description, status, owner FROM cars WHERE id = ?";
        sqlx::query_as::<_, CarInfo>(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn get_cars(&self, query: CarQuery) -> Result<CarListResponse, Error> {
        let start_index = query.start.unwrap_or(0);
        let requested_limit = query.limit.unwrap_or(6);
        let limit = if requested_limit > 0 {
            requested_limit
        } else {
            6
        };
        let sort_by = query.sort.unwrap_or_else(|| "plate_number ASC".to_string());
        let min_price = query.min_daily_rate;
        let max_price = query.max_daily_rate;
        let car_types: Option<Vec<String>> = query
            .car_type
            .map(|s| s.split(',').map(String::from).collect());
        let fuel_types: Option<Vec<String>> = query
            .fuel_type
            .map(|s| s.split(',').map(String::from).collect());
        let transmissions: Option<Vec<String>> = query
            .transmission
            .map(|s| s.split(',').map(String::from).collect());
        let status: Option<String> = query.status;

        let mut where_clauses = Vec::new();
        let mut query_params: Vec<String> = Vec::new();

        if let (Some(rental), Some(return_)) = (&query.rental_date, &query.return_date) {
            where_clauses.push(
                "id NOT IN (
                    SELECT car_id FROM reservation
                    WHERE NOT (
                        return_date < ? OR rental_date > ?
                    )
                ) AND status != 'Unavailable'"
                    .to_string(),
            );
            query_params.push(rental.clone());
            query_params.push(return_.clone());
        }

        if let Some(min) = min_price {
            where_clauses.push("daily_rate >= ?".to_string());
            query_params.push(min.to_string());
        }

        if let Some(max) = max_price {
            where_clauses.push("daily_rate <= ?".to_string());
            query_params.push(max.to_string());
        }

        if let Some(types) = &car_types {
            if !types.is_empty() {
                let placeholders = vec!["?"; types.len()].join(",");
                where_clauses.push(format!("car_type IN ({})", placeholders));
                query_params.extend(types.iter().cloned());
            }
        }
        if let Some(fuels) = &fuel_types {
            if !fuels.is_empty() {
                let placeholders = vec!["?"; fuels.len()].join(",");
                where_clauses.push(format!("fuel_type IN ({})", placeholders));
                query_params.extend(fuels.iter().cloned());
            }
        }
        if let Some(trans) = &transmissions {
            if !trans.is_empty() {
                let placeholders = vec!["?"; trans.len()].join(",");
                where_clauses.push(format!("transmission IN ({})", placeholders));
                query_params.extend(trans.iter().cloned());
            }
        }

        if let Some(status) = &status {
            where_clauses.push(format!("status IN (?)",));
            query_params.push(status.clone());
        }

        if let Some(owner_id_val) = query.owner_id {
            where_clauses.push("owner = ?".to_string());
            query_params.push(owner_id_val.to_string());
        }

        let where_clause = if !where_clauses.is_empty() {
            format!("WHERE {}", where_clauses.join(" AND "))
        } else {
            String::new()
        };

        let count_sql = format!("SELECT COUNT(*) FROM cars {}", where_clause);
        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        for param in &query_params {
            count_query = count_query.bind(param);
        }
        let count_result = count_query.fetch_one(&self.pool).await;

        let order_by_clause = match sort_by.as_str() {
            "daily_rate_asc" => "ORDER BY daily_rate ASC",
            "daily_rate_desc" => "ORDER BY daily_rate DESC",
            "rating_desc" => "ORDER BY rating DESC",
            "rating_asc" => "ORDER BY rating ASC",
            _ => "ORDER BY name ASC",
        };

        let sql = format!(
            "SELECT id, plate_number, manufacturer, name, year, car_type, fuel_type, transmission, seat_num, color, image_url, daily_rate, location, rating, description, status FROM cars {} {} LIMIT ? OFFSET ?",
            where_clause, order_by_clause
        );

        let mut cars_query = sqlx::query_as::<_, CarInfo>(&sql);
        for param in &query_params {
            cars_query = cars_query.bind(param);
        }
        cars_query = cars_query.bind(limit as u64).bind(start_index as u64);
        let cars_result = cars_query.fetch_all(&self.pool).await;

        match (count_result, cars_result) {
            (Ok(total), Ok(cars)) => Ok(CarListResponse {
                total: total as usize,
                cars,
            }),
            (Err(e), _) => {
                eprintln!("Error fetching car count: {}", e);
                Err(e)
            }
            (_, Err(e)) => {
                eprintln!("Error fetching cars: {}", e);
                Err(e)
            }
        }
    }

    async fn add_car(&self, car_info: CarInfo, images: Vec<TempFile<'_>>) -> Result<String, Error> {
        let result = sqlx::query("INSERT INTO cars (plate_number, manufacturer, name, year, car_type, fuel_type, transmission, seat_num, color, car_trim, daily_rate, location, rating, description, status, image_url, owner) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&car_info.plate_number())
            .bind(&car_info.manufacturer())
            .bind(&car_info.name())
            .bind(car_info.year())
            .bind(&car_info.car_type())
            .bind(&car_info.fuel_type())
            .bind(&car_info.transmission())
            .bind(car_info.seat_num())
            .bind(&car_info.color())
            .bind(&car_info.car_trim())
            .bind(car_info.daily_rate())
            .bind(&car_info.location())
            .bind(car_info.rating())
            .bind(&car_info.description())
            .bind(&car_info.status())
            .bind("[]")
            .bind(car_info.owner())
            .execute(&self.pool)
            .await?;

        let car_id = result.last_insert_id();

        let mut image_urls = Vec::new();

        if !images.is_empty() {
            use chrono::Utc;
            use std::fs;
            let image_dir = "static/car_images";
            fs::create_dir_all(image_dir).ok();

            for (idx, mut image) in images.into_iter().enumerate() {
                let filename = format!("car_{}_{}_{}.jpg", car_id, Utc::now().timestamp(), idx);
                let save_path = format!("{}/{}", image_dir, filename);
                image.copy_to(&save_path).await.map_err(|e| Error::Io(e))?;

                let is_main = idx == 0;
                sqlx::query(
                    "INSERT INTO car_images (car_id, image_path, is_main) VALUES (?, ?, ?)",
                )
                .bind(car_id as i64)
                .bind(&save_path)
                .bind(is_main)
                .execute(&self.pool)
                .await?;

                image_urls.push(save_path);
            }
        }

        println!("{:?}", image_urls);

        let image_urls_json = serde_json::to_string(&image_urls)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        sqlx::query("UPDATE cars SET image_url = ? WHERE id = ?")
            .bind(image_urls_json)
            .bind(car_id as i64)
            .execute(&self.pool)
            .await?;

        Ok("차량이 성공적으로 추가되었습니다.".to_string())
    }

    async fn update_car(
        &self,
        car_info: CarInfo,
        images: Vec<TempFile<'_>>,
    ) -> Result<String, Error> {
        let car_id = car_info.id().ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "차량 ID가 없습니다",
            ))
        })?;

        sqlx::query("UPDATE cars SET plate_number = ?, manufacturer = ?, name = ?, year = ?, car_type = ?, fuel_type = ?, transmission = ?, seat_num = ?, color = ?, car_trim = ?, daily_rate = ?, location = ?, rating = ?, description = ?, status = ? WHERE id = ?")
            .bind(&car_info.plate_number())
            .bind(&car_info.manufacturer())
            .bind(&car_info.name())
            .bind(car_info.year())
            .bind(&car_info.car_type())
            .bind(&car_info.fuel_type())
            .bind(&car_info.transmission())
            .bind(car_info.seat_num())
            .bind(&car_info.color())
            .bind(&car_info.car_trim())
            .bind(car_info.daily_rate())
            .bind(&car_info.location())
            .bind(car_info.rating())
            .bind(&car_info.description())
            .bind(&car_info.status())
            .bind(car_id)
            .execute(&self.pool)
            .await?;

        // 기존 이미지 URL 가져오기
        let existing_images: Vec<String> = sqlx::query_scalar::<_, String>(
            "SELECT CAST(image_url AS CHAR) FROM cars WHERE id = ?",
        )
        .bind(car_id)
        .fetch_one(&self.pool)
        .await
        .map(|json_str| serde_json::from_str(&json_str).unwrap_or_else(|_| Vec::new()))?;

        // 삭제된 이미지 처리
        let mut image_urls = existing_images;
        if let Some(deleted_images_str) = car_info.deleted_images() {
            if let Ok(deleted_images) = serde_json::from_str::<Vec<String>>(deleted_images_str) {
                // car_images 테이블에서 삭제
                for image_path in &deleted_images {
                    // 파일 시스템에서 이미지 삭제
                    if let Err(e) = std::fs::remove_file(image_path) {
                        eprintln!("Error deleting file {}: {}", image_path, e);
                    }

                    // 데이터베이스에서 이미지 정보 삭제
                    sqlx::query("DELETE FROM car_images WHERE car_id = ? AND image_path = ?")
                        .bind(car_id)
                        .bind(image_path)
                        .execute(&self.pool)
                        .await?;
                }
                // image_urls에서 제거
                image_urls.retain(|url| !deleted_images.contains(url));
            }
        }

        if !images.is_empty() {
            use chrono::Utc;
            use std::fs;
            let image_dir = "static/car_images";
            fs::create_dir_all(image_dir).ok();

            for (idx, mut image) in images.into_iter().enumerate() {
                let filename = format!("car_{}_{}_{}.jpg", car_id, Utc::now().timestamp(), idx);
                let save_path = format!("{}/{}", image_dir, filename);
                image.copy_to(&save_path).await.map_err(|e| Error::Io(e))?;

                let is_main = image_urls.is_empty() && idx == 0;
                sqlx::query(
                    "INSERT INTO car_images (car_id, image_path, is_main) VALUES (?, ?, ?)",
                )
                .bind(car_id as i64)
                .bind(&save_path)
                .bind(is_main)
                .execute(&self.pool)
                .await?;

                image_urls.push(save_path);
            }
        }

        // image_urls가 비어있지 않은 경우에만 업데이트
        let image_urls_json = serde_json::to_string(&image_urls)
            .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        sqlx::query("UPDATE cars SET image_url = ? WHERE id = ?")
            .bind(image_urls_json)
            .bind(car_id as i64)
            .execute(&self.pool)
            .await?;

        Ok("차량이 성공적으로 업데이트되었습니다.".to_string())
    }

    async fn delete_car(&self, car_info: CarInfo) -> Result<String, Error> {
        let reservation_exists = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT 1 FROM reservation WHERE car_id = ? LIMIT 1",
        )
        .bind(car_info.id())
        .fetch_optional(&self.pool)
        .await?;

        if reservation_exists.is_some() {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "차량이 예약중입니다",
            )));
        }

        // 먼저 car_images 테이블에서 이미지 삭제
        sqlx::query("DELETE FROM car_images WHERE car_id = ?")
            .bind(car_info.id())
            .execute(&self.pool)
            .await?;

        // 그 다음 cars 테이블에서 차량 삭제
        let result = sqlx::query("DELETE FROM cars WHERE id = ?")
            .bind(car_info.id())
            .execute(&self.pool)
            .await;

        match result {
            Ok(_) => Ok("차량이 성공적으로 삭제되었습니다.".to_string()),
            Err(e) => Err(Error::from(e)),
        }
    }
}
