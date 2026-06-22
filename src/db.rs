use crate::models::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn from_file(path: &Path) -> Result<Self> {
        let database_url = format!("sqlite:{}", path.display());
        Self::new(&database_url).await
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn run_migrations(&self) -> Result<()> {
        let sql = include_str!("../migrations/001_initial.sql");
        sqlx::raw_sql(sql).execute(&self.pool).await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct UserRepository {
    db: Database,
}

impl UserRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create(&self, user: &User) -> Result<User> {
        sqlx::query(
            "INSERT INTO users (id, username, created_at) VALUES (?, ?, ?)"
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(user.created_at)
        .execute(self.db.pool())
        .await?;
        Ok(user.clone())
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, username, created_at FROM users WHERE id = ?",
            id
        )
        .fetch_optional(self.db.pool())
        .await?;
        Ok(user)
    }

    pub async fn get_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, username, created_at FROM users WHERE username = ?",
            username
        )
        .fetch_optional(self.db.pool())
        .await?;
        Ok(user)
    }

    pub async fn list_all(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as!(
            User,
            "SELECT id, username, created_at FROM users ORDER BY created_at DESC"
        )
        .fetch_all(self.db.pool())
        .await?;
        Ok(users)
    }
}

#[derive(Clone)]
pub struct ExamSessionRepository {
    db: Database,
}

impl ExamSessionRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create(&self, session: &ExamSession) -> Result<ExamSession> {
        sqlx::query(
            "INSERT INTO exam_sessions (id, user_id, exam_title, start_time, end_time, is_suspicious, suspicion_reason, total_questions) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&session.id)
        .bind(&session.user_id)
        .bind(&session.exam_title)
        .bind(session.start_time)
        .bind(session.end_time)
        .bind(session.is_suspicious)
        .bind(&session.suspicion_reason)
        .bind(session.total_questions)
        .execute(self.db.pool())
        .await?;
        Ok(session.clone())
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<ExamSession>> {
        let session = sqlx::query_as!(
            ExamSession,
            r#"SELECT id, user_id, exam_title, start_time, end_time, 
                      is_suspicious as "is_suspicious: bool", suspicion_reason, total_questions 
               FROM exam_sessions WHERE id = ?"#,
            id
        )
        .fetch_optional(self.db.pool())
        .await?;
        Ok(session)
    }

    pub async fn update(&self, session: &ExamSession) -> Result<()> {
        sqlx::query(
            "UPDATE exam_sessions SET user_id = ?, exam_title = ?, start_time = ?, end_time = ?, 
             is_suspicious = ?, suspicion_reason = ?, total_questions = ? WHERE id = ?"
        )
        .bind(&session.user_id)
        .bind(&session.exam_title)
        .bind(session.start_time)
        .bind(session.end_time)
        .bind(session.is_suspicious)
        .bind(&session.suspicion_reason)
        .bind(session.total_questions)
        .bind(&session.id)
        .execute(self.db.pool())
        .await?;
        Ok(())
    }

    pub async fn mark_suspicious(&self, id: &str, reason: &str) -> Result<()> {
        sqlx::query(
            "UPDATE exam_sessions SET is_suspicious = 1, suspicion_reason = ? WHERE id = ?"
        )
        .bind(reason)
        .bind(id)
        .execute(self.db.pool())
        .await?;
        Ok(())
    }

    pub async fn end_session(&self, id: &str, end_time: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            "UPDATE exam_sessions SET end_time = ? WHERE id = ?"
        )
        .bind(end_time)
        .bind(id)
        .execute(self.db.pool())
        .await?;
        Ok(())
    }

    pub async fn list_by_user(&self, user_id: &str) -> Result<Vec<ExamSession>> {
        let sessions = sqlx::query_as!(
            ExamSession,
            r#"SELECT id, user_id, exam_title, start_time, end_time, 
                      is_suspicious as "is_suspicious: bool", suspicion_reason, total_questions 
               FROM exam_sessions WHERE user_id = ? ORDER BY start_time DESC"#,
            user_id
        )
        .fetch_all(self.db.pool())
        .await?;
        Ok(sessions)
    }

    pub async fn list_suspicious(&self) -> Result<Vec<ExamSession>> {
        let sessions = sqlx::query_as!(
            ExamSession,
            r#"SELECT id, user_id, exam_title, start_time, end_time, 
                      is_suspicious as "is_suspicious: bool", suspicion_reason, total_questions 
               FROM exam_sessions WHERE is_suspicious = 1 ORDER BY start_time DESC"#
        )
        .fetch_all(self.db.pool())
        .await?;
        Ok(sessions)
    }

    pub async fn list_all(&self) -> Result<Vec<ExamSession>> {
        let sessions = sqlx::query_as!(
            ExamSession,
            r#"SELECT id, user_id, exam_title, start_time, end_time, 
                      is_suspicious as "is_suspicious: bool", suspicion_reason, total_questions 
               FROM exam_sessions ORDER BY start_time DESC"#
        )
        .fetch_all(self.db.pool())
        .await?;
        Ok(sessions)
    }
}

#[derive(Clone)]
pub struct BehaviorEventRepository {
    db: Database,
}

