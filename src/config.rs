use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub placeholders: Placeholders,
    #[serde(default)]
    pub help: HelpMessages,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            placeholders: Placeholders::default(),
            help: HelpMessages::default(),
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

impl Config {
    pub fn load() -> Self {
        let config_path = Path::new("config.toml");
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }
}
