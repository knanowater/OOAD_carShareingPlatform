# carShareingPlatform  

이 프로젝트는 웹 기반 차량 공유 서비스 플랫폼입니다.

## 프로젝트 구조

  
```

.

├── client/ # 프론트엔드 코드

│ ├── admin/ # 관리자 페이지

│ ├── mypage/ # 사용자 마이페이지

│ ├── scripts/ # 자바스크립트 파일

│ └── *.html # HTML 페이지들

├── server/ # 백엔드 코드 (Rust)

│ ├── src/ # 소스 코드

│ ├── static/ # 정적 파일

│ └── Cargo.toml # Rust 의존성 관리

├── mysql-data/ # MySQL 데이터 저장소

└── docker-compose.yml # Docker 설정 파일

```

  

## 주요 기능

  

- 사용자 회원가입 및 로그인

- 차량 목록 조회

- 차량 상세 정보 확인

- 차량 예약 시스템

- 마이페이지를 통한 예약 관리

- 관리자 페이지를 통한 시스템 관리

  

## 기술 스택


- Frontend: HTML, Tailwind CSS, JavaScript

- Backend: Rust Rocket

- Database: MySQL

- Container: Docker

