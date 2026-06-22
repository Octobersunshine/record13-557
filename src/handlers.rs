use crate::models::*;
use crate::services::ExamService;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppState {
    pub exam_service: ExamService,
}

#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    pub user_id: Option<String>,
}

pub async fn health_check() -> impl IntoResponse {
    let mut response = HashMap::new();
    response.insert("status", "ok");
    Json(response)
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    match state.exam_service.create_user(payload.username).await {
        Ok(user) => (
            StatusCode::CREATED,
            Json(UserResponse {
                id: user.id,
                username: user.username,
                created_at: user.created_at,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match state.exam_service.get_user(&user_id).await {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(UserResponse {
                id: user.id,
                username: user.username,
                created_at: user.created_at,
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(error_response("User not found")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    match state.exam_service.list_users().await {
        Ok(users) => {
            let response: Vec<UserResponse> = users
                .into_iter()
                .map(|u| UserResponse {
                    id: u.id,
                    username: u.username,
                    created_at: u.created_at,
                })
                .collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn create_session(
    State(state): State<AppState>,
    Json(payload): Json<CreateSessionRequest>,
) -> impl IntoResponse {
    match state
        .exam_service
        .create_session(payload.user_id, payload.exam_title, payload.total_questions)
        .await
    {
        Ok(session) => (
            StatusCode::CREATED,
            Json(CreateSessionResponse {
                session_id: session.id,
                user_id: session.user_id,
                exam_title: session.exam_title,
                start_time: session.start_time,
                total_questions: session.total_questions,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.exam_service.get_session_detail(&session_id).await {
        Ok(Some(detail)) => (StatusCode::OK, Json(detail)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(error_response("Session not found")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn list_sessions(
    State(state): State<AppState>,
    Query(query): Query<ListSessionsQuery>,
) -> impl IntoResponse {
    match state
        .exam_service
        .list_sessions(query.user_id.as_deref())
        .await
    {
        Ok(sessions) => (StatusCode::OK, Json(sessions)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn list_suspicious_sessions(State(state): State<AppState>) -> impl IntoResponse {
    match state.exam_service.list_suspicious_sessions().await {
        Ok(sessions) => (StatusCode::OK, Json(sessions)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn report_event(
    State(state): State<AppState>,
    Json(payload): Json<ReportEventRequest>,
) -> impl IntoResponse {
    if payload.session_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(error_response("session_id is required")),
        )
            .into_response();
    }
    if payload.event_type.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(error_response("event_type is required")),
        )
            .into_response();
    }

    match state.exam_service.report_event(payload).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn submit_answer(
    State(state): State<AppState>,
    Json(payload): Json<SubmitAnswerRequest>,
) -> impl IntoResponse {
    if payload.session_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(error_response("session_id is required")),
        )
            .into_response();
    }

    match state.exam_service.submit_answer(payload).await {
        Ok(_) => {
            let mut response = HashMap::new();
            response.insert("status", "success");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn end_session(
    State(state): State<AppState>,
    Json(payload): Json<EndSessionRequest>,
) -> impl IntoResponse {
    match state.exam_service.end_session(&payload.session_id).await {
        Ok(Some(analysis)) => (StatusCode::OK, Json(analysis)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(error_response("Session not found")),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn get_session_analysis(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.exam_service.get_session_analysis(&session_id).await {
        Ok(analysis) => (StatusCode::OK, Json(analysis)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn mark_suspicious(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(payload): Json<MarkSuspiciousRequest>,
) -> impl IntoResponse {
    match state
        .exam_service
        .mark_suspicious(&session_id, &payload.reason)
        .await
    {
        Ok(_) => {
            let mut response = HashMap::new();
            response.insert("status", "success");
            response.insert("session_id", session_id);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct MarkSuspiciousRequest {
    pub reason: String,
}

pub async fn get_suspicious_leaderboard(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.exam_service.get_suspicious_leaderboard().await {
        Ok(leaderboard) => (StatusCode::OK, Json(leaderboard)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn export_anomalous_json(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.exam_service.export_anomalous_records().await {
        Ok(export) => (StatusCode::OK, Json(export)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

pub async fn export_anomalous_csv(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.exam_service.export_anomalous_csv().await {
        Ok(csv) => {
            let bom = "\u{FEFF}";
            let body = format!("{}{}", bom, csv);
            (
                StatusCode::OK,
                [
                    ("content-type", "text/csv; charset=utf-8"),
                    ("content-disposition", "attachment; filename=\"anomalous_records.csv\""),
                ],
                body,
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(&e.to_string())),
        )
            .into_response(),
    }
}

fn error_response(message: &str) -> HashMap<String, String> {
    let mut response = HashMap::new();
    response.insert("error".to_string(), message.to_string());
    response
}
