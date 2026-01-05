use chrono::Local;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use textwrap::wrap;

use crate::app::App;
use crate::models::InputMode;
use crate::ui::color_parser::parse_color;
use ratatui::style::Stylize;

pub mod color_parser;
pub mod components;
pub mod parser;
pub mod popups;

use components::parse_log_line;
use popups::{
    render_activity_popup, render_mood_popup, render_path_popup, render_pomodoro_popup,
    render_siren_popup, render_tag_popup, render_todo_popup,
};

/// 애플리케이션의 전체 UI를 렌더링하는 메인 함수입니다.
///
/// `f`: Ratatui 프레임 객체
/// `app`: 애플리케이션 상태 객체
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
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(chunks[0]);

    // 상단 로그 뷰
    let list_area_width = top_chunks[0].width.saturating_sub(4) as usize; // 테두리 및 여유 공간

    let list_items: Vec<ListItem> = app
        .logs
        .iter()
        .map(|entry| {
            // 텍스트 줄바꿈 처리 (멀티라인 엔트리 대응)
            let mut lines = Vec::new();

            // 사용자가 입력한 엔터(\n)를 기준으로 먼저 나눔
            for (line_idx, raw_line) in entry.content.lines().enumerate() {
                let wrapped_lines = wrap(raw_line, list_area_width);

                for (wrap_idx, wline) in wrapped_lines.iter().enumerate() {
                    // 첫 줄의 첫 조각만 타임스탬프 파싱 시도
                    if line_idx == 0 && wrap_idx == 0 {
                        lines.push(parse_log_line(wline, &app.config.theme));
                    } else {
                        let display_text = if wrap_idx > 0 {
                            format!("    {}", wline) // wrap된 줄은 더 깊게 들여쓰기
                        } else {
                            format!("{}", wline) // 사용자가 줄바꿈한 줄은 그대로
                        };

                        lines.push(parse_log_line(&display_text, &app.config.theme));
                    }
                }
            }
            ListItem::new(Text::from(lines))
        })
        .collect();

    let title = if app.is_search_result {
        format!(
            " 🔍 Search Results: {} found (Esc to reset) ",
            app.logs.len()
        )
    } else {
        let time = Local::now().format("%Y-%m-%d %H:%M");
        let pomodoro = if let Some(end_time) = app.pomodoro_end {
            let now = Local::now();
            if now < end_time {
                let remaining = end_time - now;
                format!(
                    " [🍅 {:02}:{:02}]",
                    remaining.num_minutes(),
                    remaining.num_seconds() % 60
                )
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
        InputMode::Navigate => parse_color(&app.config.theme.border_default),
        InputMode::Editing => parse_color(&app.config.theme.border_editing),
        InputMode::Search => parse_color(&app.config.theme.border_search),
    };

    let logs_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(main_border_color))
        .title(title);

    let highlight_bg = parse_color(&app.config.theme.text_highlight);
    let logs_list = List::new(list_items)
        .block(logs_block)
        .highlight_symbol("▶ ") // 조금 더 멋진 화살표
        .highlight_style(
            Style::default()
                .bg(highlight_bg)
                .add_modifier(Modifier::BOLD),
        ); // 배경색 하이라이트 

    f.render_stateful_widget(logs_list, top_chunks[0], &mut app.logs_state);

    // 오른쪽 할 일 목록 뷰 (오늘의 할 일만 필터링)
    let todo_area_width = top_chunks[1].width.saturating_sub(2) as usize; // 테두리 제외

    let todos: Vec<ListItem> = app
        .logs
        .iter()
        .filter_map(|entry| {
            // Use shared parser logic to check if it's a pending todo
            if let Some(content) = parser::extract_pending_content(&entry.content) {
                // Reconstruct a displayable todo line (e.g., "- [ ] content")
                // Or just show the content? The original code showed "- [ ] content".
                // Let's standardise it to "- [ ] content" for the sidebar.
                let display_text = format!("- [ ] {}", content);

                // 줄바꿈 처리
                let wrapped = wrap(&display_text, todo_area_width);
                let mut lines = Vec::new();
                for (i, line) in wrapped.iter().enumerate() {
                    if i == 0 {
                        lines.push(Line::from(line.to_string()));
                    } else {
                        // 체크박스(- [ ] ) 길이만큼 들여쓰기
                        lines.push(Line::from(format!("      {}", line)));
                    }
                }
                Some(ListItem::new(Text::from(lines)))
            } else {
                None
            }
        })
        .collect();

    let todo_border_color = parse_color(&app.config.theme.border_todo_header);
    // 할 일이 없으면 Green(성공?), 있으면 Yellow(진행중?) -> 기본값 유지하되 테마 적용?
    // 기존 로직: if todos.is_empty() { Color::Green } else { Color::Yellow }
    // 여기서는 Configurable하게 만들기 애매하니 일단 todo_border_color를 기본으로 하고 empty일 때만 예외 처리?
    // 혹은 Config에 todo_header_empty / todo_header_active 추가?
    // 일단 간단히 todo_border_color만 사용.

    let todo_block = Block::default()
        .borders(Borders::ALL)
        .title(" Today's Tasks ")
        .border_style(Style::default().fg(todo_border_color));

    let todo_list = List::new(todos).block(todo_block);
    f.render_widget(todo_list, top_chunks[1]);

    // 하단 입력창
    let (input_title, border_color) = match app.input_mode {
        crate::models::InputMode::Search => {
            (" Search ", parse_color(&app.config.theme.border_search))
        }
        crate::models::InputMode::Editing => {
            (" Input ", parse_color(&app.config.theme.border_editing))
        }
        crate::models::InputMode::Navigate => {
            (" Navigate ", parse_color(&app.config.theme.border_default))
        }
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(input_title)
        .border_style(Style::default().fg(border_color));

    app.textarea.set_block(input_block);

    // Editing/Search 모드일 때만 커서 스타일 적용
    match app.input_mode {
        crate::models::InputMode::Navigate => {
            app.textarea.set_cursor_style(Style::default());
        }
        _ => {
            app.textarea
                .set_cursor_line_style(Style::default().underline_color(Color::Reset));
            app.textarea.set_cursor_style(Style::default().reversed());
        }
    }

    f.render_widget(&app.textarea, chunks[1]);

    // 커서 위치 수동 설정 (한글 IME 지원을 위해 필수)
    if app.input_mode == crate::models::InputMode::Editing
        || app.input_mode == crate::models::InputMode::Search
    {
        let (row, col) = app.textarea.cursor();
        if let Some(line) = app.textarea.lines().get(row) {
            let visual_col: usize = line
                .chars()
                .take(col)
                .map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0))
                .sum();

            f.set_cursor_position((
                chunks[1].x + visual_col as u16 + 1,
                chunks[1].y + row as u16 + 1,
            ));
        }
    }

    // 하단 도움말 푸터
    let help_text = match app.input_mode {
        InputMode::Navigate => &app.config.help.navigate,
        InputMode::Editing => &app.config.help.editing,
        InputMode::Search => &app.config.help.search,
    };
    let footer = Paragraph::new(Line::from(Span::styled(
        help_text,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )))
    .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, chunks[2]);

    // 팝업 렌더링 (순서 중요: 나중에 렌더링된 것이 위에 뜸)
    if app.show_activity_popup {
        render_activity_popup(f, app);
    }

    if app.show_pomodoro_popup {
        render_pomodoro_popup(f, app);
    }

    if app.show_mood_popup {
        render_mood_popup(f, app);
    }

    if app.show_todo_popup {
        render_todo_popup(f, app);
    }

    if app.show_tag_popup {
        render_tag_popup(f, app);
    }

    if app.pomodoro_alert_expiry.is_some() {
        render_siren_popup(f);
    }

    if app.show_path_popup {
        render_path_popup(f, app);
    }

    // Render notification overlay
    if let Some((message, _)) = &app.notification {
        use ratatui::widgets::Clear;

        let area = f.area();
        let width = 30;
        let height = 3;
        let x = (area.width.saturating_sub(width)) / 2;
        let y = area.height.saturating_sub(height + 2); // Slightly above bottom

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        let paragraph = Paragraph::new(message.as_str())
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        let rect = ratatui::layout::Rect::new(x, y, width, height);
        f.render_widget(Clear, rect);
        f.render_widget(paragraph, rect);
    }
}
