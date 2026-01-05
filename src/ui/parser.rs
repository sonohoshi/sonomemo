use crate::config::Theme;
use crate::ui::color_parser::parse_color;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// 로그 라인의 의미적 구성 요소들을 정의하는 열거형입니다.
#[derive(Debug, PartialEq, Clone)]
pub enum LogToken<'a> {
    Timestamp(&'a str),     // [HH:MM:SS]
    Todo { checked: bool }, // - [ ] or - [x]
    Mood,                   // Mood:
    Tag(&'a str),           // #tag
    Url(&'a str),           // http://...
    Text(&'a str),          // Normal text
    Whitespace(&'a str),    // Space or other whitespace
}

/// 문자열 시작 부분에서 할 일 체크박스("- [ ]" 또는 "- [x]") 패턴을 찾습니다.
/// 발견 시 `Some((체크여부, 매칭된 길이))`를 반환합니다.
pub fn try_parse_todo(text: &str) -> Option<(bool, usize)> {
    static TODO_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let todo_regex = TODO_REGEX.get_or_init(|| regex::Regex::new(r"-\s*\[(\s*|x|X)\]").unwrap());

    if let Some(mat) = todo_regex.find(text) {
        // We use find() which finds the leftmost match.
        // In the context of the parser, we expect this to be called when we are looking for a todo.
        let captured_str = mat.as_str();
        let is_checked = captured_str.contains('x') || captured_str.contains('X');
        return Some((is_checked, mat.end()));
    }
    None
}

/// 표준 체크박스 접두어를 사용하여 할 일 항목 문자열을 포맷팅합니다.
pub fn format_todo(content: &str, checked: bool) -> String {
    let checkbox = if checked { "[x]" } else { "[ ]" };
    format!("- {} {}", checkbox, content)
}

/// 원본 로그 라인을 의미 있는 토큰 리스트로 분리(Tokenize)합니다.
pub fn tokenize(text: &str) -> Vec<LogToken<'_>> {
    let mut tokens = Vec::new();
    let mut current_text = text;

    // 1. Extract Timestamp (Always at the start)
    if current_text.starts_with('[')
        && let Some(end_idx) = current_text.find(']')
    {
        tokens.push(LogToken::Timestamp(&current_text[..=end_idx]));
        current_text = &current_text[end_idx + 1..];
    }

    // 2. Extract Leading Whitespace (needed to separate timestamp from content)
    let trimmed_start = current_text.trim_start();
    let leading_spaces = &current_text[..current_text.len() - trimmed_start.len()];
    if !leading_spaces.is_empty() {
        tokens.push(LogToken::Whitespace(leading_spaces));
    }
    current_text = trimmed_start;

    // 3. Extract Todo Status (Always after timestamp)
    // Regex: hyphen, optional whitespace, open bracket, (optional whitespace OR x/X), closing bracket
    // 3. Extract Todo Status (Always after timestamp)
    if let Some((checked, len)) = try_parse_todo(current_text) {
        tokens.push(LogToken::Todo { checked });
        current_text = &current_text[len..];
    }

    // 4. Tokenize Remaining Content (Words)
    static URL_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let url_regex = URL_REGEX.get_or_init(|| {
        regex::Regex::new(r"https?://[-a-zA-Z0-9+&@#/%?=~_|!:,.;]*[-a-zA-Z0-9+&@#/%=~_|]").unwrap()
    });

    // If we just consumed Todo, there might be a space after it.
    let content_len = current_text.len();
    let content_trimmed = current_text.trim_start();
    let prefix_spaces = &current_text[..content_len - content_trimmed.len()];
    if !prefix_spaces.is_empty() {
        tokens.push(LogToken::Whitespace(prefix_spaces));
    }
    current_text = content_trimmed;

    // Now iterate over words
    // Note: split_whitespace() loses the exact original whitespace chars (tabs vs spaces),
    // but correct UI rendering usually normalizes this anyway.
    // To match previous behavior precisely:

    let words: Vec<&str> = current_text.split(' ').collect();
    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            tokens.push(LogToken::Whitespace(" "));
        }

        if word.is_empty() {
            continue;
        }

        if word.starts_with('#') {
            tokens.push(LogToken::Tag(word));
        } else if *word == "Mood:" {
            tokens.push(LogToken::Mood);
        } else if let Some(mat) = url_regex.find(word) {
            let start = mat.start();
            let end = mat.end();

            if start > 0 {
                tokens.push(LogToken::Text(&word[..start]));
            }
            tokens.push(LogToken::Url(&word[start..end]));
            if end < word.len() {
                tokens.push(LogToken::Text(&word[end..]));
            }
        } else {
            tokens.push(LogToken::Text(word));
        }
    }

    tokens
}

/// 토큰 리스트를 현재 테마를 적용하여 Ratatui `Line` 객체로 렌더링합니다.
pub fn render_tokens<'a>(tokens: Vec<LogToken<'a>>, theme: &Theme) -> Line<'static> {
    let mut spans = Vec::new();

    // Context state
    let mut is_todo_item = false;

    for token in tokens {
        match token {
            LogToken::Timestamp(ts) => {
                let color = parse_color(&theme.timestamp);
                spans.push(Span::styled(ts.to_string(), Style::default().fg(color)));
            }
            LogToken::Whitespace(ws) => {
                spans.push(Span::raw(ws.to_string()));
            }
            LogToken::Todo { checked } => {
                is_todo_item = true;
                if checked {
                    let color = parse_color(&theme.todo_done);
                    spans.push(Span::styled("✅", Style::default().fg(color)));
                } else {
                    let color = parse_color(&theme.todo_wip);
                    spans.push(Span::styled("⬜", Style::default().fg(color)));
                }
            }
            LogToken::Mood => {
                let color = parse_color(&theme.mood);
                spans.push(Span::styled(
                    "🎭 Mood:",
                    Style::default().fg(color).add_modifier(Modifier::ITALIC),
                ));
            }
            LogToken::Tag(tag) => {
                let color = parse_color(&theme.tag);
                spans.push(Span::styled(
                    tag.to_string(),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ));
            }
            LogToken::Url(url) => {
                spans.push(Span::styled(
                    url.to_string(),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::UNDERLINED),
                ));
            }
            LogToken::Text(text) => {
                let mut style = Style::default();
                if is_todo_item {
                    style = style.fg(Color::Reset);
                }
                spans.push(Span::styled(text.to_string(), style));
            }
        }
    }

    Line::from(spans)
}

