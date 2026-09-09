---
name: mcsmanager-cli
description: >-
  Use and operate MCSManager Panel via the `mcsm` binary (TUI with no args, CLI
  with subcommands). Covers install from Releases, config/API Key, common panel
  workflows (nodes, instances, power, files, terminal, users, schedules), and
  TUI keys. Trigger on: mcsm, MCSManager-Client, MCSManager CLI/TUI, Panel API
  Key, instance open/stop, terminal attach, file ls/upload.
license: GPL-3.0-only
metadata:
  short-description: Operate MCSManager with mcsm CLI/TUI
---

# MCSManager Client (`mcsm`) — usage & operations

Operate an [MCSManager](https://github.com/MCSManager/MCSManager) Panel with the
`mcsm` CLI/TUI. Repo: https://github.com/CntierTeam/MCSManager-Client

**This skill is for using the product against a panel — not for developing the Rust crates.**
Contributors: see the repo `README.md` and `docs/`.

## When to use

Trigger when the user wants to:

- Install or upgrade `mcsm`
- Point `mcsm` at a Panel URL and API Key
- List / start / stop / restart instances, manage nodes, files, schedules
- Attach to instance console (`terminal attach` or TUI)
- Manage users, settings, market, audit, Java/mod/env helpers

## Hard rules (agent)

1. **No args → TUI**; **any subcommand → CLI**. Prefer CLI + `--json` for automation.
2. **Do not invent flags or endpoints.** Truth: `mcsm <cmd> --help` and [references/cli.md](references/cli.md).
3. Prefer **`--json`** for machine-readable output; parse that instead of guessing fields.
4. Prefer **API Key** auth (`mcsm config set-key`). Session login is secondary.
5. Many instance/file/terminal commands need **`--daemon <daemonId>`** (or a default via `mcsm config set-daemon`).
6. **Secrets:** never echo full API keys into chat logs, commit messages, or pasted history. Prefer `mcsm config set-key` once. If a one-shot `--apikey` is required, redact in summaries.
7. Destructive actions (`instance kill|rm`, `file rm`, user delete) — confirm intent when ambiguous.
8. User-facing replies follow the user's language.

## Install

Binary name: `mcsm`. Prebuilts: [Releases](https://github.com/CntierTeam/MCSManager-Client/releases)
(`main` pushes update **Continuous** pre-release; `v*` tags = stable).

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/CntierTeam/MCSManager-Client/main/scripts/install.sh | bash
```

Continuous / pin:

```bash
curl -fsSL https://raw.githubusercontent.com/CntierTeam/MCSManager-Client/main/scripts/install.sh | bash -s -- --continuous
curl -fsSL https://raw.githubusercontent.com/CntierTeam/MCSManager-Client/main/scripts/install.sh | bash -s -- --version v0.1.0
PREFIX=/usr/local curl -fsSL https://raw.githubusercontent.com/CntierTeam/MCSManager-Client/main/scripts/install.sh | bash
```

Ensure `~/.local/bin` (or `$PREFIX/bin`) is on `PATH`, then:

```bash
command -v mcsm && mcsm --help
```

### Windows (PowerShell)

Default: `%LOCALAPPDATA%\Programs\mcsm` (+ user PATH):

```powershell
irm https://raw.githubusercontent.com/CntierTeam/MCSManager-Client/main/scripts/install.ps1 | iex
```

Continuous / pin:

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/CntierTeam/MCSManager-Client/main/scripts/install.ps1))) -Continuous
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/CntierTeam/MCSManager-Client/main/scripts/install.ps1))) -Version v0.1.0
```

Asset: **windows-amd64**.

## Config

Default file: `~/.config/mcsm/config.toml`

```bash
mcsm config set-url http://127.0.0.1:23333
mcsm config set-key YOUR_API_KEY
mcsm config set-daemon DAEMON_UUID   # optional default node
mcsm config show
mcsm config path
```

Overrides:

| Flag | Env | Used for |
|------|-----|----------|
| `--url` | `MCSM_URL` | Panel base URL |
| `--apikey` | `MCSM_APIKEY` | Panel API Key |
| `--json` | — | JSON output |
| `--config` | — | Alternate config path |

One-shot:

```bash
mcsm --url http://panel.example.com --apikey KEY --json overview
```

Auth details: [references/auth.md](references/auth.md).

## Common workflows

### Check panel / overview

```bash
mcsm auth status --json
mcsm overview --json
mcsm node list --json
```

### Instances & power

```bash
mcsm instance list --global --json
mcsm instance get <uuid> --daemon <daemonId> --json
mcsm instance open <uuid> --daemon <daemonId>
mcsm instance stop <uuid> --daemon <daemonId>
mcsm instance restart <uuid> --daemon <daemonId>
mcsm instance kill <uuid> --daemon <daemonId>
mcsm instance cmd <uuid> --daemon <daemonId> "list"
```

### Console

```bash
mcsm terminal attach <uuid> --daemon <daemonId>
mcsm terminal log <uuid> --daemon <daemonId>
```

### Files

```bash
mcsm file ls <uuid> --target /
mcsm file mkdir <uuid> --target /plugins
mcsm file upload <uuid> --target /plugins --local ./plugin.jar
mcsm file download <uuid> --target /server.properties --local ./server.properties
mcsm file rm <uuid> --target /old.txt
```

### Schedules

```bash
mcsm schedule list <uuid> --daemon <daemonId>
# interval tasks: --time must be ≥ 3 (seconds)
mcsm schedule add <uuid> --name t --time 5 --action command --payload list --type-code 1 --count 1
mcsm schedule rm <uuid> --name t
```

### Users / settings / audit

```bash
mcsm user list --json
mcsm settings get --json
mcsm audit recent --json
```

### Market / Java / mods / Docker env

```bash
mcsm market list --json
mcsm java list <uuid> --daemon <daemonId>
mcsm mod search <query>
mcsm env images --daemon <daemonId>
mcsm env containers --daemon <daemonId>
```

## TUI

```bash
mcsm
```

| Key | Action |
|-----|--------|
| `1` | Instances |
| `2` | Nodes |
| `3` | Overview |
| `4` | Users |
| `5` | Settings |
| `h` / `?` | Help |
| `0` | Home |
| `j` / `k` or arrows | Move |
| `r` | Refresh |
| Enter (Instances) | Open terminal view |
| `o` / `s` (Instances) | Open / stop |
| `q` | Quit (not in terminal) |
| Esc (Terminal) | Back |

Live streaming console is more reliable via `mcsm terminal attach`.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| Not configured / empty overview | Missing URL or key | `mcsm config set-url` + `set-key` |
| 401 / 403 | Bad or disabled API Key | Panel → user API Key; `mcsm auth status` |
| Commands need daemon | No default node | `--daemon <id>` or `mcsm config set-daemon` |
| Schedule add rejected | Interval `< 3` | Use `--time` ≥ 3 |
| `mcsm` not found | PATH | Add `~/.local/bin` or Windows install dir |

## References

- Command tree & examples: [references/cli.md](references/cli.md)
- Auth / config: [references/auth.md](references/auth.md)
- Upstream panel: https://github.com/MCSManager/MCSManager
- This client: https://github.com/CntierTeam/MCSManager-Client
