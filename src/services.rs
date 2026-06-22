use crate::db::*;
use crate::models::*;
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;

#[derive(Clone)]
pub struct DetectionConfig {
    pub max_visibility_changes: usize,
    pub max_tab_switches: usize,
    pub max_window_blurs: usize,
    pub max_away_duration_ms: i64,
    pub max_single_away_duration_ms: i64,
    pub max_copy_events: usize,
    pub max_paste_events: usize,
    pub risk_score_threshold: f32,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            max_visibility_changes: 5,
            max_tab_switches: 3,
            max_window_blurs: 5,
            max_away_duration_ms: 60_000,
            max_single_away_duration_ms: 10_000,
            max_copy_events: 2,
            max_paste_events: 3,
            risk_score_threshold: 50.0,
        }
    }
}

#[derive(Clone)]
pub struct BehaviorDetectionService {
    event_repo: BehaviorEventRepository,
    session_repo: ExamSessionRepository,
    config: DetectionConfig,
}

impl BehaviorDetectionService {
    pub fn new(
        event_repo: BehaviorEventRepository,
        session_repo: ExamSessionRepository,
        config: DetectionConfig,
    ) -> Self {
        Self {
            event_repo,
            session_repo,
            config,
        }
    }

    pub async fn analyze_session(&self, session_id: &str) -> Result<SuspicionAnalysis> {
        let metrics = self.calculate_metrics(session_id).await?;
        let (is_suspicious, risk_score, reasons) = self.evaluate_metrics(&metrics);

        let analysis = SuspicionAnalysis {
            is_suspicious,
            risk_score,
            reasons,
            metrics,
        };

        if is_suspicious {
            let reason_str = analysis.reasons.join("; ");
            self.session_repo
                .mark_suspicious(session_id, &reason_str)
                .await?;
        }

        Ok(analysis)
    }

    pub async fn analyze_event(&self, event: &BehaviorEvent) -> Result<bool> {
        let session_id = &event.session_id;
        let metrics = self.calculate_metrics(session_id).await?;
        let (is_suspicious, _, _) = self.evaluate_metrics(&metrics);

        if is_suspicious {
            if let Some(session) = self.session_repo.get_by_id(session_id).await? {
                if !session.is_suspicious {
                    let analysis = self.analyze_session(session_id).await?;
                    let reason_str = analysis.reasons.join("; ");
                    self.session_repo
                        .mark_suspicious(session_id, &reason_str)
                        .await?;
                }
            }
        }

        Ok(is_suspicious)
    }

    async fn calculate_metrics(&self, session_id: &str) -> Result<BehaviorMetrics> {
        let events = self.event_repo.get_by_session(session_id).await?;

        let mut visibility_changes = 0;
        let mut tab_switches = 0;
        let mut window_blurs = 0;
        let mut copy_events = 0;
        let mut paste_events = 0;
        let mut away_durations: Vec<i64> = Vec::new();

        let mut last_away_start: Option<chrono::DateTime<Utc>> = None;

        for event in &events {
            match event.event_type {
                EventType::VisibilityChange => {
                    visibility_changes += 1;
                    if event.visibility_state.as_deref() == Some("hidden") {
                        last_away_start = Some(event.event_time);
                    } else if event.visibility_state.as_deref() == Some("visible") {
                        if let Some(start) = last_away_start.take() {
                            let duration = (event.event_time - start).num_milliseconds();
                            away_durations.push(duration.max(0));
                        }
                    }
                }
                EventType::TabSwitch => {
                    tab_switches += 1;
                }
                EventType::WindowBlur => {
                    window_blurs += 1;
                    last_away_start = Some(event.event_time);
                }
                EventType::WindowFocus | EventType::PageFocus => {
                    if let Some(start) = last_away_start.take() {
                        let duration = (event.event_time - start).num_milliseconds();
                        away_durations.push(duration.max(0));
                    }
                }
                EventType::Copy => {
                    copy_events += 1;
                }
                EventType::Paste => {
                    paste_events += 1;
                }
                _ => {}
            }
        }

        let total_away_duration_ms: i64 = away_durations.iter().sum();
        let max_away_duration_ms: i64 = *away_durations.iter().max().unwrap_or(&0);
        let average_away_duration_ms: f64 = if !away_durations.is_empty() {
            total_away_duration_ms as f64 / away_durations.len() as f64
        } else {
            0.0
        };

        Ok(BehaviorMetrics {
            total_events: events.len(),
            visibility_changes,
            tab_switches,
            window_blurs,
            copy_events,
            paste_events,
            total_away_duration_ms,
            max_away_duration_ms,
            average_away_duration_ms,
            away_count: away_durations.len(),
        })
    }