/// 로그 라인의 체크박스 상태를 토글(체크 <-> 해제)합니다.
/// 변경된 전체 라인 문자열을 반환합니다.
pub fn toggle_checkbox(text: &str) -> String {
    static TODO_REGEX: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let todo_regex = TODO_REGEX.get_or_init(|| regex::Regex::new(r"-\s*\[(\s*|x|X)\]").unwrap());

    // We can't use tokenize() here easily because we want to preserve exact whitespace
    // of non-todo parts, which tokenize() might slightly normalize or split separate from structure.
    // Regex replacement is safer for minimal intrusion.

    if let Some(mat) = todo_regex.find(text) {
        let captured = mat.as_str();
        let is_checked = captured.contains('x') || captured.contains('X');
        let replacement = if is_checked { "- [ ]" } else { "- [x]" };

        let mut new_text = String::with_capacity(text.len());
        new_text.push_str(&text[..mat.start()]);
        new_text.push_str(replacement);
        new_text.push_str(&text[mat.end()..]);
        return new_text;
    }

    text.to_string()
}

/// 완료되지 않은(체크되지 않은) 할 일 항목의 내용을 추출합니다.
/// 할 일이 아니거나 체크된 상태라면 `None`을 반환합니다.
pub fn extract_pending_content(text: &str) -> Option<String> {
    let tokens = tokenize(text);
    let mut is_todo = false;
    let mut is_checked = false;
    let mut content = String::new();

    for token in tokens {
        match token {
            LogToken::Todo { checked } => {
                is_todo = true;
                is_checked = checked;
            }
            LogToken::Text(t) | LogToken::Tag(t) | LogToken::Url(t) | LogToken::Whitespace(t) => {
                // Only collect content AFTER the todo token
                if is_todo {
                    content.push_str(t);
                }
            }
            LogToken::Mood => {
                // Mood can be part of content?
                if is_todo {
                    content.push_str("Mood:");
                }
            }
            _ => {}
        }
    }

    if is_todo && !is_checked {
        Some(content.trim().to_string())
    } else {
        None
    }
}

