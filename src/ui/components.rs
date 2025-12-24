use crate::config::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::Line,
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
    crate::ui::parser::parse_log_line(text, theme)
}
