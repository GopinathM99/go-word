---
name: debug
description: Track and debug issues. Use when the user reports a bug, mentions an issue to fix, or asks to debug something. Creates or updates a tracking file in docs/debugging/ before starting any debugging work.
argument-hint: [issue description]
---

# Debug Issue Tracker

You are debugging an issue reported by the user. Before you start any debugging work, you MUST create or update a tracking file in `docs/debugging/`.

The issue to debug: $ARGUMENTS

## Step 0: Ensure debugging directory and all-issues.md exist

Before anything else, check if the `docs/debugging/` directory exists. If it does not, create it. Then check if `docs/debugging/all-issues.md` exists. If it does not, create it as an empty file. This file is the master list of all reported issues and must always be present in the debugging folder.

## Step 1: Scan existing bug files

Use Glob to list all files in `docs/debugging/` matching `NNN-*.md` (e.g., `001-white-screen-on-launch.md`). Read each file's title (`# Issue NNN:` heading) to determine if this bug has been tracked before.

## Step 2: Determine if this is a new or existing issue

- If an existing file clearly covers the same bug, use that file — go to Step 3A.
- If no existing file matches, create a new file — go to Step 3B.

## Step 3A: Append to existing file

Do NOT modify any existing content in the file. Append a new session at the bottom using this format:

```
---

## Session: YYYY-MM-DD

### Problem Reported
<Describe the issue as the user reported it>

### Investigation
1. **<What you checked>**: <What you found>

### Fixes Attempted
1. **<Fix description>** — Result: Worked / Did not work / Partial fix

### Status: In Progress
```

## Step 3B: Create new file

Find the highest existing `NNN` number and increment by one. Create `docs/debugging/NNN-<short-slug>.md` where `<short-slug>` is a kebab-case summary (e.g., `003-cursor-not-visible.md`).

Write the file with this format:

```
# Issue NNN: <Short Title>
**First Reported:** YYYY-MM-DD

## Problem
<Describe the issue as the user reported it>

## Session: YYYY-MM-DD

### Investigation
1. **<What you checked>**: <What you found>

### Fixes Attempted
1. **<Fix description>** — Result: Worked / Did not work / Partial fix

### Status: In Progress
```

## Step 4: Debug the issue

Now proceed to actually investigate and fix the issue. As you work, come back and UPDATE the tracking file:

- Add each investigation step under `### Investigation` as numbered entries
- Add each fix attempt under `### Fixes Attempted` with the outcome
- When done, update `### Status:` to one of: `Resolved`, `Not Resolved`, or `Partial Fix`

## Step 5: Final update

When debugging is complete, make a final update to the tracking file:

- Ensure all investigation steps and fix attempts are recorded with outcomes
- Update the status line
- If resolved, add a `### Resolution` section with a brief summary of what fixed it

## Rules

- NEVER modify existing content in a bug file — only APPEND new sessions
- ALWAYS create or update the tracking file BEFORE starting any debugging
- ALWAYS update the file as you go — do not wait until the end
- Use three-digit zero-padded numbering (001, 002, 003...)
- Keep slug names short and descriptive in kebab-case