/// 로그 라인 파싱의 메인 진입점입니다. 토큰화와 렌더링을 한번에 수행합니다.
pub fn parse_log_line(text: &str, theme: &Theme) -> Line<'static> {
    let tokens = tokenize(text);
    render_tokens(tokens, theme)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_complex() {
        let text = "[12:00] - [ ] Study #coding http://rust-lang.org";
        let tokens = tokenize(text);

        assert_eq!(tokens[0], LogToken::Timestamp("[12:00]"));
        assert_eq!(tokens[1], LogToken::Whitespace(" ")); // Space after timestamp
        assert_eq!(tokens[2], LogToken::Todo { checked: false });
        assert_eq!(tokens[3], LogToken::Whitespace(" ")); // Space after todo
        assert_eq!(tokens[4], LogToken::Text("Study"));
        assert_eq!(tokens[5], LogToken::Whitespace(" "));
        assert_eq!(tokens[6], LogToken::Tag("#coding"));
        assert_eq!(tokens[7], LogToken::Whitespace(" "));
        assert_eq!(tokens[8], LogToken::Url("http://rust-lang.org"));
    }

    #[test]
    fn test_tokenize_simple() {
        let text = "Just plain text";
        let tokens = tokenize(text);

        // "Just plain text" -> split by space -> "Just", "plain", "text"
        // Interleaved with whitespace tokens
        assert_eq!(tokens[0], LogToken::Text("Just"));
        assert_eq!(tokens[1], LogToken::Whitespace(" "));
        assert_eq!(tokens[2], LogToken::Text("plain"));
        assert_eq!(tokens[3], LogToken::Whitespace(" "));
        assert_eq!(tokens[4], LogToken::Text("text"));
    }

    #[test]
    fn test_tokenize_mood() {
        let text = "Mood: Happy";
        let tokens = tokenize(text);
        assert_eq!(tokens[0], LogToken::Mood);
        assert_eq!(tokens[1], LogToken::Whitespace(" "));
        assert_eq!(tokens[2], LogToken::Text("Happy"));
    }

    #[test]
    fn test_flexible_todo() {
        // Tight: -[]
        let tokens = tokenize("-[] Tight");
        assert_eq!(tokens[0], LogToken::Todo { checked: false });
        assert_eq!(tokens[1], LogToken::Whitespace(" "));
        assert_eq!(tokens[2], LogToken::Text("Tight"));

        // Wide: - [   ]
        let tokens = tokenize("- [   ] Wide");
        assert_eq!(tokens[0], LogToken::Todo { checked: false });
        assert_eq!(tokens[1], LogToken::Whitespace(" "));
        assert_eq!(tokens[2], LogToken::Text("Wide"));

        // With Timestamp
        let tokens = tokenize("[12:00] -[x] Done");
        assert_eq!(tokens[0], LogToken::Timestamp("[12:00]"));
        assert_eq!(tokens[1], LogToken::Whitespace(" "));
        assert_eq!(tokens[2], LogToken::Todo { checked: true });
        assert_eq!(tokens[3], LogToken::Whitespace(" "));
        assert_eq!(tokens[4], LogToken::Text("Done"));

        // With Timestamp and tight
        let tokens = tokenize("[12:00] -[] noDone");
        assert_eq!(tokens[0], LogToken::Timestamp("[12:00]"));
        assert_eq!(tokens[1], LogToken::Whitespace(" "));
        assert_eq!(tokens[2], LogToken::Todo { checked: false });
        assert_eq!(tokens[3], LogToken::Whitespace(" "));
        assert_eq!(tokens[4], LogToken::Text("noDone"));
    }

    #[test]
    fn test_toggle_checkbox_full_line() {
        // This simulates the string read from file
        let line = "[00:53:21] - [ ] 아니 안되잖아!!!!!!!!";
        let toggled = toggle_checkbox(line);
        assert_eq!(toggled, "[00:53:21] - [x] 아니 안되잖아!!!!!!!!");

        let line_checked = "[00:53:21] - [x] Done";
        let toggled_back = toggle_checkbox(line_checked);
        assert_eq!(toggled_back, "[00:53:21] - [ ] Done");

        // Flexible cases
        let line_tight = "[12:34] -[] Tight";
        let toggled_tight = toggle_checkbox(line_tight);
        assert_eq!(toggled_tight, "[12:34] - [x] Tight"); // Normalized to wide
    }
}
