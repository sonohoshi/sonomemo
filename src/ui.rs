use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, List, ListItem, Clear},
    Frame,
};
use textwrap::wrap;
use chrono::Local;

use ratatui::style::Stylize;
use crate::app::{App, Mood, InputMode};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Min(1),
            Constraint::Length(3), // Input area
            Constraint::Length(1), // Footer (Help)
        ])
        .split(f.area());

    // 상단 영역을 좌우로 분할 (로그 70%, 할 일 목록 30%)
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(chunks[0]);

    // ... (기존 렌더링 코드 유지)

    // 상단 로그 뷰
    let list_area_width = top_chunks[0].width.saturating_sub(4) as usize; // 테두리 및 여유 공간

    let list_items: Vec<ListItem> = app.logs
        .iter()
        .map(|entry| {
            // 텍스트 줄바꿈 처리 (멀티라인 엔트리 대응)
            let mut lines = Vec::new();
            
            // 사용자가 입력한 엔터(\n)를 기준으로 먼저 나눔
            for (line_idx, raw_line) in entry.content.lines().enumerate() {
                 let wrapped_lines = wrap(raw_line, list_area_width);
                 
                 for (wrap_idx, wline) in wrapped_lines.iter().enumerate() {
                      // 첫 줄의 첫 조각만 타임스탬프 파싱 시도
                      // 그 외(사용자가 줄바꿈했거나, 너비 때문에 줄바꿈된 경우)는 일반 텍스트
                      if line_idx == 0 && wrap_idx == 0 {
                          lines.push(parse_log_line(&wline));
                      } else {
                          // 들여쓰기 처리
                          // raw_line 자체가 이미 "  "로 시작할 수 있음 (storage.rs에서 저장 시 처리)
                          // 하지만 너비 초과로 인한 wrap된 줄은 추가 들여쓰기가 필요할 수 있음
                          
                          let display_text = if wrap_idx > 0 {
                               format!("    {}", wline) // wrap된 줄은 더 깊게 들여쓰기
                          } else {
                               format!("{}", wline) // 사용자가 줄바꿈한 줄은 그대로 (이미 공백 포함됨)
                          };
                          
                          lines.push(parse_log_line(&display_text));
                      }
                 }
            }
            ListItem::new(Text::from(lines))
        })
        .collect();

    let title = if app.is_search_result {
        format!(" 🔍 Search Results: {} found (Esc to reset) ", app.logs.len())
    } else {
        let time = Local::now().format("%Y-%m-%d %H:%M");
        let pomodoro = if let Some(end_time) = app.pomodoro_end {
            let now = Local::now();
            if now < end_time {
                let remaining = end_time - now;
                format!(" [🍅 {:02}:{:02}]", remaining.num_minutes(), remaining.num_seconds() % 60)
            } else {
                "".to_string() 
            }
        } else {
            "".to_string()
        };
        
        format!(" 📝 SONOMEMO - {}{}", time, pomodoro)
    };

    // 모드에 따른 메인 테두리 색상 결정
    let main_border_color = match app.input_mode {
            InputMode::Normal => Color::Reset,
            InputMode::Editing => Color::Green,
            InputMode::Search => Color::Cyan,
    };

    let logs_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(main_border_color))
        .title(title);
    
    let logs_list = List::new(list_items)
        .block(logs_block)
        .highlight_symbol("▶ ") // 조금 더 멋진 화살표
        .highlight_style(Style::default().bg(Color::Rgb(50, 50, 50)).add_modifier(Modifier::BOLD)); // 배경색 하이라이트 
        
    f.render_stateful_widget(logs_list, top_chunks[0], &mut app.logs_state);

    // 오른쪽 할 일 목록 뷰 (오늘의 할 일만 필터링)
    let todo_area_width = top_chunks[1].width.saturating_sub(2) as usize; // 테두리 제외

    let todos: Vec<ListItem> = app.logs
        .iter()
        .filter(|entry| entry.content.contains("- [ ]"))
        .map(|entry| {
             // 타임스탬프 등 제거하고 깔끔하게 보여주기
             let content = &entry.content;
             let display_text = if let Some(idx) = content.find("- [ ]") {
                 &content[idx..] // "- [ ] 내용" 부터 표시
             } else {
                 content
             };

             // 줄바꿈 처리
             let wrapped = wrap(display_text, todo_area_width);
             let mut lines = Vec::new();
             for (i, line) in wrapped.iter().enumerate() {
                 if i == 0 {
                     lines.push(Line::from(line.to_string()));
                 } else {
                     // 체크박스(- [ ] ) 길이만큼 들여쓰기
                     lines.push(Line::from(format!("      {}", line)));
                 }
             }
             ListItem::new(Text::from(lines))
        })
        .collect();

    let todo_block = Block::default()
        .borders(Borders::ALL)
        .title(" Today's Tasks ")
        .border_style(Style::default().fg(if todos.is_empty() { Color::Green } else { Color::Yellow }));
    
    let todo_list = List::new(todos).block(todo_block);
    f.render_widget(todo_list, top_chunks[1]);

    // 하단 입력창
    let (input_title, border_color) = match app.input_mode {
        crate::app::InputMode::Search => (" Search Query (? to Search) ", Color::Cyan),
        crate::app::InputMode::Editing => (" Input (Press Esc to Normal) ", Color::Green),
        crate::app::InputMode::Normal => (" Input (Normal Mode - Press 'i' to Edit) ", Color::Reset),
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(input_title)
        .border_style(Style::default().fg(border_color));
    
    app.textarea.set_block(input_block);

    // Editing/Search 모드일 때만 커서 스타일 적용 (나머지는 기본 숨김 처리되거나 루프에서 처리)
    match app.input_mode {
        crate::app::InputMode::Normal => {
            app.textarea.set_cursor_style(Style::default()); // 커서 숨김 효과 (또는 메인루프에서 show_cursor 제어)
        },
        _ => {
            app.textarea.set_cursor_line_style(Style::default().underline_color(Color::Reset));
            app.textarea.set_cursor_style(Style::default().reversed());
        }
    }
    
    f.render_widget(&app.textarea, chunks[1]);
    
    // 커서 위치 수동 설정 (한글 IME 지원을 위해 필수)
    // IME 입력 창은 시스템 커서 위치를 따라가므로, 터미널 커서를 텍스트 입력 위치에 둬야 함.
    if app.input_mode == crate::app::InputMode::Editing || app.input_mode == crate::app::InputMode::Search {
        let (row, col) = app.textarea.cursor();
        // 현재 라인의 내용을 가져와서 커서 위치(col)까지의 '시각적 너비'를 계산
        if let Some(line) = app.textarea.lines().get(row) {
            let visual_col: usize = line.chars().take(col).map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)).sum();
            
            f.set_cursor(
                chunks[1].x + visual_col as u16 + 1,
                chunks[1].y + row as u16 + 1,
            );
        }
    }

    // 하단 도움말 푸터
    let help_text = match app.input_mode {
        InputMode::Normal => " [i] Edit  [t] Tag  [?] Search  [Enter] Toggle  [p] Pomodoro  [g] Graph  [q] Quit ",
        InputMode::Editing => " [Esc] Normal Mode  [Enter] Save Memo ",
        InputMode::Search => " [Esc] Reset Search  [Enter] Filter ",
    };
    let footer = Paragraph::new(Line::from(Span::styled(help_text, Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD))))
        .block(Block::default().borders(Borders::NONE)); // 테두리 없이 깔끔하게
    f.render_widget(footer, chunks[2]);

    // 잔디 심기 (활동 그래프) 팝업
    if app.show_activity_popup {
        render_activity_popup(f, app);
    }
    
    // 뽀모도로 입력 팝업
    if app.show_pomodoro_popup {
        let block = Block::default().title(" 🍅 Set Timer (Minutes) ").borders(Borders::ALL);
        let area = centered_rect(40, 20, f.area());
        f.render_widget(Clear, area); // 배경 지우기
        f.render_widget(block, area);

        let input_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1)])
            .margin(2)
            .split(area)[0];
            
        let text = Paragraph::new(format!("{} _", app.pomodoro_input)) // 커서 깜빡임 흉내
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(text, input_area);
    }
    
    // 기분 팝업이 켜져있다면 렌더링
    if app.show_mood_popup {
        let block = Block::default().title(" 기분이가 좀 어떠세여? ").borders(Borders::ALL);
        let area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, area); // 배경 지우기
        f.render_widget(block, area);

        let moods = Mood::all();
        let items: Vec<ListItem> = moods
            .iter()
            .map(|m| ListItem::new(m.to_str()))
            .collect();
        
        // 팝업 내부 레이아웃
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .margin(1)
            .split(area);
            
        let list = List::new(items)
            .highlight_symbol(">> ")
            .highlight_style(Style::default().fg(Color::Yellow));
            
        f.render_stateful_widget(list, popup_layout[0], &mut app.mood_list_state);
    }
    
    // 할 일 요약 팝업 렌더링 (기분 팝업보다 위에 표시)
    if app.show_todo_popup {
        let title = format!(" 지난 할 일이 {}개 남았습니다. 오늘로 가져올까요? (Y/n) ", app.pending_todos.len());
        let block = Block::default().title(title).borders(Borders::ALL).style(Style::default().fg(Color::LightRed));
        let area = centered_rect(70, 40, f.area());
        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let items: Vec<ListItem> = app.pending_todos
            .iter()
            .map(|t| ListItem::new(format!("• {}", t)))
            .collect();
        
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .margin(1)
            .split(area);
            
        let list = List::new(items)
            .highlight_symbol(">> "); // 선택 기능은 딱히 필요 없지만 리스트로 보여줌
            
        f.render_stateful_widget(list, popup_layout[0], &mut app.todo_list_state);

    }

    // 태그 목록 팝업
    if app.show_tag_popup {
        let block = Block::default().title(" 태그를 선택하세요 (Enter: 검색, Esc: 닫기) ").borders(Borders::ALL);
        let area = centered_rect(50, 60, f.area());
        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let items: Vec<ListItem> = app.tags
            .iter()
            .map(|(tag, count)| {
                ListItem::new(format!("{} ({})", tag, count))
            })
            .collect();
            
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .margin(1)
            .split(area);
            
        let list = List::new(items)
            .highlight_symbol(">> ")
            .highlight_style(Style::default().fg(Color::Cyan));
            
        f.render_stateful_widget(list, popup_layout[0], &mut app.tag_list_state);
    }

    // 뽀모도로 강제 알림 (가장 최상위)
    if app.pomodoro_alert_expiry.is_some() {
        render_siren_popup(f);
    }
}

