# AGENTS.md

## Purpose

`wid` exists to help humans keep their focus while working with coding agents across multiple projects.

It is a lightweight shared log, not a full project-management system.

The core question `wid` should answer is:

> What am I doing right now, and what should I come back to after the next interruption?

## What wid should optimize for

- short, human-readable task summaries
- short notes that preserve decisions, progress, and next steps
- one clear active task at a time
- smooth use from both humans and AI agents
- plain Markdown that can still be read and edited directly

## What wid should not become

`wid` should not grow into a conventional task manager.

Avoid adding features whose main value is project-management completeness rather than focus recovery.

## Explicit non-goals

The following are out of scope unless the project direction changes significantly:

- nested tasks
- parent/child task trees
- task dependencies
- dependency graphs
- milestones
- estimates
- due dates
- assignees
- priorities as a large formal system
- dashboards for project-management reporting

## Design bias

When there is a choice between:

- a simpler feature that helps a human quickly recover focus
- a more powerful feature that makes `wid` feel like a general task manager

prefer the simpler feature.

If a proposed feature increases cognitive load, adds workflow ceremony, or makes the log harder to scan, it is probably the wrong direction for `wid`.
