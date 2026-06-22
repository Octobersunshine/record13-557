use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            username,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamSession {
    pub id: String,
    pub user_id: String,
    pub exam_title: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub is_suspicious: bool,
    pub suspicion_reason: Option<String>,
    pub total_questions: i32,
}

impl ExamSession {
    pub fn new(user_id: String, exam_title: String, total_questions: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            exam_title,
            start_time: Utc::now(),
            end_time: None,
            is_suspicious: false,
            suspicion_reason: None,
            total_questions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    VisibilityChange,
    PageBlur,
    PageFocus,
    TabSwitch,
    WindowBlur,
    WindowFocus,
    MouseLeave,
    MouseEnter,
    KeyDown,
    Copy,
    Paste,
    Cut,
    ContextMenu,
    Print,
    FullscreenChange,
    ScreenChange,
    Custom,
}

impl EventType {
    pub fn as_str(&self) -> &str {
        match self {
            EventType::VisibilityChange => "visibility_change",
            EventType::PageBlur => "page_blur",
            EventType::PageFocus => "page_focus",
            EventType::TabSwitch => "tab_switch",
            EventType::WindowBlur => "window_blur",
            EventType::WindowFocus => "window_focus",
            EventType::MouseLeave => "mouse_leave",
            EventType::MouseEnter => "mouse_enter",
            EventType::KeyDown => "key_down",
            EventType::Copy => "copy",
            EventType::Paste => "paste",
            EventType::Cut => "cut",
            EventType::ContextMenu => "context_menu",
            EventType::Print => "print",
            EventType::FullscreenChange => "fullscreen_change",
            EventType::ScreenChange => "screen_change",
            EventType::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "visibility_change" => EventType::VisibilityChange,
            "page_blur" => EventType::PageBlur,
            "page_focus" => EventType::PageFocus,
            "tab_switch" => EventType::TabSwitch,
            "window_blur" => EventType::WindowBlur,
            "window_focus" => EventType::WindowFocus,
            "mouse_leave" => EventType::MouseLeave,
            "mouse_enter" => EventType::MouseEnter,
            "key_down" => EventType::KeyDown,
            "copy" => EventType::Copy,
            "paste" => EventType::Paste,
            "cut" => EventType::Cut,
            "context_menu" => EventType::ContextMenu,
            "print" => EventType::Print,
            "fullscreen_change" => EventType::FullscreenChange,
            "screen_change" => EventType::ScreenChange,
            _ => EventType::Custom,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorEvent {
    pub id: Option<i64>,
    pub session_id: String,
    pub event_type: EventType,
    pub event_time: DateTime<Utc>,
    pub page_x: Option<i32>,
    pub page_y: Option<i32>,
    pub screen_x: Option<i32>,
    pub screen_y: Option<i32>,
    pub visibility_state: Option<String>,
    pub duration_ms: Option<i64>,
    pub details: Option<String>,
}

impl BehaviorEvent {
    pub fn new(session_id: String, event_type: EventType) -> Self {
        Self {
            id: None,
            session_id,
            event_type,
            event_time: Utc::now(),
            page_x: None,
            page_y: None,
            screen_x: None,
            screen_y: None,
            visibility_state: None,
            duration_ms: None,
            details: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionAnswer {
    pub id: Option<i64>,
    pub session_id: String,
    pub question_id: i32,
    pub answer: Option<String>,
    pub answered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub user_id: String,
    pub exam_title: String,
    pub total_questions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub user_id: String,
    pub exam_title: String,
    pub start_time: DateTime<Utc>,
    pub total_questions: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEventRequest {
    pub session_id: String,
    pub event_type: String,
    pub event_time: Option<DateTime<Utc>>,
    pub page_x: Option<i32>,
    pub page_y: Option<i32>,
    pub screen_x: Option<i32>,
    pub screen_y: Option<i32>,
    pub visibility_state: Option<String>,
    pub duration_ms: Option<i64>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEventResponse {
    pub event_id: i64,
    pub session_id: String,
    pub event_type: String,
    pub event_time: DateTime<Utc>,
    pub is_suspicious: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitAnswerRequest {
    pub session_id: String,
    pub question_id: i32,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndSessionRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetailResponse {
    pub session: ExamSession,
    pub events: Vec<BehaviorEvent>,
    pub answers: Vec<QuestionAnswer>,
    pub analysis: SuspicionAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspicionAnalysis {
    pub is_suspicious: bool,
    pub risk_score: f32,
    pub reasons: Vec<String>,
    pub metrics: BehaviorMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorMetrics {
    pub total_events: usize,
    pub visibility_changes: usize,
    pub tab_switches: usize,
    pub window_blurs: usize,
    pub copy_events: usize,
    pub paste_events: usize,
    pub total_away_duration_ms: i64,
    pub max_away_duration_ms: i64,
    pub average_away_duration_ms: f64,
    pub away_count: usize,
    pub frequent_switches_1min: usize,
    pub frequent_switches_5min: usize,
    pub max_switches_per_minute: usize,
    pub total_copy_characters: usize,
    pub max_single_copy_characters: usize,
    pub frequent_copy_1min: usize,
    pub total_paste_characters: usize,
    pub paste_to_answer_ratio: f64,
    pub rapid_succession_events: usize,
    pub suspicious_content_matches: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
}
