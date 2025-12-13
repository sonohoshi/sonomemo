# Sonomemo (소노메모) 🧠

**Sonomemo**는 **ADHD를 위한 문맥 기록용 터미널 앱**입니다.

"방금 뭐 하려고 했지?", "지난 1시간 동안 난 뭘 한 거지?"
자꾸 끊기는 생각의 흐름과 문맥을 터미널에서 즉시 붙잡아두세요.
화려한 기능보다는 **빠른 기록**과 **현재 상태 파악**에 집중하여, 당신의 뇌가 길을 잃지 않게 도와줍니다.

![Sonomemo Screenshot](screenshot_placeholder.png)

## ✨ 왜 Sonomemo인가요?

- **🧠 문맥의 외장 하드**: 휘발되는 단기 기억을 즉시 텍스트로 박제하세요.
- **⚡ 로딩 없는 즉시 기록**: 딴짓 할 틈을 주지 않습니다. 켜자마자 바로 적으세요.
- **🍅 강제 환기 (뽀모도로)**: 과몰입의 늪에서 사이렌으로 당신을 구조해줍니다.
- **🌱 시각적 피드백**: 활동 그래프로 자신의 하루 패턴을 객관적으로 마주하세요.
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
