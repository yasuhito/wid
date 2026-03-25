# wid for Codex

`wid` gives Codex a simple way to leave short, useful progress breadcrumbs for humans.

The focus is not heavy process documentation. The focus is a concise shared log of:

- what the agent is about to do
- what changed
- what was finished

## Quick Install

Tell Codex:

```text
Fetch and follow instructions from https://raw.githubusercontent.com/yasuhito/wid/refs/heads/master/.codex/INSTALL.md
```

## What the Skill Teaches

The Codex skill teaches a small workflow:

1. Read current state with `wid --json`
2. Reuse an existing task if possible
3. Otherwise create a new task with `wid now ...`
4. Leave short notes with `wid note --id ...`
5. Close finished work with `wid done --id ...`

It also teaches the writing style:

- short task summaries
- short notes
- project tags like `@wid` or `@md-edit`
- no long specs or diary-like logs inside `wid`

## Example

```bash
wid now write Codex skill for wid @wid
wid note keep the skill focused on concise task-sized updates
wid done
```
