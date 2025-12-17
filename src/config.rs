use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;
use std::fs;
use std::path::Path;

pub fn key_match(key: &KeyEvent, bindings: &[String]) -> bool {
    for binding in bindings {
        if is_match(key, binding) {
            return true;
        }
    }
    false
}

fn is_match(key: &KeyEvent, binding: &str) -> bool {
    let binding = binding.to_lowercase();
    let parts: Vec<&str> = binding.split('+').collect();

    let mut target_modifiers = KeyModifiers::NONE;
    let mut target_code = KeyCode::Null;

    for part in parts {
        match part {
            "ctrl" => target_modifiers.insert(KeyModifiers::CONTROL),
            "opt" | "alt" => target_modifiers.insert(KeyModifiers::ALT),
            "shift" => target_modifiers.insert(KeyModifiers::SHIFT),
            "enter" => target_code = KeyCode::Enter,
            "esc" => target_code = KeyCode::Esc,
            "backspace" => target_code = KeyCode::Backspace,
            "tab" => target_code = KeyCode::Tab,
            "up" => target_code = KeyCode::Up,
            "down" => target_code = KeyCode::Down,
            "left" => target_code = KeyCode::Left,
            "right" => target_code = KeyCode::Right,
            // Handle single characters and other keys
            c if c.chars().count() == 1 => {
                if let Some(ch) = c.chars().next() {
                    target_code = KeyCode::Char(ch);
                }
            }
            _ => {} // Ignore unknown parts
        }
    }

    // Special case: "shift+enter" -> KeyCode::Enter with Shift modifier
    // But crossterm might report KeyCode::Char('\n') or similar depending on terminal?
    // Actually KeyCode::Enter is reported for Enter key.

    // Check modifiers match
    // Note: We only check if the target modifiers are present.
    // If user presses Ctrl+Shift+C but binding says "Ctrl+C", strictly it's not a match?
    // Let's assume strict match for modifiers except maybe ignoring NumLock/CapsLock.

    // Relaxed check: key code matches and required modifiers are present.
    // Allow extra modifiers? No, simpler to be strict.

    if key.code != target_code {
        // Special handling for KeyCode::Char vs shifted char
        // e.g. binding "H" (implicitly shift+h) vs key "h"
        // Current parser lowercases everything: "q" -> Char('q').
        // If user presses 'Q' (Shift+q), crossterm reports Char('Q') and Shift modifier.
        // But my parser sets target code to 'q'.

        if let KeyCode::Char(c) = key.code {
            if let KeyCode::Char(tc) = target_code {
                if c.to_lowercase().next() == Some(tc) {
                    // Chars match case-insensitively, now check modifiers
                    // If binding specified "shift", target_modifiers has SHIFT.
                    // If user pressed Shift, key.modifiers has SHIFT.
                    return key.modifiers.contains(target_modifiers);
                }
            }
        }
        return false;
    }

    key.modifiers.contains(target_modifiers)
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub placeholders: Placeholders,
    #[serde(default)]
    pub help: HelpMessages,
    #[serde(default)]
    pub keybindings: KeyBindings,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub data: DataConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DataConfig {
    pub log_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Placeholders {
    pub navigate: String,
    pub editing: String,
    pub search: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HelpMessages {
    pub navigate: String,
    pub editing: String,
    pub search: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct KeyBindings {
    #[serde(default)]
    pub navigate: NavigateBindings,
    #[serde(default)]
    pub editing: EditingBindings,
    #[serde(default)]
    pub search: SearchBindings,
    #[serde(default)]
    pub popup: PopupBindings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NavigateBindings {
    pub quit: Vec<String>,
    pub tags: Vec<String>,
    pub insert: Vec<String>,
    pub search: Vec<String>,
    pub pomodoro: Vec<String>,
    pub graph: Vec<String>,
    pub toggle_todo: Vec<String>,
    pub path: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EditingBindings {
    pub save: Vec<String>,    // Enter
    pub newline: Vec<String>, // Shift+Enter
    pub cancel: Vec<String>,  // Esc
}

#[derive(Debug, Deserialize, Clone)]
pub struct SearchBindings {
    pub submit: Vec<String>,
    pub cancel: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PopupBindings {
    pub confirm: Vec<String>,
    pub cancel: Vec<String>,
    pub up: Vec<String>,
    pub down: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Theme {
    pub border_default: String,
    pub border_editing: String,
    pub border_search: String,
    pub border_todo_header: String,
    pub text_highlight: String,
    pub todo_done: String,
    pub todo_wip: String,
    pub tag: String,
    pub mood: String,
    pub timestamp: String,
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            log_path: "logs".to_string(),
        }
    }
}

impl Default for Placeholders {
    fn default() -> Self {
        Self {
            navigate: "키를 눌러 각종 기능을 사용하세요...".to_string(),
            editing: "여기에 메모를 입력하세요...".to_string(),
            search: "검색할 단어를 입력하세요...".to_string(),
        }
    }
}

impl Default for HelpMessages {
    fn default() -> Self {
        Self {
            navigate:
                " [i] Edit  [t] Tag  [?] Search  [Enter] Toggle  [p] Pomodoro  [g] Graph  [q] Quit "
                    .to_string(),
            editing: " [Esc] Navigate Mode  [Enter] Save Memo  [Shift+Enter] New Line ".to_string(),
            search: " [Esc] Reset Search  [Enter] Filter ".to_string(),
        }
    }
}

impl Default for NavigateBindings {
    fn default() -> Self {
        Self {
            quit: vec!["q".to_string(), "ㅂ".to_string()],
            tags: vec!["t".to_string(), "ㅅ".to_string()],
            insert: vec!["i".to_string(), "ㅑ".to_string()],
            search: vec!["?".to_string()],
            pomodoro: vec!["p".to_string(), "ㅔ".to_string()],
            graph: vec!["g".to_string(), "ㅎ".to_string()],
            toggle_todo: vec!["enter".to_string()],
            path: vec!["l".to_string(), "ㅣ".to_string()],
        }
    }
}

impl Default for EditingBindings {
    fn default() -> Self {
        Self {
            save: vec!["enter".to_string()],
            newline: vec!["shift+enter".to_string()],
            cancel: vec!["esc".to_string()],
        }
    }
}

impl Default for SearchBindings {
    fn default() -> Self {
        Self {
            submit: vec!["enter".to_string()],
            cancel: vec!["esc".to_string()],
        }
    }
}

impl Default for PopupBindings {
    fn default() -> Self {
        Self {
            confirm: vec!["enter".to_string(), "y".to_string(), "ㅛ".to_string()],
            cancel: vec!["esc".to_string(), "n".to_string(), "ㅜ".to_string()],
            up: vec!["up".to_string()],
            down: vec!["down".to_string()],
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border_default: "Reset".to_string(),
            border_editing: "Green".to_string(),
            border_search: "Cyan".to_string(),
            border_todo_header: "Yellow".to_string(),
            text_highlight: "50,50,50".to_string(), // RGB background
            todo_done: "Green".to_string(),
            todo_wip: "Red".to_string(),
            tag: "Yellow".to_string(),
            mood: "Magenta".to_string(),
            timestamp: "Blue".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Path::new("config.toml");
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                } else {
                    eprintln!("Failed to parse config.toml, using defaults.");
                }
            }
        }
        Self::default()
    }
}
