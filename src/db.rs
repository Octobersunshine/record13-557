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

#[derive(Clone)]
pub struct LeaderboardRepository {
    db: Database,
}

impl LeaderboardRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn get_suspicious_user_ranking(&self) -> Result<Vec<SuspiciousUserRank>> {
        let rows = sqlx::query!(
            r#"SELECT 
                u.id as user_id,
                u.username,
                COUNT(es.id) as total_sessions,
                SUM(CASE WHEN es.is_suspicious = 1 THEN 1 ELSE 0 END) as suspicious_sessions,
                MAX(es.start_time) as latest_suspicious_time
               FROM users u
               INNER JOIN exam_sessions es ON es.user_id = u.id
               GROUP BY u.id, u.username
               HAVING suspicious_sessions > 0
               ORDER BY suspicious_sessions DESC, u.username ASC"#
        )
        .fetch_all(self.db.pool())
        .await?;

        let mut ranks = Vec::new();
        for row in rows {
            let user_id = row.user_id;
            let username = row.username;
            let total_sessions = row.total_sessions;
            let suspicious_sessions = row.suspicious_sessions.unwrap_or(0);
            let latest_time = row.latest_suspicious_time;

            let suspicious_rate = if total_sessions > 0 {
                suspicious_sessions as f64 / total_sessions as f64 * 100.0
            } else {
                0.0
            };

            let session_rows = sqlx::query!(
                "SELECT id, is_suspicious, suspicion_reason FROM exam_sessions WHERE user_id = ? AND is_suspicious = 1",
                user_id
            )
            .fetch_all(self.db.pool())
            .await?;

            let mut max_risk_score: f32 = 0.0;
            let mut all_reasons: Vec<String> = Vec::new();

            for sr in &session_rows {
                if let Some(reason) = &sr.suspicion_reason {
                    for r in reason.split("; ") {
                        let trimmed = r.trim().to_string();
                        if !trimmed.is_empty() && !all_reasons.contains(&trimmed) {
                            all_reasons.push(trimmed);
                        }
                    }
                }
            }

            let event_stats = sqlx::query!(
                r#"SELECT 
                    COALESCE(SUM(CASE WHEN event_type = 'visibility_change' THEN 1 ELSE 0 END), 0) as visibility_changes,
                    COALESCE(SUM(CASE WHEN event_type = 'tab_switch' THEN 1 ELSE 0 END), 0) as tab_switches,
                    COALESCE(SUM(CASE WHEN event_type = 'window_blur' THEN 1 ELSE 0 END), 0) as window_blurs,
                    COALESCE(SUM(CASE WHEN event_type = 'copy' THEN 1 ELSE 0 END), 0) as copy_events,
                    COALESCE(SUM(CASE WHEN event_type = 'paste' THEN 1 ELSE 0 END), 0) as paste_events,
                    COALESCE(SUM(CASE WHEN event_type IN ('visibility_change', 'window_blur', 'page_blur') AND visibility_state = 'hidden' THEN COALESCE(duration_ms, 0) ELSE 0 END), 0) as total_away_ms
                   FROM behavior_events 
                   WHERE session_id IN (SELECT id FROM exam_sessions WHERE user_id = ?)"#,
                user_id
            )
            .fetch_one(self.db.pool())
            .await?;

            let total_away_sec = (event_stats.total_away_ms.unwrap_or(0) as f64) / 1000.0;

            ranks.push(SuspiciousUserRank {
                user_id,
                username,
                total_sessions,
                suspicious_sessions,
                suspicious_rate,
                max_risk_score,
                total_visibility_changes: event_stats.visibility_changes.unwrap_or(0),
                total_tab_switches: event_stats.tab_switches.unwrap_or(0),
                total_window_blurs: event_stats.window_blurs.unwrap_or(0),
                total_copy_events: event_stats.copy_events.unwrap_or(0),
                total_paste_events: event_stats.paste_events.unwrap_or(0),
                total_away_duration_sec: total_away_sec,
                latest_suspicious_time: latest_time,
                suspicion_reasons: all_reasons,
            });
        }

        ranks.sort_by(|a, b| {
            b.suspicious_sessions
                .cmp(&a.suspicious_sessions)
                .then_with(|| b.suspicious_rate.partial_cmp(&a.suspicious_rate).unwrap_or(std::cmp::Ordering::Equal))
        });

