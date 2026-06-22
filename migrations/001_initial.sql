CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS exam_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    exam_title TEXT NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    is_suspicious INTEGER DEFAULT 0,
    suspicion_reason TEXT,
    total_questions INTEGER DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS behavior_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_time DATETIME NOT NULL,
    page_x INTEGER,
    page_y INTEGER,
    screen_x INTEGER,
    screen_y INTEGER,
    visibility_state TEXT,
    duration_ms INTEGER,
    details TEXT,
    FOREIGN KEY (session_id) REFERENCES exam_sessions(id)
);

CREATE TABLE IF NOT EXISTS question_answers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    question_id INTEGER NOT NULL,
    answer TEXT,
    answered_at DATETIME,
    FOREIGN KEY (session_id) REFERENCES exam_sessions(id)
);

CREATE INDEX IF NOT EXISTS idx_behavior_session ON behavior_events(session_id);
CREATE INDEX IF NOT EXISTS idx_behavior_type ON behavior_events(event_type);
CREATE INDEX IF NOT EXISTS idx_exam_user ON exam_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_exam_suspicious ON exam_sessions(is_suspicious);