impl BehaviorEventRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create(&self, event: &BehaviorEvent) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO behavior_events (session_id, event_type, event_time, page_x, page_y, screen_x, screen_y, visibility_state, duration_ms, details) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&event.session_id)
        .bind(event.event_type.as_str())
        .bind(event.event_time)
        .bind(event.page_x)
        .bind(event.page_y)
        .bind(event.screen_x)
        .bind(event.screen_y)
        .bind(&event.visibility_state)
        .bind(event.duration_ms)
        .bind(&event.details)
        .execute(self.db.pool())
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn get_by_session(&self, session_id: &str) -> Result<Vec<BehaviorEvent>> {
        let rows = sqlx::query!(
            "SELECT id, session_id, event_type, event_time, page_x, page_y, screen_x, screen_y, visibility_state, duration_ms, details 
             FROM behavior_events WHERE session_id = ? ORDER BY event_time ASC",
            session_id
        )
        .fetch_all(self.db.pool())
        .await?;

        let events = rows
            .into_iter()
            .map(|row| BehaviorEvent {
                id: Some(row.id),
                session_id: row.session_id,
                event_type: EventType::from_str(&row.event_type),
                event_time: row.event_time,
                page_x: row.page_x,
                page_y: row.page_y,
                screen_x: row.screen_x,
                screen_y: row.screen_y,
                visibility_state: row.visibility_state,
                duration_ms: row.duration_ms,
                details: row.details,
            })
            .collect();

        Ok(events)
    }

    pub async fn get_by_session_and_type(
        &self,
        session_id: &str,
        event_type: &EventType,
    ) -> Result<Vec<BehaviorEvent>> {
        let rows = sqlx::query!(
            "SELECT id, session_id, event_type, event_time, page_x, page_y, screen_x, screen_y, visibility_state, duration_ms, details 
             FROM behavior_events WHERE session_id = ? AND event_type = ? ORDER BY event_time ASC",
            session_id,
            event_type.as_str()
        )
        .fetch_all(self.db.pool())
        .await?;

        let events = rows
            .into_iter()
            .map(|row| BehaviorEvent {
                id: Some(row.id),
                session_id: row.session_id,
                event_type: EventType::from_str(&row.event_type),
                event_time: row.event_time,
                page_x: row.page_x,
                page_y: row.page_y,
                screen_x: row.screen_x,
                screen_y: row.screen_y,
                visibility_state: row.visibility_state,
                duration_ms: row.duration_ms,
                details: row.details,
            })
            .collect();

        Ok(events)
    }

    pub async fn count_by_type(&self, session_id: &str, event_type: &EventType) -> Result<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM behavior_events WHERE session_id = ? AND event_type = ?"
        )
        .bind(session_id)
        .bind(event_type.as_str())
        .fetch_optional(self.db.pool())
        .await?;
        Ok(count.unwrap_or(0))
    }

    pub async fn get_total_away_duration(&self, session_id: &str) -> Result<i64> {
        let total: Option<i64> = sqlx::query_scalar(
            "SELECT COALESCE(SUM(duration_ms), 0) FROM behavior_events 
             WHERE session_id = ? AND event_type IN ('visibility_change', 'window_blur', 'page_blur') 
             AND visibility_state = 'hidden'"
        )
        .bind(session_id)
        .fetch_optional(self.db.pool())
        .await?;
        Ok(total.unwrap_or(0))
    }

    pub async fn get_max_away_duration(&self, session_id: &str) -> Result<i64> {
        let max: Option<i64> = sqlx::query_scalar(
            "SELECT COALESCE(MAX(duration_ms), 0) FROM behavior_events 
             WHERE session_id = ? AND event_type IN ('visibility_change', 'window_blur', 'page_blur') 
             AND visibility_state = 'hidden' AND duration_ms IS NOT NULL"
        )
        .bind(session_id)
        .fetch_optional(self.db.pool())
        .await?;
        Ok(max.unwrap_or(0))
    }
}

#[derive(Clone)]
pub struct QuestionAnswerRepository {
    db: Database,
}

impl QuestionAnswerRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create(&self, answer: &QuestionAnswer) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO question_answers (session_id, question_id, answer, answered_at) 
             VALUES (?, ?, ?, ?)"
        )
        .bind(&answer.session_id)
        .bind(answer.question_id)
        .bind(&answer.answer)
        .bind(answer.answered_at)
        .execute(self.db.pool())
        .await?;
        Ok(result.last_insert_rowid())
    }

    pub async fn update(&self, answer: &QuestionAnswer) -> Result<()> {
        sqlx::query(
            "UPDATE question_answers SET answer = ?, answered_at = ? WHERE session_id = ? AND question_id = ?"
        )
        .bind(&answer.answer)
        .bind(answer.answered_at)
        .bind(&answer.session_id)
        .bind(answer.question_id)
        .execute(self.db.pool())
        .await?;
        Ok(())
    }

    pub async fn get_by_session(&self, session_id: &str) -> Result<Vec<QuestionAnswer>> {
        let answers = sqlx::query_as!(
            QuestionAnswer,
            "SELECT id, session_id, question_id, answer, answered_at 
             FROM question_answers WHERE session_id = ? ORDER BY question_id ASC",
            session_id
        )
        .fetch_all(self.db.pool())
        .await?;
        Ok(answers)
    }

    pub async fn get_by_question(
        &self,
        session_id: &str,
        question_id: i32,
    ) -> Result<Option<QuestionAnswer>> {
        let answer = sqlx::query_as!(
            QuestionAnswer,
            "SELECT id, session_id, question_id, answer, answered_at 
             FROM question_answers WHERE session_id = ? AND question_id = ?",
            session_id,
            question_id
        )
        .fetch_optional(self.db.pool())
        .await?;
        Ok(answer)
    }
}
