# wid

`wid` is a small CLI for keeping a global "what I'm doing" log in Markdown.

## Usage

```bash
wid
wid add some text
wid add
wid done
wid done -i
wid focus -i
wid rm -i
wid now some text
wid now
```

`wid` prints the log.
`wid add some text` appends a new pending entry.
`wid add` prompts for a one-line pending entry on standard input.
`wid done` marks the active entry as done, or the last pending entry when nothing is active.
`wid done -i` lets you choose which unfinished entry to mark as done.
`wid focus -i` lets you choose which pending entry becomes the active one.
`wid rm -i` lets you choose which entry to delete after confirmation.
`wid now some text` appends a new active entry.
`wid now` prompts for a one-line summary on standard input.

## Entry states

```text
[ ] pending
[>] active
[x] done
```

`wid` keeps at most one active entry in the log.

## Storage

The default log file is:

```text
~/.local/share/wid/log.md
```
