---
name: wid
description: Use when working in Codex across one or more coding projects and you need to keep a concise shared log of what the agent is about to do, what changed, and what was finished
---

# wid

## Overview

Use `wid` to leave short, human-readable breadcrumbs in a single global log while you work.

The goal is not to produce a detailed spec or plan. The goal is to keep the human oriented during agent-heavy work by recording:

- what you are about to do
- what changed in the task
- what you finished

Keep entries and notes short. Prefer task-sized progress over step-by-step narration.

## When to Use

Use this skill when:

- you are starting work on a task in this repo or another repo
- you are resuming an existing task after a context switch
- you made a meaningful decision worth preserving
- you finished a task and should close it

Do not use this skill for:

- every tiny shell command
- long design notes or implementation plans
- verbose status diaries

## Core Workflow

1. Inspect the current log before changing it.
   ```bash
   wid --json
   ```
2. Reuse an existing task when one already matches the work.
3. If no task matches, create one with a short sentence about what you are about to do.
   ```bash
   wid now fix failing CI in md-edit @md-edit
   ```
4. As the task evolves, leave short notes for decisions, risks, or next steps.
   ```bash
   wid note --id 8f3c2d1a6b4e cache issue only reproduces on ubuntu-latest
   ```
5. When the task is finished, close it.
   ```bash
   wid done --id 8f3c2d1a6b4e
   ```

If an `--id` command fails with `item changed or not found`, fetch fresh state with `wid --json` and retry with the new transient id.

After creating a new task, usually add at least one note right away. The first note should capture one of:

- the first concrete step
- the key constraint
- why this task exists

## Keep wid Current

Treat `wid` as a current task map, not an append-only diary.

If the plan changes and the existing task or notes no longer describe the real work:

- update the task summary with `wid edit --id ...`
- update stale notes with `wid edit --id ...`
- remove obsolete notes with `wid rm --id ...`

Do not let old notes pile up when they would mislead the human reading the log later.

Prefer a small number of current, accurate notes over a long trail of outdated notes.

## Writing Rules

### Task summaries

Task summaries should describe the next meaningful unit of work.

Good:

- `add tag add and tag rm commands @wid`
- `fix flaky CI in md-edit @md-edit @ci`
- `write Codex skill for wid @wid`

Avoid:

- `run cargo test`
- `inspect file`
- `think about approach a bit more`

### Notes

Notes should be short and useful to the human returning later.

Good note types:

- decision: `use transient ids instead of storing ids in markdown`
- scope: `Codex first, other harnesses later`
- progress: `done -i now shows notes under each item`
- next step: `add tag add/rm --id next`

Avoid:

- repeating the whole task summary
- multi-paragraph explanations
- low-signal command transcripts
- notes that used to be true but no longer describe the current plan

## Tagging Rules

Each task should have a project tag when possible.

Examples:

- `@wid`
- `@md-edit`
- `@ci`
- `@agent`

Prefer one project tag plus optional helper tags.

## Safe Command Patterns

Start a new active task:

```bash
wid now write Codex skill for wid @wid
```

Add a pending task without switching focus:

```bash
wid add support tag editing by id @wid
```

Add a note to an existing task:

```bash
wid note --id 8f3c2d1a6b4e Codex first, other harnesses later
```

Edit an existing task or note:

```bash
wid edit --id 8f3c2d1a6b4e write Codex skill and install docs for wid @wid
```

Finish an existing task:

```bash
wid done --id 8f3c2d1a6b4e
```

Add text that starts with dashes by using stdin:

```bash
echo '--json should include note ids' | wid note --id 8f3c2d1a6b4e
```

## Codex Behavior

When this skill applies:

- check `wid --json` before creating a new task
- prefer continuing an existing tagged task over creating duplicates
- create a new task only when the work is genuinely new
- after `wid now`, usually add an initial note immediately
- leave a note when scope changes, a decision is made, or a useful next step becomes clear
- when the task summary or notes become stale, update or remove them instead of only appending more text
- close the task when the work is actually finished

The human should be able to open `wid` and understand the work at a glance.
