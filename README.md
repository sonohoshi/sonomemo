# Sonomemo (소노메모) 🌿

**Sonomemo**는 개발자와 파워 유저를 위한 **키보드 중심의 터미널 메모 애플리케이션**입니다.
Rust와 Ratatui로 제작되어 가볍고 빠르며, 마우스를 건드리지 않고도 생각의 흐름을 유지하며 메모를 남길 수 있습니다.

![Sonomemo Screenshot](screenshot_placeholder.png)

## ✨ 주요 기능

- **⚡ 초고속 메모 작성**: 켜자마자 바로 입력 모드. 생각나는 즉시 기록하세요.
- **📝 마크다운 기반**: 모든 데이터는 `YYYY-MM-DD.md` 형식의 로컬 텍스트 파일로 저장됩니다.
- **🍅 뽀모도로 타이머**: 몰입을 위한 타이머와 강제 휴식 알림(Siren) 기능.
- **📊 활동 그래프 (잔디)**: 지난 2주간의 기록 습관을 시각적으로 확인하세요.
- **✅ 할 일 및 태그**: `- [ ]` 문법으로 할 일 자동 인식, `#태그`로 분류.

## 🚀 설치 방법

### 요구 사항
- Rust (Cargo)가 설치되어 있어야 합니다. (https://rustup.rs/)
- 트루컬러(TrueColor)를 지원하는 터미널 권장 (Windows Terminal, iTerm2, Alacritty 등)

### 빌드 및 실행
```bash
git clone https://github.com/sonohoshi/sonomemo.git
cd sonomemo
cargo install --path .
# 이제 어디서든 'sonomemo'를 입력하여 실행할 수 있습니다!
```

### Crates.io를 통한 설치 (추천)
Rust가 설치되어 있다면 가장 간편한 방법입니다.
```bash
cargo install sonomemo
```

### 소스코드 빌드
```bash
git clone https://github.com/sonohoshi/sonomemo.git
cd sonomemo
cargo install --path .
```

## ⌨️ 단축키 (Keybindings)

| 키 | 모드 | 설명 |
|:--- |:--- |:--- |
| `i` | Normal | 입력 모드 전환 (메모 작성) |
| `Esc` | Any | Normal 모드 복귀 / 팝업 닫기 |
| `?` | Normal | 검색 모드 진입 |
| `t` | Normal | 태그 필터링 |
| `p` | Normal | **뽀모도로 타이머 설정** |
| `g` | Normal | **활동 그래프 확인** |

## 🛠️ 기여하기 (Contributing)
이 프로젝트는 오픈소스입니다. 버그 제보나 기능 제안은 Issue를 통해 남겨주세요!

## 📄 라이선스
MIT License (LICENSE 파일을 확인하세요)