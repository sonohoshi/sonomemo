use std::{error::Error, io};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod ui;
mod storage;

use chrono::{Local, Duration};
use app::{App, InputMode, Mood};

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
        // 뽀모도로 타이머 체크
        if let Some(end_time) = app.pomodoro_end {
            if Local::now() >= end_time {
                app.pomodoro_end = None; // 타이머 종료
                // 5초간 알림 & 입력 차단
                app.pomodoro_alert_expiry = Some(Local::now() + Duration::seconds(5));
            }
        }
        
        // 알림 만료 체크
        if let Some(expiry) = app.pomodoro_alert_expiry {
            if Local::now() >= expiry {
                app.pomodoro_alert_expiry = None; // 알림 종료
            }
        }

        terminal.draw(|f| ui::ui(f, app))?;
        
        // 알림 표시 중일 때는 입력을 아예 받지 않음 (강제 휴식/주목)
        if app.pomodoro_alert_expiry.is_some() {
            // 이벤트 폴링만 하고 처리는 건너뜀 (화면 갱신은 위에서 함)
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
                // 팝업 모드일 때 키 처리
                if app.show_mood_popup {
                    match key.code {
                        KeyCode::Up => {
                            let i = match app.mood_list_state.selected() {
                                Some(i) => if i == 0 { Mood::all().len() - 1 } else { i - 1 },
                                None => 0,
                            };
                            app.mood_list_state.select(Some(i));
                        },
                        KeyCode::Down => {
                            let i = match app.mood_list_state.selected() {
                                Some(i) => if i >= Mood::all().len() - 1 { 0 } else { i + 1 },
                                None => 0,
                            };
                            app.mood_list_state.select(Some(i));
                        },
                        KeyCode::Enter => {
                            // 기분 선택 완료
                            if let Some(i) = app.mood_list_state.selected() {
                                let mood = Mood::all()[i];
                                let _ = storage::append_entry(&format!("Mood: {}", mood.to_str()));
                                app.update_logs();
                            }
                            // 기분 체크 후 할 일 요약 확인 및 이월
                            // (기분 팝업에서 넘어왔을 때도 체크 여부 확인)
                            let already_checked = storage::is_carryover_done().unwrap_or(false);
                            if !already_checked {
                                if let Ok(todos) = storage::get_last_file_pending_todos() {
                                    if !todos.is_empty() {
                                        app.pending_todos = todos;
                                        app.show_todo_popup = true;
                                    } else {
                                        app.input_mode = InputMode::Editing; 
                                        // 할 일이 없어도 체크한 것으로 간주
                                        let _ = storage::mark_carryover_done();
                                    }
                                } else {
                                    app.input_mode = InputMode::Editing;
                                    let _ = storage::mark_carryover_done();
                                }
                            } else {
                                app.input_mode = InputMode::Editing;
                            }
                            app.show_mood_popup = false;
                        },
                        KeyCode::Esc => {
                            // 선택 없이 닫기
                            app.show_mood_popup = false;
                            app.input_mode = InputMode::Editing;
                        },
                        _ => {}
                    }
                    continue; // 팝업 처리했으면 아래 로직 건너뜀
                }

                if app.show_todo_popup {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Char('ㅛ') => {
                            // 이월(Carry over) 수행
                            for todo in &app.pending_todos {
                                // "[ ] 내용" 형태일 수 있으니 그대로 추가
                                // 날짜는 오늘 날짜 파일에 들어가므로 자동 적용
                                let _ = storage::append_entry(todo); 
                            }
                            app.update_logs();
                            app.show_todo_popup = false;
                            app.input_mode = InputMode::Editing;
                            let _ = storage::mark_carryover_done();
                        },
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('ㅜ') | KeyCode::Esc => {
                            // 아니오 (그냥 닫기)
                            app.show_todo_popup = false;
                            app.input_mode = InputMode::Editing;
                            let _ = storage::mark_carryover_done();
                        },
                        _ => {}
                    }
                    continue;
                }



                if app.show_tag_popup {
                    match key.code {
                        KeyCode::Up => {
                            let i = match app.tag_list_state.selected() {
                                Some(i) => if i == 0 { 0 } else { i - 1 },
                                None => 0,
                            };
                            app.tag_list_state.select(Some(i));
                        },
                        KeyCode::Down => {
                            let i = match app.tag_list_state.selected() {
                                Some(i) => if i >= app.tags.len() - 1 { app.tags.len() - 1 } else { i + 1 },
                                None => 0,
                            };
                            app.tag_list_state.select(Some(i));
                        },
                        KeyCode::Enter => {
                             if let Some(i) = app.tag_list_state.selected() {
                                 if i < app.tags.len() {
                                     let query = app.tags[i].0.clone();
                                     // 태그로 검색 실행
                                     if let Ok(results) = storage::search_entries(&query) {
                                         app.logs = results;
                                         app.is_search_result = true;
                                         app.logs_state.select(Some(0));
                                     }
                                 }
                             }
                             app.show_tag_popup = false;
                             app.input_mode = InputMode::Normal;
                        },
                        KeyCode::Esc => {
                            app.show_tag_popup = false;
                            app.input_mode = InputMode::Normal;
                        },
                        _ => {}
                    }
                    continue;
                }

                if app.show_activity_popup {
                    // 아무 키나 누르면 닫기
                    app.show_activity_popup = false;
                    continue;
                }

                if app.show_pomodoro_popup {
                    match key.code {
                        KeyCode::Char(c) if c.is_digit(10) => {
                            app.pomodoro_input.push(c);
                        },
                        KeyCode::Backspace => {
                            app.pomodoro_input.pop();
                        },
                        KeyCode::Enter => {
                            // 입력값 파싱 후 타이머 설정
                            let mins: i64 = app.pomodoro_input.parse().unwrap_or(25);
                            if mins > 0 {
                                app.pomodoro_end = Some(Local::now() + Duration::minutes(mins));
                            }
                            app.show_pomodoro_popup = false;
                            app.pomodoro_input.clear();
                        },
                        KeyCode::Esc => {
                            app.show_pomodoro_popup = false;
                            app.pomodoro_input.clear();
                        },
                        _ => {}
                    }
                    continue;
                }

                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('t') | KeyCode::Char('ㅅ') => {
                             if let Ok(tags) = storage::get_all_tags() {
                                 app.tags = tags;
                                 if !app.tags.is_empty() {
                                     app.tag_list_state.select(Some(0));
                                     app.show_tag_popup = true;
                                     // Popup 모드로 진입 (InputMode는 Normal 유지하되 플래그로 제어)
                                 }
                             }
                        },
                        KeyCode::Char('q') | KeyCode::Char('ㅂ') => app.quit(),
                        KeyCode::Char('i') | KeyCode::Char('ㅑ') => app.input_mode = InputMode::Editing,
                        KeyCode::Char('?') => {
                            app.input_mode = InputMode::Search;
                            app.textarea.set_placeholder_text("검색할 단어를 입력하세요...");
                        },
                        KeyCode::Up => app.scroll_up(),
                        KeyCode::Down => app.scroll_down(),
                        KeyCode::Esc => {
                            if app.is_search_result {
                                app.update_logs(); // 검색 해제
                            }
                        },
                        KeyCode::Enter => {
                            // 할 일 토글
                            if let Some(i) = app.logs_state.selected() {
                                if i < app.logs.len() {
                                    let entry = &app.logs[i];
                                    if entry.content.contains("- [ ]") || entry.content.contains("- [x]") {
                                        let _ = storage::toggle_todo_status(entry);
                                        // 현재 뷰 갱신 (검색 중이면 다시 검색, 아니면 오늘 로그 리로드)
                                        if app.is_search_result {
                                            // TODO: 검색 결과 리프레시 로직 필요 (복잡해서 일단은 오늘 로그로 리셋하거나 놔둠)
                                            // 간단히 텍스트만 바꿀 수도 있지만, 파일이 바뀌었으므로 리로딩이 안전
                                             app.update_logs();
                                        } else {
                                            app.update_logs();
                                        }
                                        // 커서 위치 유지를 위해
                                        app.logs_state.select(Some(i));
                                    }
                                }
                            }
                        },
                        KeyCode::Char('p') | KeyCode::Char('ㅔ') => {
                            // 뽀모도로 팝업 열기 (이미 켜져있으면 끄기 -> 질문: 끄는 키는? 그냥 p 다시 누르면 꺼지게 할까? 아니면 팝업에서 0 입력?)
                            // 심플하게: 타이머가 돌고 있으면 끄고, 없으면 설정 팝업
                            if app.pomodoro_end.is_some() {
                                app.pomodoro_end = None; // 끄기
                            } else {
                                app.show_pomodoro_popup = true;
                                app.pomodoro_input = "25".to_string(); // 기본값
                            }
                        },
                        KeyCode::Char('g') | KeyCode::Char('ㅎ') => {
                            // 잔디 심기 토글
                            if app.show_activity_popup {
                                app.show_activity_popup = false;
                            } else {
                                if let Ok(data) = storage::get_activity_stats() {
                                    app.activity_data = data;
                                    app.show_activity_popup = true;
                                }
                            }
                        },
                        _ => {}
                    },
                    InputMode::Search => {
                        match key.code {
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.textarea.set_placeholder_text("여기에 메모를 입력하세요... (Enter: 저장, Esc: 모드 전환, :q 종료)");
                            },
                            KeyCode::Enter => {
                                let query = app.textarea.lines().iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(" ");
                                if !query.trim().is_empty() {
                                    if let Ok(results) = storage::search_entries(&query) {
                                        app.logs = results;
                                        app.is_search_result = true;
                                        app.logs_state.select(Some(0));
                                    }
                                }
                                app.textarea.delete_line_by_head();
                                app.input_mode = InputMode::Normal;
                                app.textarea.set_placeholder_text("여기에 메모를 입력하세요... (Enter: 저장, Esc: 모드 전환, :q 종료)");
                            },
                            KeyCode::Char('p') => {
                                // 뽀모도로 토글
                                if app.pomodoro_end.is_some() {
                                    app.pomodoro_end = None; // 끄기
                                } else {
                                    // 25분 뒤 시간 설정
                                    app.pomodoro_end = Some(Local::now() + Duration::minutes(25));
                                }
                            },
                            KeyCode::Char('g') => {
                                // 잔디 심기 토글
                                if app.show_activity_popup {
                                    app.show_activity_popup = false;
                                } else {
                                    if let Ok(data) = storage::get_activity_stats() {
                                        app.activity_data = data;
                                        app.show_activity_popup = true;
                                    }
                                }
                            },
                            _ => { app.textarea.input(key); }
                        }
                    },
                    InputMode::Editing => {
                        match key.code {
                            KeyCode::Esc => app.input_mode = InputMode::Normal,
                            KeyCode::Enter => {
                                // Shift + Enter => 줄바꿈
                                if key.modifiers.contains(KeyModifiers::SHIFT) {
                                    app.textarea.insert_newline();
                                } else {
                                    // 그냥 Enter => 저장 (기존 로직)
                                let input = app.textarea.lines().iter().map(|s| s.as_str()).collect::<Vec<&str>>().join("\n  ");
                                if !input.trim().is_empty() {
                                    if let Err(e) = storage::append_entry(&input) {
                                        // 에러 처리 (로그에 표시 등)
                                        // 여기서는 간단히 무시하거나 stderr 출력
                                        eprintln!("Error saving: {}", e);
                                    }
                                    // UI 갱신 (스크롤 포함)
                                    app.update_logs();
                                }
                                
                                // 멀티라인 입력 후 전체 내용을 지워야 함
                                // delete_line_by_head는 한 줄만 지우므로, 전체 라인을 지우는 로직으로 변경
                                while !app.textarea.is_empty() {
                                    app.textarea.delete_line_by_head(); // 현재 라인 삭제
                                    app.textarea.delete_char(); // 줄바꿈 문자 삭제 (또는 라인 병합 유도)
                                    // 안전하게는 그냥 textarea를 새로 만드는게 낫지만, 스타일 유지를 위해 반복 삭제
                                    
                                    // 더 간단하고 확실한 방법: 새 인스턴스로 교체하되 스타일 재설정 (ui.rs에서 매번 설정하므로 괜찮음)
                                    app.textarea = tui_textarea::TextArea::default();
                                    break;
                                }
                            }
                        },
                            _ => {
                                app.textarea.input(key);
                            }
                        }
                    }
                }
            }
        }

        }

        if app.should_quit {
            return Ok(());
        }
    }
}
