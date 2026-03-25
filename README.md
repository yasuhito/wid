# wid

`wid` is a small CLI for the agent-coding era.

When you work with coding agents, context switching happens constantly. You start one agent, wait for it to finish, jump to another project, review something else, come back, and then realize your focus has become fuzzy. You still have tasks, but you no longer have a clear answer to "what am I actually doing right now?"

`wid` exists for that moment.

It keeps a single global Markdown log of your work and gives you one explicit active item at a time. The goal is not project management in the abstract. The goal is to make it easy to return to the right thing after the next interruption.

## A Story

Imagine you are juggling several projects while agents are running in the background.

You think of a few things you need to do later, but you are not starting them yet:

```bash
wid add tighten README wording
wid add investigate failing CI on md-edit
wid add review open PR comments
```

Those become pending items:

```text
[ ] pending
```

Then you decide what you are actually starting now:

```bash
wid now investigate failing CI on md-edit
```

That item becomes the one active thing:

```text
[>] active
```

While working, you may want to leave yourself little breadcrumbs so that future-you can re-enter the task quickly:

```bash
wid note failure only happens on ubuntu-latest
wid note probably related to cache invalidation
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

And if you wrote something by mistake, remove it:

```bash
wid rm -i
```

At any time, just run:

```bash
wid
```

That prints the whole log so you can see your current flow, including notes under each item.

The same log view also appears automatically after successful state-changing commands, so you can immediately see how your focus shifted without typing `wid` again.

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
wid edit
wid edit -i
wid done
wid done -i
wid rm -i
```

- `wid` prints the log
- `wid add` adds a pending item without changing focus
- `wid now` adds a new active item and focuses it immediately
- `wid focus` focuses the latest item
- `wid focus -i` lets you choose an item to focus
- `wid note` appends a note under the active item, or the latest open item
- `wid edit` edits the active item, or the latest item
- `wid done` closes the active item, or the latest pending item
- `wid rm -i` removes an item after confirmation

## Storage

The default log file is:

```text
~/.local/share/wid/log.md
```

The log is plain Markdown, intended to stay readable without a dedicated viewer.
