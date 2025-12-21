use crate::config::Config;
use crate::models::{InputMode, LogEntry};
use crate::storage;
use chrono::{DateTime, Local};
use ratatui::widgets::ListState;
use std::collections::HashMap;
use tui_textarea::TextArea;

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
    pub show_path_popup: bool,

    // 뽀모도로 입력 팝업
    pub show_pomodoro_popup: bool,
    pub pomodoro_input: String,

    // 뽀모도로 종료 알림 (이 시간까지 알림 표시 & 입력 차단)
    // 뽀모도로 종료 알림 (이 시간까지 알림 표시 & 입력 차단)
    pub pomodoro_alert_expiry: Option<DateTime<Local>>,

    // 설정 (안내 문구 등)
    // 설정 (안내 문구 등)
    pub config: Config,
    pub path_list_state: ListState,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let config = Config::load();

        let mut textarea = TextArea::default();
        textarea.set_placeholder_text(&config.placeholders.editing);

        let logs =
            storage::read_today_entries(&config.data.log_path).unwrap_or_else(|_| Vec::new());
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
            let already_checked =
                storage::is_carryover_done(&config.data.log_path).unwrap_or(false);
            if !already_checked {
                if let Ok(todos) = storage::get_last_file_pending_todos(&config.data.log_path) {
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
            show_path_popup: false,
            show_pomodoro_popup: false,
            pomodoro_input: String::new(),
            pomodoro_alert_expiry: None,
            config,
            path_list_state: ListState::default(),
        }
    }

    pub fn update_logs(&mut self) {
        if let Ok(logs) = storage::read_today_entries(&self.config.data.log_path) {
            self.logs = logs;
            self.is_search_result = false;
            if !self.logs.is_empty() {
                self.logs_state.select(Some(self.logs.len() - 1));
            }
        }
    }

    pub fn scroll_up(&mut self) {
        if self.logs.is_empty() {
            return;
        }

        let i = match self.logs_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.logs_state.select(Some(i));
    }

    pub fn scroll_down(&mut self) {
        if self.logs.is_empty() {
            return;
        }

        let i = match self.logs_state.selected() {
            Some(i) => {
                if i >= self.logs.len() - 1 {
                    self.logs.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.logs_state.select(Some(i));
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn transition_to(&mut self, mode: InputMode) {
        // Mode specific entry logic
        match mode {
            InputMode::Navigate => {
                // Search 모드에서 돌아올 때는 입력창 내용을(검색어) 비워야 함
                if self.input_mode == InputMode::Search {
                    self.textarea = TextArea::default();
                }
                self.textarea
                    .set_placeholder_text(&self.config.placeholders.navigate);
            }
            InputMode::Editing => {
                self.textarea
                    .set_placeholder_text(&self.config.placeholders.editing);
                // 검색 결과 화면에서 편집으로 넘어갈 때 전체 로그로 복귀
                if self.is_search_result {
                    self.update_logs();
                }
            }
            InputMode::Search => {
                self.textarea = TextArea::default(); // 검색어 입력 위해 초기화
                self.textarea
                    .set_placeholder_text(&self.config.placeholders.search);
            }
        }
        self.input_mode = mode;
    }
}
