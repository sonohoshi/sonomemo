use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 주어진 키 입력이 특정 바인딩 목록 중 하나와 일치하는지 확인합니다.
///
/// `key`: 사용자가 입력한 키 이벤트
/// `bindings`: "ctrl+c", "enter" 등 설정된 키 바인딩 문자열 리스트
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

        if let KeyCode::Char(c) = key.code
            && let KeyCode::Char(tc) = target_code
            && c.to_lowercase().next() == Some(tc)
            && key.modifiers.contains(target_modifiers)
        {
            return true;
        }
        return false;
    }

    key.modifiers.contains(target_modifiers)
}

/// 애플리케이션의 전체 설정을 담는 최상위 구조체입니다.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
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

/// 데이터 관련 설정입니다 (예: 로그 저장 경로).
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataConfig {
    pub log_path: String,
}

/// UI의 입력 필드에 표시될 플레이스홀더 텍스트 설정입니다.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Placeholders {
    pub navigate: String,
    pub editing: String,
    pub search: String,
}

/// UI 하단 등에 표시될 도움말 메시지 설정입니다.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HelpMessages {
    pub navigate: String,
    pub editing: String,
    pub search: String,
}

/// 전체 키 바인딩 설정을 모아둔 구조체입니다.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
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

/// 'Navigate' (기본 탐색) 모드에서의 키 바인딩입니다.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NavigateBindings {
    pub quit: Vec<String>,
    pub tags: Vec<String>,
    pub insert: Vec<String>,
    pub search: Vec<String>,
    pub pomodoro: Vec<String>,
    pub graph: Vec<String>,
    pub toggle_todo: Vec<String>,
    pub path: Vec<String>,
    #[serde(default = "default_next_todo")]
    pub next_todo: Vec<String>,
    #[serde(default = "default_prev_todo")]
    pub prev_todo: Vec<String>,
    #[serde(default = "default_copy")]
    pub copy: Vec<String>,
}

fn default_next_todo() -> Vec<String> {
    vec!["]".to_string()]
}
fn default_prev_todo() -> Vec<String> {
    vec!["[".to_string()]
}
fn default_copy() -> Vec<String> {
    vec!["y".to_string(), "ㅛ".to_string()]
}

/// 'Editing' (작성/수정) 모드에서의 키 바인딩입니다.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EditingBindings {
    pub save: Vec<String>,    // Enter
    pub newline: Vec<String>, // Shift+Enter
    pub cancel: Vec<String>,  // Esc
}

/// 'Search' (검색) 모드에서의 키 바인딩입니다.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchBindings {
    pub submit: Vec<String>,
    pub cancel: Vec<String>,
}

/// 팝업 창 등에서의 키 바인딩입니다.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PopupBindings {
    pub confirm: Vec<String>,
    pub cancel: Vec<String>,
    pub up: Vec<String>,
    pub down: Vec<String>,
}

/// UI 색상 테마 설정입니다. 가능한 색상은 `tui` 크레이트의 색상 이름(예: "Red", "Blue") 혹은 RGB 값("r,g,b")입니다.
#[derive(Debug, Deserialize, Serialize, Clone)]
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
                " [i] Edit  [t] Tag  [?] Search  [Enter] Toggle  [p] Pomodoro  [y] Copy  [[]] Todo Nav  [g] Graph  [l] PATH  [q] Quit "
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
            next_todo: default_next_todo(),
            prev_todo: default_prev_todo(),
            copy: default_copy(),
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
    /// `config.toml` 파일에서 설정을 로드합니다.
    ///
    /// 파일이 존재하지 않거나 파싱에 실패하면 기본값을 사용하며,
    /// 파일이 없을 경우 기본 설정으로 새 파일을 생성합니다.
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
            return Self::default();
        }

        let default_config = Self::default();
        if let Ok(toml_str) = toml::to_string_pretty(&default_config)
            && let Err(e) = fs::write(config_path, toml_str)
        {
            eprintln!("Failed to write default config: {}", e);
        }
        default_config
    }
}
