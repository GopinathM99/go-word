# OOXML Numbering and Lists — Detailed

## Parts
- /word/numbering.xml

## Core Elements
- w:abstractNum: list template with multiple levels.
- w:num: concrete list instance referencing abstractNum.
- w:lvl: level definition.
- w:lvlOverride: per-instance overrides.

## Mapping
- Internal ListTemplate <-> w:abstractNum
- Internal ListInstance <-> w:num

## Level Properties (w:lvl)
- w:start: initial number
- w:numFmt: format (decimal, roman, letter, bullet)
- w:lvlText: display template (e.g., "%1.")
- w:pPr: paragraph properties for level
- w:rPr: run properties for numbering text
- w:suff: suffix (tab/space/nothing)

## Overrides
- If w:lvlOverride exists:
  - Preserve overrides and map to list instance.
  - Do not modify base abstractNum.

## Restart and Continuation
- Use w:startOverride or list restart props.
- Maintain list continuation across paragraphs.

## Round-Trip Rules
- Preserve numbering IDs and ordering.
- Do not collapse identical abstractNum definitions.
