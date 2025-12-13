#[derive(PartialEq)]
pub enum InputMode {
    Navigate,
    Editing,
    Search,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mood {
    Happy,
    Neutral,
    Stressed,
    Focused,
    Tired,
}

impl Mood {
    pub fn all() -> Vec<Mood> {
        vec![Mood::Happy, Mood::Neutral, Mood::Stressed, Mood::Focused, Mood::Tired]
    }
    
    pub fn to_str(&self) -> &'static str {
        match self {
            Mood::Happy => "😊 조음",
            Mood::Neutral => "😐 걍그럼",
            Mood::Stressed => "😫 구림",
            Mood::Focused => "🧐 집중",
            Mood::Tired => "😴 피곤",
        }
    }
}

#[derive(Clone)]
pub struct LogEntry {
    pub content: String,
    pub file_path: String,
    pub line_number: usize,
}
