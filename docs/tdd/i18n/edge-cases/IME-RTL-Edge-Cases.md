# IME and RTL Edge Cases

## IME Edge Cases
- Composition across line breaks.
- Composition across styled runs.
- Backspace during composition.

## RTL Edge Cases
- Mixed LTR/RTL in same line.
- Cursor movement at script boundaries.
- Selection spanning RTL and LTR segments.

## Combined Scenarios
- IME composition in RTL paragraphs.
- RTL text in tables and headers.

## Testing
- Use sample docs for Arabic, Hebrew, Japanese, Chinese.
- Verify caret visual position vs logical offset.
