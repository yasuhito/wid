# wid

`wid` is a small CLI for keeping a global "what I'm doing" log in Markdown.

## Usage

```bash
wid
wid now some text
wid now
```

`wid` prints the log.
`wid now some text` appends a new entry.
`wid now` prompts for a one-line summary on standard input.

## Storage

The default log file is:

```text
~/.local/share/wid/log.md
```
