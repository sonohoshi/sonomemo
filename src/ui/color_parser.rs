use ratatui::style::Color;

pub fn parse_color(s: &str) -> Color {
    let s = s.trim().to_lowercase();
    match s.as_str() {
        "reset" => Color::Reset,
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "gray" => Color::Gray,
        "darkgray" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "white" => Color::White,
        _ => {
            if s.contains(',') {
                let parts: Vec<&str> = s.split(',').collect();
                if parts.len() == 3
                    && let (Ok(r), Ok(g), Ok(b)) = (
                        parts[0].trim().parse(),
                        parts[1].trim().parse(),
                        parts[2].trim().parse(),
                    )
                {
                    return Color::Rgb(r, g, b);
                }
            }
            Color::Reset
        }
    }
}
