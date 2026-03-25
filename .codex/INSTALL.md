# Installing the wid Skill for Codex

Enable the `wid` skill in Codex through native skill discovery.

## Prerequisites

- Git
- Codex with native skill discovery enabled
- `wid` installed and available in `PATH`

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yasuhito/wid.git ~/.codex/wid
   ```

2. Create the skills symlink:
   ```bash
   mkdir -p ~/.agents/skills
   ln -s ~/.codex/wid/skills ~/.agents/skills/wid
   ```

3. Restart Codex so it discovers the skill.

## Verify

```bash
ls -la ~/.agents/skills/wid
```

You should see a symlink pointing at `~/.codex/wid/skills`.

Then ask Codex to open the skill:

```text
Use the wid skill before continuing.
```

## Update

```bash
cd ~/.codex/wid && git pull
```

The updated skill will be picked up after restarting Codex.
