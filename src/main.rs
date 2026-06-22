mod db;
mod handlers;
mod models;
mod services;

use crate::db::*;
use crate::handlers::*;
use crate::services::*;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "exam_monitor=debug,tower_http=debug,axum=info".into()),
        )
        .init();

    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "exam_monitor.db".to_string());
    let db_url = format!("sqlite:{}", db_path);

    tracing::info!("Connecting to database: {}", db_url);
    let db = Database::new(&db_url).await?;
    db.run_migrations().await?;
    tracing::info!("Database migrations completed");

    let user_repo = UserRepository::new(db.clone());
    let session_repo = ExamSessionRepository::new(db.clone());
    let event_repo = BehaviorEventRepository::new(db.clone());
    let answer_repo = QuestionAnswerRepository::new(db.clone());
    let leaderboard_repo = LeaderboardRepository::new(db.clone());

    let detection_config = DetectionConfig::default();
    let detection_service = BehaviorDetectionService::new(
        event_repo.clone(),
        session_repo.clone(),
        answer_repo.clone(),
        detection_config,
    );

    let exam_service = ExamService::new(
        user_repo,
        session_repo,
        event_repo,
        answer_repo,
        detection_service,
        leaderboard_repo,
    );

    let app_state = AppState { exam_service };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let static_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static");
    tracing::info!("Serving static files from: {:?}", static_dir);

    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/users", post(create_user).get(list_users))
        .route("/users/:id", get(get_user))
        .route("/sessions", post(create_session).get(list_sessions))
        .route("/sessions/suspicious", get(list_suspicious_sessions))
        .route("/sessions/:id", get(get_session))
        .route("/sessions/:id/analysis", get(get_session_analysis))
        .route("/sessions/:id/suspicious", post(mark_suspicious))
        .route("/sessions/end", post(end_session))
        .route("/events", post(report_event))
        .route("/answers", post(submit_answer))
        .route("/leaderboard/suspicious", get(get_suspicious_leaderboard))
        .route("/export/anomalous", get(export_anomalous_json))
        .route("/export/anomalous/csv", get(export_anomalous_csv));

    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new(static_dir))
        .with_state(app_state)
        .layer(cors);

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()?;
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    tracing::info!("Server starting on http://{}", addr);
    println!("\n========================================");
    println!("  🚀 考试行为监控系统已启动");
    println!("  📡 服务器地址: http://{}", addr);
    println!("  🌐 前端页面: http://{}/", addr);
    println!("  📊 API 文档: http://{}/api/health", addr);
    println!("========================================\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
