# Spellcheck — Detailed Spec

## Sources
- Integrate Hunspell dictionaries or platform spell API.
- Support multi-language per paragraph.

## Behavior
- Underline misspelled words in red.
- Show suggestions on right-click.

## Performance
- Spellcheck in background thread.
- Cache results per paragraph hash.

## Ignore Rules
- Ignore words in code spans or fields if configured.
- Maintain user dictionary.
