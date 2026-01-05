/// 애플리케이션의 현재 입력 모드를 정의합니다.
#[derive(PartialEq)]
pub enum InputMode {
    /// 로그 리스트를 탐색하는 기본 모드입니다.
    Navigate,
    /// 새로운 로그를 작성하거나 수정하는 편집 모드입니다.
    Editing,
    /// 로그 내용을 검색하는 모드입니다.
    Search,
}

/// 사용자의 기분 상태를 나타내는 열거형입니다.
#[derive(Clone, Copy, PartialEq)]
pub enum Mood {
    /// 기분이 좋을 때 (Happy)
    Happy,
    /// 평범할 때 (Neutral)
    Neutral,
    /// 스트레스 받을 때 (Stressed)
    Stressed,
    /// 집중하고 있을 때 (Focused)
    Focused,
    /// 피곤할 때 (Tired)
    Tired,
}

impl Mood {
    /// 지원되는 모든 기분 상태의 리스트를 반환합니다.
    pub fn all() -> Vec<Mood> {
        vec![
            Mood::Happy,
            Mood::Neutral,
            Mood::Stressed,
            Mood::Focused,
            Mood::Tired,
        ]
    }

    /// 기분 상태에 해당하는 이모지와 텍스트 설명을 반환합니다.
    pub fn as_str(&self) -> &'static str {
        match self {
            Mood::Happy => "😊 조음",
            Mood::Neutral => "😐 걍그럼",
            Mood::Stressed => "😫 구림",
            Mood::Focused => "🧐 집중",
            Mood::Tired => "😴 피곤",
        }
    }
}

/// 파싱된 로그 항목을 나타내는 구조체입니다.
#[derive(Clone)]
pub struct LogEntry {
    /// 로그의 원본 텍스트 내용입니다.
    pub content: String,
    /// 로그가 저장된 파일의 경로입니다.
    pub file_path: String,
    /// 파일 내에서의 라인 번호입니다 (0-based 또는 1-based, storage 구현에 따름).
    pub line_number: usize,
}
