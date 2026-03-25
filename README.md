# wid

`wid` is a small CLI for the agent-coding era.

When you work with coding agents, context switching happens constantly. You start one agent, wait for it to finish, jump to another project, review something else, come back, and then realize your focus has become fuzzy. You still have tasks, but you no longer have a clear answer to "what am I actually doing right now?"

`wid` exists for that moment.

It keeps a single global Markdown log of your work and gives you one explicit active item at a time. The goal is not project management in the abstract. The goal is to make it easy to return to the right thing after the next interruption.

## A Story

Imagine you are juggling several projects while agents are running in the background.

You think of a few things you need to do later, but you are not starting them yet:

```bash
wid add tighten README wording @wid
wid add investigate failing CI on md-edit @md-edit
wid add review open PR comments
```

Those become pending items:

```text
[ ] pending
```

Then you decide what you are actually starting now:

```bash
wid now investigate failing CI on md-edit @md-edit @ci
```

That item becomes the one active thing:

```text
[>] active
```

While working, you may want to leave yourself little breadcrumbs so that future-you can re-enter the task quickly:

```bash
wid note failure only happens on ubuntu-latest
wid note probably related to cache invalidation
wid note -i
```

Later, an agent finishes another task and you need to switch. You can jump back to the latest item:

```bash
wid focus
```

Or choose explicitly from the list:

```bash
wid focus -i
```

If the wording of an item is no longer right, you can refine it instead of adding noise:

```bash
wid edit
wid edit -i
```

When the task is done, close it:

```bash
wid done
wid done -i
```

Done items become:

```text
[x] done
```

`wid done -i` opens a picker for all items. Use `Space` to toggle `[ ]` and `[x]`, then `Enter` to confirm.

When too many completed items start to make the main log noisy, move them out of the way:

```bash
wid archive
wid archive --yes
```

`wid archive` asks for confirmation before moving all done items into a separate archive log. Use `wid archive --yes` from scripts or agents when you want to skip the prompt and keep your main log focused on the work that is still alive.

And if you wrote something by mistake, remove it:

```bash
wid rm -i
```

If you need to make a manual fix, you can also open the raw Markdown directly in your editor:

```bash
wid open
wid open --archive
```

At any time, just run:

```bash
wid
```

That prints the whole log so you can see your current flow, including `📝` notes under each item.

The same log view also appears automatically after successful state-changing commands, so you can immediately see how your focus shifted without typing `wid` again.

If you are exploring the CLI from an agent or a script, start with:

```bash
wid --help
```

Then drill down with command-specific help such as `wid add --help` or `wid done --help`. Each help page includes an `Examples:` section so you can learn the command progressively instead of memorizing everything up front.

If an agent needs to inspect the log programmatically, it can use:

```bash
wid --json
```

That prints the same log as structured JSON, with a transient per-entry `id` that agents can use immediately in follow-up commands. The `id` is derived from the current entry contents and is not stored in the Markdown file.

For example, an agent can read the current list with `wid --json`, choose an item by its transient `id`, and then close it with:

```bash
wid done --id 8f3c2d1a6b4e
```

Agents can also add notes and remove either whole items or individual notes by transient id:

```bash
wid note --id 8f3c2d1a6b4e "waiting for CI to finish"
wid rm --id note_4a1d9c2e7f55
wid tag add --id 8f3c2d1a6b4e @wid @agent
```

If an agent wants to avoid shell quoting entirely, it can stream text through standard input:

```bash
echo '--id support for agent workflows' | wid now
echo '--json shape' | wid note --id 8f3c2d1a6b4e
```

## Build and Install

`wid` is a standard Rust CLI project.

Build it locally:

```bash
cargo build
```

Run it without installing:

```bash
cargo run --
cargo run -- now investigate failing CI on md-edit
```

Install it into your local Cargo bin directory:

```bash
cargo install --path .
```

That places the binary in:

```text
~/.cargo/bin/wid
```

If `~/.cargo/bin` is in your `PATH`, you can then run `wid` directly.

## Codex Skill

`wid` also ships with a Codex skill so agents can keep the log updated in a concise, human-readable way.

See:

- [Codex install guide](/home/yasuhito/Work/wid/.codex/INSTALL.md)
- [Codex usage guide](/home/yasuhito/Work/wid/docs/README.codex.md)

## Why This Helps

`wid` is built around a simple idea: your attention should have a visible home.

In agent-heavy workflows, the problem is often not "I forgot to write down tasks." The problem is "I stopped knowing which task currently owns my mind." `wid` makes that explicit with exactly one active item across the entire log.

That is why the model is small:

```text
[ ] pending
[>] active
[x] done
```

The log stays readable as plain Markdown, but the CLI gives you enough structure to keep your focus from dissolving when projects and agents pile up.

You can also use trailing `@tags` to keep track of which project or workstream an item belongs to:

```bash
wid add improve JSON examples @wid @docs
wid now fix flaky CI on md-edit @md-edit @ci
```

`wid` treats those as tags in JSON output, so agents can use `@wid` or `@md-edit` as lightweight project markers without needing extra CLI flags.

## Quick Reference

```bash
wid
wid add TEXT...
wid add
wid now TEXT...
wid now
wid focus
wid focus -i
wid note TEXT...
wid note
wid note --id ID TEXT...
wid edit
wid edit --id ID TEXT...
wid edit -i
wid tag add --id ID @tag...
wid tag rm --id ID @tag...
wid open
wid open --archive
wid done
wid done --id ID
wid done -i
wid archive
wid rm --id ID
wid rm -i
```

- `wid` prints the log
- `wid --json` prints the log as structured JSON for agents or scripts
- `wid add` adds a pending item without changing focus
- `wid now` adds a new active item and focuses it immediately
- `wid focus` focuses the latest item
- `wid focus -i` lets you choose an item to focus
- `wid note` appends a note under the active item, or the latest open item
- `wid note --id` appends a note to a specific item by transient id from `wid --json`
- `wid edit` edits the active item, or the latest item
- `wid edit --id` edits a specific item or note by transient id from `wid --json`
- `wid edit -i` edits either an item or a `📝` note from the inline picker
- `wid tag add` adds one or more `@tags` to a specific item by transient id
- `wid tag rm` removes one or more `@tags` from a specific item by transient id
- `wid open` opens `log.md` in `$EDITOR`
- `wid open --archive` opens `archive.md` in `$EDITOR`
- `wid done` closes the active item, or the latest pending item
- `wid done --id` closes a specific item by transient id from `wid --json`
- `wid done -i` toggles done state for multiple items before confirmation
- `wid archive` moves all done items into `archive.md`
- `wid rm --id` removes a specific item or note by transient id from `wid --json`
- `wid rm -i` removes an item or note after confirmation

## Storage

The default log file is:

```text
~/.local/share/wid/log.md
```

Archived done items are stored in:

```text
~/.local/share/wid/archive.md
```

The log is plain Markdown, intended to stay readable without a dedicated viewer.
