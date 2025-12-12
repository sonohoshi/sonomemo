use tui_textarea::TextArea;
use ratatui::widgets::ListState;
use chrono::{DateTime, Local};
use std::collections::HashMap;
use crate::storage;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Search,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mood {
    Happy,
    Neutral,
    Stressed,
    Focused,
    Tired,
}

impl Mood {
    pub fn all() -> Vec<Mood> {
        vec![Mood::Happy, Mood::Neutral, Mood::Stressed, Mood::Focused, Mood::Tired]
    }
    
    pub fn to_str(&self) -> &'static str {
        match self {
            Mood::Happy => "😊 조음",
            Mood::Neutral => "😐 걍그럼",
            Mood::Stressed => "😫 구림",
            Mood::Focused => "🧐 집중",
            Mood::Tired => "😴 피곤",
        }
    }
}

#[derive(Clone)]
pub struct LogEntry {
    pub content: String,
    pub file_path: String,
    pub line_number: usize,
}

pub struct App<'a> {
    pub input_mode: InputMode,
    pub textarea: TextArea<'a>,
    pub logs: Vec<LogEntry>,
    pub logs_state: ListState,
    pub show_mood_popup: bool,
    pub mood_list_state: ListState,
    pub show_todo_popup: bool, // 할 일 요약 팝업
    pub pending_todos: Vec<String>,
    pub todo_list_state: ListState,
    pub show_tag_popup: bool,
    pub tags: Vec<(String, usize)>, // (태그명, 횟수)
    pub tag_list_state: ListState,
    pub is_search_result: bool,
    pub should_quit: bool,
    
    // 로컬 파워 기능
    pub pomodoro_end: Option<DateTime<Local>>,
    pub show_activity_popup: bool,
    pub activity_data: HashMap<String, usize>, // "YYYY-MM-DD" -> line_count
    
    // 뽀모도로 입력 팝업
    pub show_pomodoro_popup: bool,
    pub pomodoro_input: String,
    
    // 뽀모도로 종료 알림 (이 시간까지 알림 표시 & 입력 차단)
    pub pomodoro_alert_expiry: Option<DateTime<Local>>,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let mut textarea = TextArea::default();
        textarea.set_placeholder_text("여기에 메모를 입력하세요... (Enter: 저장, Esc: 모드 전환, :q 종료)");
        
        let logs = storage::read_today_entries().unwrap_or_else(|_| Vec::new());
        let mut logs_state = ListState::default();
        if !logs.is_empty() {
            logs_state.select(Some(logs.len() - 1));
        }

        // 이미 기분 로그가 있는지 확인
        let has_mood = logs.iter().any(|log| log.content.contains("Mood: "));
        let show_mood_popup = !has_mood;
        
        let mut mood_list_state = ListState::default();
        if show_mood_popup {
            mood_list_state.select(Some(0));
        }

        let mut show_todo_popup = false;
        let mut pending_todos = Vec::new();



        if !show_mood_popup {
            // 기분 팝업이 안 뜨는 경우(이미 기분 입력함)에도 체크할지, 
            // 아니면 그냥 뜰 때만 체크할지는 정책 나름이지만, 일단 시작 시 체크
            // 단, 오늘 이미 체크했으면 다시 묻지 않음
            let already_checked = storage::is_carryover_done().unwrap_or(false);
            if !already_checked {
                if let Ok(todos) = storage::get_last_file_pending_todos() {
                    if !todos.is_empty() {
                        pending_todos = todos;
                        show_todo_popup = true;
                    }
                }
            }
        }

        let input_mode = InputMode::Editing;

        App {
            input_mode,
            textarea,
            logs,
            logs_state,
            show_mood_popup,
            mood_list_state,
            show_todo_popup,
            pending_todos,
            todo_list_state: ListState::default(),
            show_tag_popup: false,
            tags: Vec::new(),
            tag_list_state: ListState::default(),
            is_search_result: false,
            should_quit: false,
            pomodoro_end: None,
            show_activity_popup: false,
            activity_data: HashMap::new(),
            show_pomodoro_popup: false,
            pomodoro_input: String::new(),
            pomodoro_alert_expiry: None,
        }
    }

    pub fn update_logs(&mut self) {
        if let Ok(logs) = storage::read_today_entries() {
            self.logs = logs;
            self.is_search_result = false;
            if !self.logs.is_empty() {
                self.logs_state.select(Some(self.logs.len() - 1));
            }
        }
    }

    pub fn on_tick(&mut self) {}

    pub fn scroll_up(&mut self) {
        if self.logs.is_empty() { return; }
        
        let i = match self.logs_state.selected() {
            Some(i) => if i == 0 { 0 } else { i - 1 },
            None => 0,
        };
        self.logs_state.select(Some(i));
    }

    pub fn scroll_down(&mut self) {
        if self.logs.is_empty() { return; }

        let i = match self.logs_state.selected() {
            Some(i) => if i >= self.logs.len() - 1 { self.logs.len() - 1 } else { i + 1 },
            None => 0,
        };
        self.logs_state.select(Some(i));
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
