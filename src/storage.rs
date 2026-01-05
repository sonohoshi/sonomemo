use crate::models::LogEntry;
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

/// 지정된 경로에 로그 디렉토리가 존재하는지 확인하고, 없으면 생성합니다.
pub fn ensure_log_dir(log_path: &str) -> io::Result<()> {
    let path = PathBuf::from(log_path);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 오늘 날짜에 해당하는 로그 파일의 경로를 생성합니다.
fn get_today_file_path(log_path: &str) -> PathBuf {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut path = PathBuf::from(log_path);
    path.push(format!("{}.md", today));
    path
}

/// 오늘 로그 파일에 새로운 항목을 추가합니다.
///
/// `content`: 추가할 로그 내용
pub fn append_entry(log_path: &str, content: &str) -> io::Result<()> {
    ensure_log_dir(log_path)?;
    let path = get_today_file_path(log_path);

    let time = Local::now().format("%H:%M:%S").to_string();
    let line = format!("[{}] {}\n", time, content);

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    file.write_all(line.as_bytes())?;
    Ok(())
}

/// 오늘 작성된 모든 로그 항목을 읽어옵니다.
pub fn read_today_entries(log_path: &str) -> io::Result<Vec<LogEntry>> {
    ensure_log_dir(log_path)?;
    let path = get_today_file_path(log_path);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let path_str = path.to_string_lossy().to_string();
    let content = fs::read_to_string(&path)?;

    Ok(parse_log_content(&content, &path_str))
}

/// 모든 로그 파일에서 검색어(`query`)가 포함된 항목을 찾습니다.
pub fn search_entries(log_path: &str, query: &str) -> io::Result<Vec<LogEntry>> {
    ensure_log_dir(log_path)?;
    let dir = PathBuf::from(log_path);
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let path_str = path.to_string_lossy().to_string();
                let date_str = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                if let Ok(content) = fs::read_to_string(&path) {
                    let parsed_entries = parse_log_content(&content, &path_str);
                    for entry in parsed_entries {
                        if entry.content.contains(query) {
                            // 날짜 정보 추가
                            let display_content = format!("[{}] {}", date_str, entry.content);

                            results.push(LogEntry {
                                content: display_content,
                                file_path: entry.file_path,
                                line_number: entry.line_number,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}

/// 로그 파일의 내용을 파싱하여 `LogEntry` 리스트로 변환합니다.
/// 들여쓰기된 라인은 이전 항목의 내용으로 병합 처리합니다.
fn parse_log_content(content: &str, path_str: &str) -> Vec<LogEntry> {
    let mut entries: Vec<LogEntry> = Vec::new();

    for (i, line) in content.lines().enumerate() {
        if line.contains("System: Carryover Checked") {
            continue;
        }

        let is_continuation = line.starts_with("  ") || line.starts_with('\t');

        if is_continuation && let Some(last) = entries.last_mut() {
            last.content.push('\n');
            last.content.push_str(line);
            continue;
        }

        entries.push(LogEntry {
            content: line.to_string(),
            file_path: path_str.to_string(),
            line_number: i,
        });
    }
    entries
}

// ... imports
// Remove regex import if not used elsewhere (it was only used here)
// use regex::Regex;
use crate::ui::parser;

// ...

/// 특정 로그 항목의 할 일(체크박스) 상태를 토글합니다.
/// 해당 파일의 정확한 라인을 찾아 내용을 수정합니다.
pub fn toggle_todo_status(entry: &LogEntry) -> io::Result<()> {
    let content = fs::read_to_string(&entry.file_path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    if entry.line_number < lines.len() {
        // Use shared parser logic to toggle
        lines[entry.line_number] = parser::toggle_checkbox(&lines[entry.line_number]);
    }

    let mut new_content = lines.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    let mut file = fs::File::create(&entry.file_path)?;
    file.write_all(new_content.as_bytes())?;

    Ok(())
}

/// 가장 최근(오늘 제외) 로그 파일에서 완료되지 않은 할 일 목록을 가져옵니다.
/// 이월(carryover) 기능을 위해 사용됩니다.
pub fn get_last_file_pending_todos(log_path: &str) -> io::Result<Vec<String>> {
    ensure_log_dir(log_path)?;
    let dir = PathBuf::from(log_path);
    let today = Local::now().format("%Y-%m-%d").to_string();

    if let Ok(entries) = fs::read_dir(dir) {
        let mut file_paths = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md")
                && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                && stem != today
            {
                file_paths.push(path);
            }
        }
        file_paths.sort();

        if let Some(last_path) = file_paths.last() {
            let mut todos = Vec::new();
            if let Ok(content) = fs::read_to_string(last_path) {
                for line in content.lines() {
                    // Use shared parser logic to extract pending todos
                    if let Some(todo_content) = parser::extract_pending_content(line) {
                        todos.push(todo_content);
                    }
                }
            }
            return Ok(todos);
        }
    }
    Ok(Vec::new())
}

/// 모든 로그 파일에서 태그('#')를 추출하여 빈도수 순으로 정렬해 반환합니다.
pub fn get_all_tags(log_path: &str) -> io::Result<Vec<(String, usize)>> {
    use std::collections::HashMap;

    ensure_log_dir(log_path)?;
    let dir = PathBuf::from(log_path);
    let mut tag_counts = HashMap::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md")
                && let Ok(content) = fs::read_to_string(&path)
            {
                for line in content.lines() {
                    for word in line.split_whitespace() {
                        if word.starts_with('#') && word.len() > 1 {
                            *tag_counts.entry(word.to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    let mut tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
    // 많이 쓰인 순서대로 정렬 (내림차순)
    tags.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(tags)
}

/// 오늘 할 일 이월 작업이 이미 수행되었는지 확인합니다.
pub fn is_carryover_done(log_path: &str) -> io::Result<bool> {
    ensure_log_dir(log_path)?;
    let path = get_today_file_path(log_path);
    if !path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(path)?;
    Ok(content.contains("System: Carryover Checked"))
}

/// 오늘 할 일 이월 작업이 완료되었음을 시스템 마커로 기록합니다.
pub fn mark_carryover_done(log_path: &str) -> io::Result<()> {
    append_entry(log_path, "System: Carryover Checked")
}

/// 날짜별 활동(로그 수) 통계를 수집합니다.
/// "잔디밭(contribution graph)" 시각화를 위해 사용됩니다.
pub fn get_activity_stats(log_path: &str) -> io::Result<std::collections::HashMap<String, usize>> {
    use std::collections::HashMap;

    ensure_log_dir(log_path)?;
    let dir = PathBuf::from(log_path);
    let mut stats = HashMap::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md")
                && let Some(filename) = path.file_stem().and_then(|s| s.to_str())
            {
                // 파일명(YYYY-MM-DD)을 키로 사용
                if let Ok(content) = fs::read_to_string(&path) {
                    // 빈 줄이나 시스템 마커 제외하고 카운트
                    let count = content
                        .lines()
                        .filter(|l| {
                            !l.trim().is_empty() && !l.contains("System: Carryover Checked")
                        })
                        .count();
                    stats.insert(filename.to_string(), count);
                }
            }
        }
    }
    Ok(stats)
}
