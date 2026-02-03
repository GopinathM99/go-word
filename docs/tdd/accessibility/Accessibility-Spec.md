# Accessibility Specification

## Goals
- Full keyboard accessibility.
- Screen reader compatibility with ARIA semantics.
- High contrast and scaling support.

## Screen Reader Roles
- Document surface: role="document" or custom role with ARIA labeling.
- Paragraphs: role="paragraph" (or group) with accessible name.
- Text runs: expose as text nodes via accessibility tree.

## Keyboard Navigation
- Tab cycles through UI controls.
- Arrow keys move caret by grapheme/word/line.
- Ctrl/Cmd+Arrow for word navigation.
- PageUp/PageDown for page navigation.

## Focus Management
- Single focus ring for editor surface.
- When focus is in editor, caret position is announced.

## High Contrast
- Respect OS high-contrast settings.
- Provide custom theme with high contrast palette.

## Scaling
- Support UI scaling up to 200%.
- Text rendering scales without layout glitches.

## ARIA Labels
- Toolbar groups labeled (e.g., "Font", "Paragraph").
- Buttons have aria-label and shortcut hints.
