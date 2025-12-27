# AI Context for Sonomemo Project 🧠

이 파일은 AI 에이전트가 Sonomemo 프로젝트의 맥락을 빠르게 파악하고 일관된 도움을 주기 위해 작성되었습니다.
새로운 대화를 시작할 때 이 파일의 내용을 참고하세요.

## 1. 기본 규칙 (Basic Rules)
- **언어 (Language)**: 모든 대화, 코드 설명, 주석, 문서는 **한국어(Korean)**로 작성한다.
- **태도**: 친절하고 명확하게 설명하며, 사용자의 의도를 먼저 파악한다. 유지보수하기 쉬운 코드를 지향한다.
- **검증 및 포맷팅**:
    - 코드를 수정한 후에는 반드시 `cargo fmt`를 실행하여 코드 스타일을 정리한다.
    - 또, `cargo clippy`를 실행하여 코드 품질을 검증한다.
    - 이후 `cargo check`를 실행하여 컴파일 에러가 없는지 확인한다.
- **컨텍스트 유지**: 프로젝트의 아키텍처, 주요 기술 스택, 혹은 컨벤션에 변경이 있을 경우, **반드시 이 파일(`AI_CONTEXT.md`)을 갱신**하여 최신 상태를 유지한다.
- **문제 해결 공유**: 작업 도중 예기치 않은 문제(컴파일 에러, 크래시 등)가 발생하면, 반드시 **원인**과 **어떻게 수정했는지**를 사용자에게 명확히 설명한다.

## 2. 프로젝트 개요 (Overview)
- **이름**: Sonomemo (소노메모)
- **목적**: ADHD 사용자를 위한 터미널 기반 문맥 기록(Context Logging) 및 생산성 도구.
- **핵심 가치**: 빠른 입력, 시각적 피드백(잔디밭, 무드), 방해 없는 UX.

## 3. 기술 스택 (Tech Stack)
- **Language**: Rust 🦀 (Edition 2024)
- **TUI Framework**: `ratatui`, `crossterm`
- **Input Handling**: `tui-textarea`, `crossterm` event loop
- **Utils**: `regex` (URL detection)
- **Data**: 로컬 Markdown 파일 (`logs/YYYY-MM-DD.md` or configurable `data.log_path`)

## 4. 아키텍처 및 주요 파일 (Files)
- **설정 (`config.toml`)**: 사용자 정의 키 바인딩, 테마 색상, 로그 경로. (없으면 실행 시 자동 생성됨)
- **진입점 (`src/main.rs`)**:
    - 앱 초기화, 메인 이벤트 루프.
    - **중요**: macOS `Shift+Enter` 지원을 위해 `KeyboardEnhancementFlags`가 활성화되어 있음.
- **설정 로직 (`src/config.rs`)**: TOML 파싱, 키 매칭 헬퍼(`key_match`) 함수 제공.
- **UI (`src/ui/`)**:
    - `mod.rs`: 전체 레이아웃 (로그 뷰, 할 일, 입력창).
    - `parser.rs`: 로그 라인 파싱(`tokenize`, `try_parse_todo`) 및 포맷팅(`format_todo`).
    - `color_parser.rs`: 테마 색상 문자열 파싱.
- **데이터 (`src/storage.rs`)**: 파일 I/O 및 파싱 로직.

## 5. 컨벤션 (Conventions)
- **키 바인딩**: 하드코딩하지 않고 `app.config.keybindings`를 참조한다.
- **색상**: `Color::Red` 대신 `app.config.theme`의 값을 사용한다.
- **커밋/배포**: 버전 업데이트 시 `Cargo.toml` 수정 및 `cargo check` 필수.
