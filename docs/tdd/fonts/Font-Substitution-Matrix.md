# Font Substitution Matrix

## Goals
- Provide deterministic cross-platform fallback.
- Minimize layout drift between OSes.

## Core Substitutions (Examples)
| Requested | Windows Fallback | macOS Fallback | Linux Fallback |
| --- | --- | --- | --- |
| Calibri | Calibri | Helvetica Neue | Liberation Sans |
| Times New Roman | Times New Roman | Times | Liberation Serif |
| Arial | Arial | Helvetica Neue | Liberation Sans |
| Cambria | Cambria | Times New Roman | Liberation Serif |
| Courier New | Courier New | Menlo | Liberation Mono |

## Rules
- Preserve original font name in document.
- Use platform-specific fallback table for rendering.
- Store chosen fallback in layout cache for stability.

## CJK
- Japanese: MS Gothic -> Hiragino Sans (macOS) -> Noto Sans CJK JP (Linux)
- Chinese: SimSun -> PingFang SC -> Noto Sans CJK SC
- Korean: Malgun Gothic -> Apple SD Gothic Neo -> Noto Sans CJK KR

## RTL
- Arabic: Arial -> Geeza Pro -> Noto Naskh Arabic
- Hebrew: Arial -> Helvetica -> Noto Sans Hebrew