        Ok(ranks)
    }

    pub async fn count_suspicious_sessions(&self) -> Result<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM exam_sessions WHERE is_suspicious = 1"
        )
        .fetch_optional(self.db.pool())
        .await?;
        Ok(count.unwrap_or(0))
    }

    pub async fn get_all_suspicious_sessions_detail(&self) -> Result<Vec<ExportAnomalousRecord>> {
        let sessions = sqlx::query!(
            r#"SELECT id, user_id, exam_title, start_time, end_time, 
                      is_suspicious as "is_suspicious: bool", suspicion_reason, total_questions 
               FROM exam_sessions WHERE is_suspicious = 1 ORDER BY start_time DESC"#
        )
        .fetch_all(self.db.pool())
        .await?;

        let mut records = Vec::new();

        for (idx, session) in sessions.iter().enumerate() {
            let username = sqlx::query_scalar::<_, String>(
                "SELECT username FROM users WHERE id = ?"
            )
            .bind(&session.user_id)
            .fetch_optional(self.db.pool())
            .await?
            .unwrap_or_else(|| "unknown".to_string());

            let event_stats = sqlx::query!(
                r#"SELECT 
                    COALESCE(SUM(CASE WHEN event_type = 'visibility_change' THEN 1 ELSE 0 END), 0) as visibility_changes,
                    COALESCE(SUM(CASE WHEN event_type = 'tab_switch' THEN 1 ELSE 0 END), 0) as tab_switches,
                    COALESCE(SUM(CASE WHEN event_type = 'window_blur' THEN 1 ELSE 0 END), 0) as window_blurs,
                    COALESCE(SUM(CASE WHEN event_type = 'copy' THEN 1 ELSE 0 END), 0) as copy_events,
                    COALESCE(SUM(CASE WHEN event_type = 'paste' THEN 1 ELSE 0 END), 0) as paste_events,
                    COALESCE(SUM(CASE WHEN event_type IN ('visibility_change', 'window_blur', 'page_blur') AND visibility_state = 'hidden' THEN COALESCE(duration_ms, 0) ELSE 0 END), 0) as total_away_ms,
                    COALESCE(MAX(CASE WHEN event_type IN ('visibility_change', 'window_blur', 'page_blur') AND visibility_state = 'hidden' AND duration_ms IS NOT NULL THEN duration_ms ELSE 0 END), 0) as max_away_ms,
                    COALESCE(SUM(CASE WHEN event_type = 'copy' THEN COALESCE(LENGTH(details), 0) ELSE 0 END), 0) as total_copy_chars,
                    COALESCE(SUM(CASE WHEN event_type = 'paste' THEN COALESCE(LENGTH(details), 0) ELSE 0 END), 0) as total_paste_chars
                   FROM behavior_events WHERE session_id = ?"#,
                session.id
            )
            .fetch_one(self.db.pool())
            .await?;

            let suspicious_matches = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM behavior_events WHERE session_id = ? AND event_type IN ('copy', 'paste') AND (details LIKE '%答案%' OR details LIKE '%解析%' OR details LIKE '%题库%' OR details LIKE '%搜题%' OR details LIKE '%作弊%' OR details LIKE '%参考答案%' OR details LIKE '%答案大全%' OR details LIKE '%考试答案%')"
            )
            .bind(&session.id)
            .fetch_one(self.db.pool())
            .await?;

            records.push(ExportAnomalousRecord {
                rank: idx + 1,
                user_id: session.user_id.clone(),
                username,
                session_id: session.id.clone(),
                exam_title: session.exam_title.clone(),
                start_time: session.start_time,
                end_time: session.end_time,
                risk_score: 0.0,
                suspicion_reason: session.suspicion_reason.clone().unwrap_or_default(),
                visibility_changes: event_stats.visibility_changes.unwrap_or(0),
                tab_switches: event_stats.tab_switches.unwrap_or(0),
                window_blurs: event_stats.window_blurs.unwrap_or(0),
                copy_events: event_stats.copy_events.unwrap_or(0),
                paste_events: event_stats.paste_events.unwrap_or(0),
                total_away_duration_sec: event_stats.total_away_ms.unwrap_or(0) as f64 / 1000.0,
                max_away_duration_sec: event_stats.max_away_ms.unwrap_or(0) as f64 / 1000.0,
                total_copy_characters: event_stats.total_copy_chars.unwrap_or(0),
                total_paste_characters: event_stats.total_paste_chars.unwrap_or(0),
                suspicious_content_matches: suspicious_matches,
            });
        }

        Ok(records)
    }
}