    fn evaluate_metrics(&self, metrics: &BehaviorMetrics) -> (bool, f32, Vec<String>) {
        let mut reasons: Vec<String> = Vec::new();
        let mut risk_score: f32 = 0.0;
        let mut violations: HashMap<&str, usize> = HashMap::new();

        if metrics.visibility_changes > self.config.max_visibility_changes {
            violations.insert("visibility_changes", metrics.visibility_changes);
            risk_score += 20.0;
            reasons.push(format!(
                "页面可见性变化次数过多: {} 次 (阈值: {})",
                metrics.visibility_changes, self.config.max_visibility_changes
            ));
        }

        if metrics.tab_switches > self.config.max_tab_switches {
            violations.insert("tab_switches", metrics.tab_switches);
            risk_score += 25.0;
            reasons.push(format!(
                "标签页切换次数过多: {} 次 (阈值: {})",
                metrics.tab_switches, self.config.max_tab_switches
            ));
        }

        if metrics.window_blurs > self.config.max_window_blurs {
            violations.insert("window_blurs", metrics.window_blurs);
            risk_score += 15.0;
            reasons.push(format!(
                "窗口失焦次数过多: {} 次 (阈值: {})",
                metrics.window_blurs, self.config.max_window_blurs
            ));
        }

        if metrics.total_away_duration_ms > self.config.max_away_duration_ms {
            violations.insert("total_away", metrics.total_away_duration_ms as usize);
            risk_score += 30.0;
            reasons.push(format!(
                "累计离开页面时间过长: {:.1} 秒 (阈值: {:.1} 秒)",
                metrics.total_away_duration_ms as f64 / 1000.0,
                self.config.max_away_duration_ms as f64 / 1000.0
            ));
        }

        if metrics.max_away_duration_ms > self.config.max_single_away_duration_ms {
            violations.insert("max_away", metrics.max_away_duration_ms as usize);
            risk_score += 25.0;
            reasons.push(format!(
                "单次离开页面时间过长: {:.1} 秒 (阈值: {:.1} 秒)",
                metrics.max_away_duration_ms as f64 / 1000.0,
                self.config.max_single_away_duration_ms as f64 / 1000.0
            ));
        }

        if metrics.copy_events > self.config.max_copy_events {
            violations.insert("copy_events", metrics.copy_events);
            risk_score += 35.0;
            reasons.push(format!(
                "复制操作次数过多: {} 次 (阈值: {})",
                metrics.copy_events, self.config.max_copy_events
            ));
        }

        if metrics.paste_events > self.config.max_paste_events {
            violations.insert("paste_events", metrics.paste_events);
            risk_score += 40.0;
            reasons.push(format!(
                "粘贴操作次数过多: {} 次 (阈值: {})",
                metrics.paste_events, self.config.max_paste_events
            ));
        }

        let is_suspicious = risk_score >= self.config.risk_score_threshold || !violations.is_empty();

        (is_suspicious, risk_score.min(100.0), reasons)
    }
}

#[derive(Clone)]
pub struct ExamService {
    user_repo: UserRepository,
    session_repo: ExamSessionRepository,
    event_repo: BehaviorEventRepository,
    answer_repo: QuestionAnswerRepository,
    detection_service: BehaviorDetectionService,
}