fn render_siren_popup(f: &mut Frame) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red).bg(Color::Black).add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK));
    
    let area = centered_rect(80, 60, f.area());
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let siren_art = vec![
        "         _______  TIME'S UP!  _______",
        "        /       \\            /       \\",
        "       |  (o)  |   🚨🚨🚨   |  (o)  |",
        "        \\_______/            \\_______/",
        "",
        "      Take a break! Stretch! Drink water!",
        "      (Input blocked for 5 seconds)",
    ];

    let text_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .margin(2)
        .split(area)[0];

    let mut art_spans = Vec::new();
    for line in siren_art {
        art_spans.push(ListItem::new(Line::from(Span::styled(line, Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)))));
    }
    
    // 중앙 정렬을 위해 Paragraph 대신 List를 썼으나, Paragraph가 나을 수 있음.
    // 여기선 심플하게 List로 처리
    f.render_widget(List::new(art_spans), text_area);
}

// 팝업 위치 계산 헬퍼
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn parse_log_line(text: &str) -> Line<'static> {
    let mut spans = Vec::new();
    
    // 타임스탬프 처리 [HH:MM:SS]
    let parts: Vec<&str> = text.splitn(2, "] ").collect();
    if parts.len() == 2 && parts[0].starts_with('[') {
        spans.push(Span::styled(format!("{}] ", parts[0]), Style::default().fg(Color::Blue)));
        
        let content = parts[1];
        
        // TODO 체크박스 처리
        let (content, todo_prefix) = if content.starts_with("- [ ] ") {
            spans.push(Span::styled("⬜ ", Style::default().fg(Color::Red))); // 미완료 이모지
            (&content[6..], true)
        } else if content.starts_with("- [x] ") {
            spans.push(Span::styled("✅ ", Style::default().fg(Color::Green))); // 완료 이모지
            (&content[6..], true)
        } else {
            (content, false)
        };

        // 태그 파싱 (#단어)
        for (i, word) in content.split_whitespace().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" ".to_string()));
            }
            if word.starts_with('#') {
                spans.push(Span::styled(word.to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            } else if word.starts_with("Mood:") {
                 // Mood: Happy -> 😐 Happy 변환 로직은 복잡하니 일단 텍스트 컬러링만 강화
                 spans.push(Span::styled("🎭 Mood:", Style::default().fg(Color::Magenta).add_modifier(Modifier::ITALIC)));
                 // "Mood:" 뒤의 단어는 루프 다음 순회에서 처리됨
            } else {
                if todo_prefix {
                     // 할 일 내용은 약간 밝게
                     spans.push(Span::styled(word.to_string(), Style::default().fg(Color::Reset)));
                } else {
                     spans.push(Span::raw(word.to_string()));
                }
            }
        }
    } else {
        // 형식이 없는 일반 텍스트
        spans.push(Span::raw(text.to_string()));
    }

    Line::from(spans)
}

fn render_activity_popup(f: &mut Frame, app: &App) {
    let block = Block::default().title(" 🌱 Activity Graph (Last 2 Weeks) ").borders(Borders::ALL);
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    // 간단하게 최근 14일(2주)치만 리스트로 보여주는 형태로 구현 (복잡한 그리드는 TUI 제약상 일단 생략)
    let today = Local::now().date_naive();
    let mut items = Vec::new();

    for i in 0..14 {
        let date = today - chrono::Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();
        let count = app.activity_data.get(&date_str).cloned().unwrap_or(0);
        
        let bar_len = count.min(20); // 최대 20칸
        let bar: String = "■".repeat(bar_len);
        
        // 색상: 0=회색, 1~4=연두, 5+=진초록
        let color = if count == 0 { Color::DarkGray }
                    else if count < 5 { Color::Green } 
                    else { Color::LightGreen };

        items.push(ListItem::new(Line::from(vec![
            Span::raw(format!("{} : {:3} logs ", date_str, count)),
            Span::styled(bar, Style::default().fg(color))
        ])));
    }

    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .margin(2)
        .split(area)[0];

    f.render_widget(List::new(items), inner_area);
}
