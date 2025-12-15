use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
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

use app::App;
use chrono::{Duration, Local};
use models::{InputMode, Mood};

fn main() -> Result<(), Box<dyn Error>> {
    // 터미널 초기화
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 앱 실행
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // 터미널 복구
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
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

            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Press {
                    handle_key_input(app, key);
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn check_timers(app: &mut App) {
    if let Some(end_time) = app.pomodoro_end {
        if Local::now() >= end_time {
            app.pomodoro_end = None; // 타이머 종료
            app.pomodoro_alert_expiry = Some(Local::now() + Duration::seconds(5));
        }
    }

    if let Some(expiry) = app.pomodoro_alert_expiry {
        if Local::now() >= expiry {
            app.pomodoro_alert_expiry = None; // 알림 종료
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
    false
}

fn handle_mood_popup(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Up => {
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
        }
        KeyCode::Down => {
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
        }
        KeyCode::Enter => {
            if let Some(i) = app.mood_list_state.selected() {
                let mood = Mood::all()[i];
                let _ = storage::append_entry(&format!("Mood: {}", mood.to_str()));
                app.update_logs();
            }
            check_carryover(app);
            app.show_mood_popup = false;
        }
        KeyCode::Esc => {
            app.show_mood_popup = false;
            app.transition_to(InputMode::Editing);
        }
        _ => {}
    }
}

fn check_carryover(app: &mut App) {
    let already_checked = storage::is_carryover_done().unwrap_or(false);
    if !already_checked {
        if let Ok(todos) = storage::get_last_file_pending_todos() {
            if !todos.is_empty() {
                app.pending_todos = todos;
                app.show_todo_popup = true;
            } else {
                app.transition_to(InputMode::Editing);
                let _ = storage::mark_carryover_done();
            }
        } else {
            app.transition_to(InputMode::Editing);
            let _ = storage::mark_carryover_done();
        }
    } else {
        app.transition_to(InputMode::Editing);
    }
}

fn handle_todo_popup(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Char('ㅛ') => {
            for todo in &app.pending_todos {
                let _ = storage::append_entry(todo);
            }
            app.update_logs();
            app.show_todo_popup = false;
            app.transition_to(InputMode::Editing);
            let _ = storage::mark_carryover_done();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('ㅜ') | KeyCode::Esc => {
            app.show_todo_popup = false;
            app.transition_to(InputMode::Editing);
            let _ = storage::mark_carryover_done();
        }
        _ => {}
    }
}

fn handle_tag_popup(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Up => {
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
        }
        KeyCode::Down => {
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
        }
        KeyCode::Enter => {
            if let Some(i) = app.tag_list_state.selected() {
                if i < app.tags.len() {
                    let query = app.tags[i].0.clone();
                    if let Ok(results) = storage::search_entries(&query) {
                        app.logs = results;
                        app.is_search_result = true;
                        app.logs_state.select(Some(0));
                    }
                }
            }
            app.show_tag_popup = false;
            app.transition_to(InputMode::Navigate);
        }
        KeyCode::Esc => {
            app.show_tag_popup = false;
            app.transition_to(InputMode::Navigate);
        }
        _ => {}
    }
}

fn handle_pomodoro_popup(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Char(c) if c.is_digit(10) => {
            app.pomodoro_input.push(c);
        }
        KeyCode::Backspace => {
            app.pomodoro_input.pop();
        }
        KeyCode::Enter => {
            let mins: i64 = app.pomodoro_input.parse().unwrap_or(25);
            if mins > 0 {
                app.pomodoro_end = Some(Local::now() + Duration::minutes(mins));
            }
            app.show_pomodoro_popup = false;
            app.pomodoro_input.clear();
        }
        KeyCode::Esc => {
            app.show_pomodoro_popup = false;
            app.pomodoro_input.clear();
        }
        _ => {}
    }
}

fn handle_normal_mode(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Char('t') | KeyCode::Char('ㅅ') => {
            if let Ok(tags) = storage::get_all_tags() {
                app.tags = tags;
                if !app.tags.is_empty() {
                    app.tag_list_state.select(Some(0));
                    app.show_tag_popup = true;
                }
            }
        }
        KeyCode::Char('q') | KeyCode::Char('ㅂ') => app.quit(),
        KeyCode::Char('i') | KeyCode::Char('ㅑ') => {
            app.transition_to(InputMode::Editing);
        }
        KeyCode::Char('?') => {
            app.transition_to(InputMode::Search);
        }
        KeyCode::Up => app.scroll_up(),
        KeyCode::Down => app.scroll_down(),
        KeyCode::Esc => {
            if app.is_search_result {
                app.update_logs();
            }
        }
        KeyCode::Enter => {
            if let Some(i) = app.logs_state.selected() {
                if i < app.logs.len() {
                    let entry = &app.logs[i];
                    if entry.content.contains("- [ ]") || entry.content.contains("- [x]") {
                        let _ = storage::toggle_todo_status(entry);
                        if app.is_search_result {
                            app.update_logs(); // TODO: Maintain search, but reloading is safer
                        } else {
                            app.update_logs();
                        }
                        app.logs_state.select(Some(i));
                    }
                }
            }
        }
        KeyCode::Char('p') | KeyCode::Char('ㅔ') => {
            if app.pomodoro_end.is_some() {
                app.pomodoro_end = None; // 끄기
            } else {
                app.show_pomodoro_popup = true;
                app.pomodoro_input = "25".to_string();
            }
        }
        KeyCode::Char('g') | KeyCode::Char('ㅎ') => {
            if let Ok(data) = storage::get_activity_stats() {
                app.activity_data = data;
                app.show_activity_popup = true;
            }
        }
        _ => {}
    }
}

fn handle_search_mode(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Navigate;
            app.textarea
                .set_placeholder_text("키를 눌러 각종 기능을 사용하세요...");
        }
        KeyCode::Enter => {
            let query = app
                .textarea
                .lines()
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .join(" ");
            if !query.trim().is_empty() {
                if let Ok(results) = storage::search_entries(&query) {
                    app.logs = results;
                    app.is_search_result = true;
                    app.logs_state.select(Some(0));
                }
            }
            app.transition_to(InputMode::Navigate);
        }
        _ => {
            app.textarea.input(key);
        }
    }
}

fn handle_editing_mode(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.transition_to(InputMode::Navigate);
        }
        KeyCode::Enter => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.textarea.insert_newline();
            } else {
                let input = app
                    .textarea
                    .lines()
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>()
                    .join("\n           ");
                if !input.trim().is_empty() {
                    if let Err(e) = storage::append_entry(&input) {
                        eprintln!("Error saving: {}", e);
                    }
                    app.update_logs();
                }

                // 텍스트 영역 초기화 (스타일 유지를 위해 재생성 대신 삭제 반복했었으나, 여기선 재생성 루틴이 깔끔함)
                // 하지만 ui::ui 함수에서 스타일을 매번 설정해주므로 재생성이 안전함.
                app.textarea = tui_textarea::TextArea::default();
                app.transition_to(InputMode::Editing);
            }
        }
        _ => {
            app.textarea.input(key);
        }
    }
}
