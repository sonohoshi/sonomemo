use crate::config::Theme;
use crate::ui::color_parser::parse_color;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

// 팝업 위치 계산 헬퍼
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

pub fn parse_log_line(text: &str, theme: &Theme) -> Line<'static> {
    let mut spans = Vec::new();

    // 타임스탬프 처리 [HH:MM:SS]
    let parts: Vec<&str> = text.splitn(2, "] ").collect();
    if parts.len() == 2 && parts[0].starts_with('[') {
        let timestamp_color = parse_color(&theme.timestamp);
        spans.push(Span::styled(
            format!("{}] ", parts[0]),
            Style::default().fg(timestamp_color),
        ));

        let content = parts[1];

        // TODO 체크박스 처리
        let (content, todo_prefix) = if content.starts_with("- [ ] ") {
            let color = parse_color(&theme.todo_wip);
            spans.push(Span::styled("⬜ ", Style::default().fg(color))); // 미완료 이모지
            (&content[6..], true)
        } else if content.starts_with("- [x] ") {
            let color = parse_color(&theme.todo_done);
            spans.push(Span::styled("✅ ", Style::default().fg(color))); // 완료 이모지
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
                let tag_color = parse_color(&theme.tag);
                spans.push(Span::styled(
                    word.to_string(),
                    Style::default().fg(tag_color).add_modifier(Modifier::BOLD),
                ));
            } else if word.starts_with("Mood:") {
                let mood_color = parse_color(&theme.mood);
                spans.push(Span::styled(
                    "🎭 Mood:",
                    Style::default()
                        .fg(mood_color)
                        .add_modifier(Modifier::ITALIC),
                ));
            } else {
                if todo_prefix {
                    spans.push(Span::styled(
                        word.to_string(),
                        Style::default().fg(Color::Reset),
                    ));
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
