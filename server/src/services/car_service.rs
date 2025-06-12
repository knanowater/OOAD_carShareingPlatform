use crate::models::car::{CarForm, CarInfo, CarListResponse, CarQuery};
use crate::repositories::car_repository::{CarRepository, MySqlCarRepository};
use rocket::form::Form;
use rocket::http::Status;
use sqlx::MySqlPool;

pub struct CarService {
    pool: MySqlPool,
}

impl CarService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    // GRASP - Information Expert 패턴: 차량 생성 비즈니스 로직을 서비스에서 처리
    pub async fn create_car(
        &self,
        mut form: Form<CarForm<'_>>,
        user_id: i32,
    ) -> Result<String, String> {
        let car_repository = MySqlCarRepository::new(self.pool.clone());

        // GRASP - Creator 패턴: CarInfo 객체 생성 및 설정
        let mut car_info = CarInfo::new();
        car_info.set_plate_number(form.plate_number.clone());
        car_info.set_manufacturer(form.manufacturer.clone());
        car_info.set_name(form.name.clone());
        car_info.set_year(form.year);
        car_info.set_car_type(form.car_type.clone());
        car_info.set_fuel_type(form.fuel_type.clone());
        car_info.set_transmission(form.transmission.clone());
        car_info.set_seat_num(form.seat_num);
        car_info.set_car_trim(Some(form.car_trim.clone()));
        car_info.set_daily_rate(form.daily_rate);
        car_info.set_location(form.location.clone());
        car_info.set_rating(0.0);
        car_info.set_description(Some(form.description.clone()));
        car_info.set_status("Available".to_string());
        car_info.set_owner(Some(user_id));
        car_info.set_color(Some(form.color.clone()));

        let images = std::mem::take(&mut form.images);

        // GRASP - Low Coupling: Repository를 통한 데이터 접근
        car_repository
            .add_car(car_info, images)
            .await
            .map_err(|e| e.to_string())
    }

    // GRASP - Information Expert 패턴: 차량 업데이트 비즈니스 로직
    pub async fn update_car(&self, mut form: Form<CarForm<'_>>) -> Result<String, String> {
        let car_repository = MySqlCarRepository::new(self.pool.clone());

        let mut car_info = CarInfo::new();
        car_info.set_id(form.id.ok_or_else(|| "차량 ID가 필요합니다".to_string())?);
        car_info.set_plate_number(form.plate_number.clone());
        car_info.set_manufacturer(form.manufacturer.clone());
        car_info.set_name(form.name.clone());
        car_info.set_year(form.year);
        car_info.set_car_type(form.car_type.clone());
        car_info.set_fuel_type(form.fuel_type.clone());
        car_info.set_transmission(form.transmission.clone());
        car_info.set_seat_num(form.seat_num);
        car_info.set_car_trim(Some(form.car_trim.clone()));
        car_info.set_daily_rate(form.daily_rate);
        car_info.set_location(form.location.clone());
        car_info.set_rating(0.0);
        car_info.set_description(Some(form.description.clone()));
        car_info.set_status("Available".to_string());
        car_info.set_deleted_images(form.deleted_images.clone());
        car_info.set_color(Some(form.color.clone()));

        let images = std::mem::take(&mut form.images);

        car_repository
            .update_car(car_info, images)
            .await
            .map_err(|e| e.to_string())
    }

    // GRASP - Information Expert 패턴: 차량 삭제 비즈니스 로직
    pub async fn delete_car(&self, car_id: i32, user_id: i32) -> Result<String, (Status, String)> {
        let car_repository = MySqlCarRepository::new(self.pool.clone());

        // 차량 조회
        let car = car_repository
            .get_car_by_id(car_id)
            .await
            .map_err(|_| (Status::InternalServerError, "Error retrieving car".into()))?
            .ok_or((
                Status::NotFound,
                format!("Car with ID {} not found", car_id),
            ))?;

        // 소유권 검증
        if car.owner().unwrap() != user_id {
            return Err((
                Status::Forbidden,
                "You do not have permission to delete this car.".into(),
            ));
        }

        // 차량 삭제
        car_repository
            .delete_car(car)
            .await
            .map(|msg| format!("{{\"status\": \"success\", \"message\": \"{}\"}}", msg))
            .map_err(|e| {
                let err_msg = e.to_string();
                let user_msg = if err_msg.contains("차량이 예약중입니다") {
                    "차량이 예약중입니다".to_string()
                } else {
                    err_msg
                };
                (Status::InternalServerError, user_msg)
            })
    }

    // GRASP - Information Expert 패턴: 차량 목록 조회
    pub async fn get_cars(&self, query: CarQuery) -> Result<CarListResponse, String> {
        let car_repository = MySqlCarRepository::new(self.pool.clone());
        car_repository
            .get_cars(query)
            .await
            .map_err(|e| e.to_string())
    }

    // GRASP - Information Expert 패턴: 차량 상세 조회
    pub async fn get_car_by_id(&self, id: i32) -> Result<Option<CarInfo>, String> {
        let car_repository = MySqlCarRepository::new(self.pool.clone());
        car_repository
            .get_car_by_id(id)
            .await
            .map_err(|e| e.to_string())
    }
}
