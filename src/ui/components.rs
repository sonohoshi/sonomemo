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
    // 텍스트가 [로 시작하고 ]가 있는 경우 타임스탬프로 간주
    let timestamp_match = if text.starts_with('[') {
        text.find(']').map(|i| (i, &text[..=i], &text[i + 1..]))
    } else {
        None
    };

    if let Some((_, timestamp_part, mut content_part)) = timestamp_match {
        // 타임스탬프 뒤 공백 제거
        if content_part.starts_with(' ') {
            content_part = &content_part[1..];
        }

        let timestamp_color = parse_color(&theme.timestamp);
        spans.push(Span::styled(
            format!("{} ", timestamp_part), // 표시할 땐 공백 추가
            Style::default().fg(timestamp_color),
        ));

        let content = content_part;

        // TODO 체크박스 처리
        let (content, todo_prefix) = if let Some(stripped) = content.strip_prefix("- [ ] ") {
            let color = parse_color(&theme.todo_wip);
            spans.push(Span::styled("⬜ ", Style::default().fg(color))); // 미완료 이모지
            (stripped, true)
        } else if let Some(stripped) = content.strip_prefix("- [x] ") {
            let color = parse_color(&theme.todo_done);
            spans.push(Span::styled("✅ ", Style::default().fg(color))); // 완료 이모지
            (stripped, true)
        } else {
            (content, false)
        };

        // 태그 파싱 (#단어)
        static URL_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let url_regex = URL_REGEX.get_or_init(|| {
            regex::Regex::new(r"https?://[-a-zA-Z0-9+&@#/%?=~_|!:,.;]*[-a-zA-Z0-9+&@#/%=~_|]")
                .unwrap()
        });

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
            } else if let Some(mat) = url_regex.find(word) {
                let start = mat.start();
                let end = mat.end();

                // URL 앞부분 (괄호 등)
                if start > 0 {
                    spans.push(Span::styled(
                        word[..start].to_string(),
                        if todo_prefix {
                            Style::default().fg(Color::Reset)
                        } else {
                            Style::default()
                        },
                    ));
                }

                // URL 본문
                spans.push(Span::styled(
                    word[start..end].to_string(),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                ));

                // URL 뒷부분
                if end < word.len() {
                    spans.push(Span::styled(
                        word[end..].to_string(),
                        if todo_prefix {
                            Style::default().fg(Color::Reset)
                        } else {
                            Style::default()
                        },
                    ));
                }
            } else if todo_prefix {
                spans.push(Span::styled(
                    word.to_string(),
                    Style::default().fg(Color::Reset),
                ));
            } else {
                spans.push(Span::raw(word.to_string()));
            }
        }
    } else {
        // 형식이 없는 일반 텍스트
        spans.push(Span::raw(text.to_string()));
    }

    Line::from(spans)
}
