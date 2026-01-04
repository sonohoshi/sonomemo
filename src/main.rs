use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
        KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use std::{error::Error, io};

mod app;
mod config;
mod models;
mod storage;
mod ui;

use crate::config::key_match;
use app::App;
use chrono::{Duration, Local};
use models::{InputMode, Mood};

fn main() -> Result<(), Box<dyn Error>> {
    // 앱 초기화 및 설정 로드
    let mut app = App::new();

    // 터미널 초기화
    // 터미널 초기화
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture,)?;

    // 키보드 향상 플래그는 지원되지 않는 터미널(예: Windows Legacy Console)에서 에러를 뱉을 수 있음.
    // 에러가 발생해도 앱 실행엔 지장이 없으므로 무시함.
    let _ = execute!(
        stdout,
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS,
        )
    );

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 앱 실행
    let res = run_app(&mut terminal, &mut app);

    // 터미널 복구
    disable_raw_mode()?;

    // 종료 시에도 플래그 해제 시도 (실패해도 무방)
    let _ = execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags);

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        // 뽀모도로 타이머 및 알림 체크
        check_timers(app);

        terminal.draw(|f| ui::ui(f, app))?;

        // 알림 표시 중일 때는 입력을 아예 받지 않음 (강제 휴식/주목)
        if app.pomodoro_alert_expiry.is_some() {
            if event::poll(std::time::Duration::from_millis(100))? {
                let _ = event::read()?; // 이벤트 소모
            }
            continue;
        }

        if event::poll(std::time::Duration::from_millis(250))? {
            let event = event::read()?;

            if let Event::Mouse(mouse_event) = event {
                match mouse_event.kind {
                    event::MouseEventKind::ScrollUp => app.scroll_up(),
                    event::MouseEventKind::ScrollDown => app.scroll_down(),
                    _ => {}
                }
            }

            if let Event::Key(key) = event
                && key.kind == KeyEventKind::Press
            {
                handle_key_input(app, key);
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn check_timers(app: &mut App) {
    if let Some(end_time) = app.pomodoro_end
        && Local::now() >= end_time
    {
        app.pomodoro_end = None; // 타이머 종료
        app.pomodoro_alert_expiry = Some(Local::now() + Duration::seconds(5));
    }

    if let Some(expiry) = app.pomodoro_alert_expiry
        && Local::now() >= expiry
    {
        app.pomodoro_alert_expiry = None; // 알림 종료
    }
    
    // Notification expiry check
    if let Some((_, expiry)) = app.notification {
        if Local::now() >= expiry {
            app.notification = None;
        }
    }
}

fn handle_key_input(app: &mut App, key: event::KeyEvent) {
    if handle_popup_events(app, key) {
        return;
    }

    match app.input_mode {
        InputMode::Navigate => handle_normal_mode(app, key),
        InputMode::Editing => handle_editing_mode(app, key),
        InputMode::Search => handle_search_mode(app, key),
    }
}

fn handle_popup_events(app: &mut App, key: event::KeyEvent) -> bool {
    if app.show_mood_popup {
        handle_mood_popup(app, key);
        return true;
    }
    if app.show_todo_popup {
        handle_todo_popup(app, key);
        return true;
    }
    if app.show_tag_popup {
        handle_tag_popup(app, key);
        return true;
    }
    if app.show_activity_popup {
        // 아무 키나 누르면 닫기
        app.show_activity_popup = false;
        return true;
    }
    if app.show_pomodoro_popup {
        handle_pomodoro_popup(app, key);
        return true;
    }
    if app.show_path_popup {
        handle_path_popup(app, key);
        return true;
    }
    false
}

fn handle_mood_popup(app: &mut App, key: event::KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.up) {
        let i = match app.mood_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    Mood::all().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        app.mood_list_state.select(Some(i));
    } else if key_match(&key, &app.config.keybindings.popup.down) {
        let i = match app.mood_list_state.selected() {
            Some(i) => {
                if i >= Mood::all().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        app.mood_list_state.select(Some(i));
    } else if key_match(&key, &app.config.keybindings.popup.confirm) {
        if let Some(i) = app.mood_list_state.selected() {
            let mood = Mood::all()[i];
            let _ = storage::append_entry(
                &app.config.data.log_path,
                &format!("Mood: {}", mood.as_str()),
            );
            app.update_logs();
        }
        check_carryover(app);
        app.show_mood_popup = false;
    } else if key_match(&key, &app.config.keybindings.popup.cancel) {
        app.show_mood_popup = false;
        app.transition_to(InputMode::Editing);
    }
}

fn check_carryover(app: &mut App) {
    let already_checked = storage::is_carryover_done(&app.config.data.log_path).unwrap_or(false);
    if !already_checked {
        if let Ok(todos) = storage::get_last_file_pending_todos(&app.config.data.log_path) {
            if !todos.is_empty() {
                app.pending_todos = todos;
                app.show_todo_popup = true;
            } else {
                app.transition_to(InputMode::Editing);
                let _ = storage::mark_carryover_done(&app.config.data.log_path);
            }
        } else {
            app.transition_to(InputMode::Editing);
            let _ = storage::mark_carryover_done(&app.config.data.log_path);
        }
    } else {
        app.transition_to(InputMode::Editing);
    }
}

fn handle_todo_popup(app: &mut App, key: event::KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.confirm) {
        for todo in &app.pending_todos {
            let formatted = ui::parser::format_todo(todo, false);
            let _ = storage::append_entry(&app.config.data.log_path, &formatted);
        }
        app.update_logs();
        app.show_todo_popup = false;
        app.transition_to(InputMode::Editing);
        let _ = storage::mark_carryover_done(&app.config.data.log_path);
    } else if key_match(&key, &app.config.keybindings.popup.cancel) {
        app.show_todo_popup = false;
        app.transition_to(InputMode::Editing);
        let _ = storage::mark_carryover_done(&app.config.data.log_path);
    }
}

fn handle_tag_popup(app: &mut App, key: event::KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.up) {
        let i = match app.tag_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        app.tag_list_state.select(Some(i));
    } else if key_match(&key, &app.config.keybindings.popup.down) {
        let i = match app.tag_list_state.selected() {
            Some(i) => {
                if i >= app.tags.len() - 1 {
                    app.tags.len() - 1
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        app.tag_list_state.select(Some(i));
    } else if key_match(&key, &app.config.keybindings.popup.confirm) {
        if let Some(i) = app.tag_list_state.selected()
            && i < app.tags.len()
        {
            let query = app.tags[i].0.clone();
            if let Ok(results) = storage::search_entries(&app.config.data.log_path, &query) {
                app.logs = results;
                app.is_search_result = true;
                app.logs_state.select(Some(0));
            }
        }
        app.show_tag_popup = false;
        app.transition_to(InputMode::Navigate);
    } else if key_match(&key, &app.config.keybindings.popup.cancel) {
        app.show_tag_popup = false;
        app.transition_to(InputMode::Navigate);
    }
}

fn handle_pomodoro_popup(app: &mut App, key: event::KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.confirm) {
        let mins: i64 = app.pomodoro_input.parse().unwrap_or(25);
        if mins > 0 {
            app.pomodoro_end = Some(Local::now() + Duration::minutes(mins));
        }
        app.show_pomodoro_popup = false;
        app.pomodoro_input.clear();
    } else if key_match(&key, &app.config.keybindings.popup.cancel) {
        app.show_pomodoro_popup = false;
        app.pomodoro_input.clear();
    } else {
        match key.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                app.pomodoro_input.push(c);
            }
            KeyCode::Backspace => {
                app.pomodoro_input.pop();
            }
            _ => {}
        }
    }
}

fn handle_normal_mode(app: &mut App, key: event::KeyEvent) {
    if key_match(&key, &app.config.keybindings.navigate.tags) {
        if let Ok(tags) = storage::get_all_tags(&app.config.data.log_path) {
            app.tags = tags;
            if !app.tags.is_empty() {
                app.tag_list_state.select(Some(0));
                app.show_tag_popup = true;
            }
        }
    } else if key_match(&key, &app.config.keybindings.navigate.quit) {
        app.quit();
    } else if key_match(&key, &app.config.keybindings.navigate.insert) {
        app.transition_to(InputMode::Editing);
    } else if key_match(&key, &app.config.keybindings.navigate.search) {
        app.transition_to(InputMode::Search);
    } else if key.code == KeyCode::Up {
        app.scroll_up();
    } else if key.code == KeyCode::Down {
        app.scroll_down();
    } else if key.code == KeyCode::Esc {
        if app.is_search_result {
            app.update_logs();
        }
    } else if key_match(&key, &app.config.keybindings.navigate.toggle_todo) {
        if let Some(i) = app.logs_state.selected()
            && i < app.logs.len()
        {
            let entry = &app.logs[i];
            // Just call toggle logic; let it decide if it's a todo
            let _ = storage::toggle_todo_status(entry);
            app.update_logs();
            app.logs_state.select(Some(i));
        }
    } else if key_match(&key, &app.config.keybindings.navigate.pomodoro) {
        if app.pomodoro_end.is_some() {
            app.pomodoro_end = None; // 끄기
        } else {
            app.show_pomodoro_popup = true;
            app.pomodoro_input = "25".to_string();
        }
    } else if key_match(&key, &app.config.keybindings.navigate.graph) {
        if let Ok(data) = storage::get_activity_stats(&app.config.data.log_path) {
            app.activity_data = data;
            app.show_activity_popup = true;
        }
    } else if key_match(&key, &app.config.keybindings.navigate.path) {
        // Initialize selection
        app.path_list_state.select(Some(0));
        app.show_path_popup = true;
    } else if key_match(&key, &app.config.keybindings.navigate.next_todo) {
        app.jump_next_todo();
    } else if key_match(&key, &app.config.keybindings.navigate.prev_todo) {
        app.jump_prev_todo();
    } else if key_match(&key, &app.config.keybindings.navigate.copy) {
        app.copy_current_log();
    }
}

fn handle_search_mode(app: &mut App, key: event::KeyEvent) {
    if key_match(&key, &app.config.keybindings.search.cancel) {
        app.input_mode = InputMode::Navigate;
        app.textarea
            .set_placeholder_text("키를 눌러 각종 기능을 사용하세요...");
    } else if key_match(&key, &app.config.keybindings.search.submit) {
        let query = app
            .textarea
            .lines()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join(" ");
        if !query.trim().is_empty()
            && let Ok(results) = storage::search_entries(&app.config.data.log_path, &query)
        {
            app.logs = results;
            app.is_search_result = true;
            app.logs_state.select(Some(0));
        }
        app.transition_to(InputMode::Navigate);
    } else {
        app.textarea.input(key);
    }
}

fn handle_editing_mode(app: &mut App, key: event::KeyEvent) {
    if key_match(&key, &app.config.keybindings.editing.cancel) {
        app.transition_to(InputMode::Navigate);
    } else if key_match(&key, &app.config.keybindings.editing.newline) {
        app.textarea.insert_newline();
    } else if key_match(&key, &app.config.keybindings.editing.save)
        || (key.code == event::KeyCode::Enter
            && !key.modifiers.contains(event::KeyModifiers::SHIFT))
    {
        // 텍스트 영역의 모든 줄을 가져와서 저장
        let lines = app.textarea.lines().to_vec();
        let input = lines
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join("\n           ");

        if !input.trim().is_empty() {
            if let Err(e) = storage::append_entry(&app.config.data.log_path, &input) {
                eprintln!("Error saving: {}", e);
            }
            app.update_logs();
        }

        // 텍스트 영역 초기화 (커서 위치 상관없이 전체 초기화)
        app.textarea = tui_textarea::TextArea::default();
        app.transition_to(InputMode::Editing);
    } else {
        app.textarea.input(key);
    }
}

fn handle_path_popup(app: &mut App, key: event::KeyEvent) {
    if key_match(&key, &app.config.keybindings.popup.confirm) {
        let index = app.path_list_state.selected().unwrap_or(0);

        let path_to_open = if index == 0 {
            // 1. Log Path
            if let Ok(abs_path) = std::fs::canonicalize(&app.config.data.log_path) {
                abs_path
            } else {
                std::path::PathBuf::from(&app.config.data.log_path)
            }
        } else {
            // 2. Config Path (CWD)
            std::env::current_dir().unwrap_or_default()
        };

        if let Err(e) = open::that(path_to_open) {
            eprintln!("Failed to open folder: {}", e);
        }

        app.show_path_popup = false;
        app.transition_to(InputMode::Navigate);
    } else if key_match(&key, &app.config.keybindings.popup.cancel) {
        app.show_path_popup = false;
        app.transition_to(InputMode::Navigate);
    } else if key_match(&key, &app.config.keybindings.popup.up) {
        let i = match app.path_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        app.path_list_state.select(Some(i));
    } else if key_match(&key, &app.config.keybindings.popup.down) {
        let i = match app.path_list_state.selected() {
            Some(i) => {
                if i >= 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        app.path_list_state.select(Some(i));
    }
}
