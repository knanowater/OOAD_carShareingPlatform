use crate::utils::serve_html;
use rocket::fs::NamedFile;
use rocket::get; // serve_html 함수 임포트

// 각 페이지 라우트 정의
#[get("/")]
pub async fn index_page() -> Option<NamedFile> {
    serve_html("../client/index.html").await
}

#[get("/login")]
pub async fn login_page() -> Option<NamedFile> {
    serve_html("../client/login.html").await
}

#[get("/signup")]
pub async fn signup_page() -> Option<NamedFile> {
    serve_html("../client/signup.html").await
}

#[get("/list")]
pub async fn list_page() -> Option<NamedFile> {
    serve_html("../client/list.html").await
}

#[get("/reservation")]
pub async fn reservation_page() -> Option<NamedFile> {
    serve_html("../client/reservation.html").await
}

#[allow(unused_variables)]
#[get("/reservation/success?<id>")]
pub async fn reservation_success_page(id: Option<String>) -> Option<NamedFile> {
    serve_html("../client/reservation-success.html").await
}

#[get("/mypage")]
pub async fn mypage_page() -> Option<NamedFile> {
    serve_html("../client/mypage/mypage.html").await
}

#[get("/mypage/reservations")]
pub async fn mypage_reservations_page() -> Option<NamedFile> {
    serve_html("../client/mypage/reservations.html").await
}

#[get("/overdue_fee")]
pub async fn overdue_fee_page() -> Option<NamedFile> {
    serve_html("../client/mypage/overdue_fee.html").await
}

#[get("/admin/dashboard")]
pub async fn admin_dashboard_page() -> Option<NamedFile> {
    serve_html("../client/admin/dashboard.html").await
}

#[get("/admin/vehicles")]
pub async fn car_management_page() -> Option<NamedFile> {
    serve_html("../client/admin/vehicles.html").await
}

#[allow(unused_variables)]
#[get("/detail?<id>")]
pub async fn car_detail_page(id: Option<String>) -> Option<NamedFile> {
    serve_html("../client/detail.html").await
}

#[get("/mypage/host/add_car")]
pub async fn host_add_car_page() -> Option<NamedFile> {
    serve_html("../client/mypage/host/add_car.html").await
}

#[get("/mypage/host/management")]
pub async fn host_management_page() -> Option<NamedFile> {
    serve_html("../client/mypage/host/management.html").await
}

#[get("/mypage/host/reservations")]
pub async fn host_reservations_page() -> Option<NamedFile> {
    serve_html("../client/mypage/host/reservations.html").await
}
