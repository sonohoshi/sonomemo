use super::components::centered_rect;
use crate::app::App;
use crate::models::Mood;
use chrono::Local;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

pub fn render_siren_popup(f: &mut Frame) {
    let block = Block::default().borders(Borders::ALL).style(
        Style::default()
            .fg(Color::Red)
            .bg(Color::Black)
            .add_modifier(Modifier::BOLD | Modifier::RAPID_BLINK),
    );

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
        art_spans.push(ListItem::new(Line::from(Span::styled(
            line,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))));
    }

    f.render_widget(List::new(art_spans), text_area);
}

pub fn render_activity_popup(f: &mut Frame, app: &App) {
    let block = Block::default()
        .title(" 🌱 Activity Graph (Last 2 Weeks) ")
        .borders(Borders::ALL);
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let today = Local::now().date_naive();
    let mut items = Vec::new();

    for i in 0..14 {
        let date = today - chrono::Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();
        let count = app.activity_data.get(&date_str).cloned().unwrap_or(0);

        let bar_len = count.min(20); // 최대 20칸
        let bar: String = "■".repeat(bar_len);

        let color = if count == 0 {
            Color::DarkGray
        } else if count < 5 {
            Color::Green
        } else {
            Color::LightGreen
        };

        items.push(ListItem::new(Line::from(vec![
            Span::raw(format!("{} : {:3} logs ", date_str, count)),
            Span::styled(bar, Style::default().fg(color)),
        ])));
    }

    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .margin(2)
        .split(area)[0];

    f.render_widget(List::new(items), inner_area);
}

pub fn render_pomodoro_popup(f: &mut Frame, app: &App) {
    let block = Block::default()
        .title(" 🍅 Set Timer (Minutes) ")
        .borders(Borders::ALL);
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let input_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1)])
        .margin(2)
        .split(area)[0];

    let text = Paragraph::new(format!("{} _", app.pomodoro_input))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(text, input_area);
}

pub fn render_mood_popup(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title(" 기분이가 좀 어떠세여? ")
        .borders(Borders::ALL);
    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let moods = Mood::all();
    let items: Vec<ListItem> = moods.iter().map(|m| ListItem::new(m.to_str())).collect();

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

pub fn render_todo_popup(f: &mut Frame, app: &mut App) {
    let title = format!(
        " 지난 할 일이 {}개 남았습니다. 오늘로 가져올까요? (Y/n) ",
        app.pending_todos.len()
    );
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightRed));
    let area = centered_rect(70, 40, f.area());
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = app
        .pending_todos
        .iter()
        .map(|t| ListItem::new(format!("• {}", t)))
        .collect();

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .margin(1)
        .split(area);

    let list = List::new(items).highlight_symbol(">> ");

    f.render_stateful_widget(list, popup_layout[0], &mut app.todo_list_state);
}

pub fn render_tag_popup(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title(" 태그를 선택하세요 (Enter: 검색, Esc: 닫기) ")
        .borders(Borders::ALL);
    let area = centered_rect(50, 60, f.area());
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = app
        .tags
        .iter()
        .map(|(tag, count)| ListItem::new(format!("{} ({})", tag, count)))
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
