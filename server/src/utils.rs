use chrono::Utc;
use lazy_static::lazy_static;
use rand::{RngCore, SeedableRng, rngs::StdRng};
use rocket::fs::NamedFile;
use std::env;
use std::path::Path;
use std::sync::Mutex;

lazy_static! {
    static ref RNG: Mutex<StdRng> = {
        let seed = env::var("RNG_SEED")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or_else(|| {
                let fallback = Utc::now().timestamp() as u64;
                eprintln!("[RNG] RNG_SEED not set, using fallback seed: {}", fallback);
                fallback
            });

        Mutex::new(StdRng::seed_from_u64(seed))
    };
}

// 예약 ID 생성 함수
pub fn generate_reservation_id() -> String {
    let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let mut rng = RNG.lock().expect("Failed to lock RNG");
    let random_part = rng.next_u32();
    format!("RSV-{}-{}", timestamp, random_part)
}

// HTML 파일 서빙 함수
pub async fn serve_html(path: &str) -> Option<NamedFile> {
    NamedFile::open(Path::new(path)).await.ok()
}