impl ExamService {
    pub fn new(
        user_repo: UserRepository,
        session_repo: ExamSessionRepository,
        event_repo: BehaviorEventRepository,
        answer_repo: QuestionAnswerRepository,
        detection_service: BehaviorDetectionService,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
            event_repo,
            answer_repo,
            detection_service,
        }
    }

    pub async fn create_user(&self, username: String) -> Result<User> {
        if let Some(existing) = self.user_repo.get_by_username(&username).await? {
            return Ok(existing);
        }
        let user = User::new(username);
        self.user_repo.create(&user).await
    }

    pub async fn get_user(&self, user_id: &str) -> Result<Option<User>> {
        self.user_repo.get_by_id(user_id).await
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        self.user_repo.list_all().await
    }

    pub async fn create_session(
        &self,
        user_id: String,
        exam_title: String,
        total_questions: i32,
    ) -> Result<ExamSession> {
        let session = ExamSession::new(user_id, exam_title, total_questions);
        self.session_repo.create(&session).await
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<ExamSession>> {
        self.session_repo.get_by_id(session_id).await
    }

    pub async fn get_session_detail(&self, session_id: &str) -> Result<Option<SessionDetailResponse>> {
        let session = match self.session_repo.get_by_id(session_id).await? {
            Some(s) => s,
            None => return Ok(None),
        };

        let events = self.event_repo.get_by_session(session_id).await?;
        let answers = self.answer_repo.get_by_session(session_id).await?;
        let analysis = self.detection_service.analyze_session(session_id).await?;

        Ok(Some(SessionDetailResponse {
            session,
            events,
            answers,
            analysis,
        }))
    }

    pub async fn list_sessions(&self, user_id: Option<&str>) -> Result<Vec<ExamSession>> {
        match user_id {
            Some(uid) => self.session_repo.list_by_user(uid).await,
            None => self.session_repo.list_all().await,
        }
    }

    pub async fn list_suspicious_sessions(&self) -> Result<Vec<ExamSession>> {
        self.session_repo.list_suspicious().await
    }

    pub async fn report_event(&self, req: ReportEventRequest) -> Result<ReportEventResponse> {
        let event_type = EventType::from_str(&req.event_type);
        let event_time = req.event_time.unwrap_or_else(Utc::now);

        let event = BehaviorEvent {
            id: None,
            session_id: req.session_id.clone(),
            event_type: event_type.clone(),
            event_time,
            page_x: req.page_x,
            page_y: req.page_y,
            screen_x: req.screen_x,
            screen_y: req.screen_y,
            visibility_state: req.visibility_state,
            duration_ms: req.duration_ms,
            details: req.details,
        };

        let event_id = self.event_repo.create(&event).await?;
        let is_suspicious = self.detection_service.analyze_event(&event).await?;

        Ok(ReportEventResponse {
            event_id,
            session_id: event.session_id,
            event_type: event.event_type.as_str().to_string(),
            event_time: event.event_time,
            is_suspicious,
        })
    }

    pub async fn submit_answer(&self, req: SubmitAnswerRequest) -> Result<()> {
        let existing = self
            .answer_repo
            .get_by_question(&req.session_id, req.question_id)
            .await?;

        let answer = QuestionAnswer {
            id: existing.as_ref().and_then(|a| a.id),
            session_id: req.session_id,
            question_id: req.question_id,
            answer: Some(req.answer),
            answered_at: Some(Utc::now()),
        };

        if existing.is_some() {
            self.answer_repo.update(&answer).await?;
        } else {
            self.answer_repo.create(&answer).await?;
        }

        Ok(())
    }

    pub async fn end_session(&self, session_id: &str) -> Result<Option<SuspicionAnalysis>> {
        self.session_repo.end_session(session_id, Utc::now()).await?;
        let analysis = self.detection_service.analyze_session(session_id).await?;
        Ok(Some(analysis))
    }

    pub async fn mark_suspicious(&self, session_id: &str, reason: &str) -> Result<()> {
        self.session_repo.mark_suspicious(session_id, reason).await
    }

    pub async fn get_session_analysis(&self, session_id: &str) -> Result<SuspicionAnalysis> {
        self.detection_service.analyze_session(session_id).await
    }
}
