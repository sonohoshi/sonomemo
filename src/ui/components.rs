use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
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

pub fn parse_log_line(text: &str) -> Line<'static> {
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
