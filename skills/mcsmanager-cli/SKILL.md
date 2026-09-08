---
name: mcsmanager-cli
description: >-
  Develop and operate the MCSManager-CLI Rust workspace (mcsm binary: TUI with no args, CLI with subcommands).
  Covers crate contracts (mcsm-protocol/panel/daemon/core/cli/tui), Panel HTTP envelope, Daemon passport stream/upload,
  local integration testing against MCSManager, and command usage. Trigger on: MCSManager-CLI, mcsm, MCSManager API,
  Panel/Daemon client, TUI/CLI for MCSManager, .local-mcsm, full_local_test, CONTRACT.md.
license: MIT
metadata:
  short-description: MCSManager Rust CLI/TUI workspace guide
---

# MCSManager-CLI

Rust workspace client for MCSManager Panel + Daemon. Binary name: `mcsm`.

## Hard rules

1. **No args → TUI**; **any subcommand → CLI**.
2. CLI/TUI **must not** call HTTP/WS directly — only through `mcsm-core` services.
3. Cross-crate types live only in `mcsm-protocol`. Changing a public trait/DTO requires updating `docs/CONTRACT.md`.
4. Prefer API Key auth (`X-Request-Api-Key` + `?apikey=`). Session token is secondary.
5. Code and comments in English; user-facing agent replies follow the user's language.

## Workspace map

| Crate | Role |
|-------|------|
| `mcsm-protocol` | Envelope, IDs, DTOs, stream constants |
| `mcsm-panel` | `PanelClient` / `HttpPanelClient` → `/api/*` |
| `mcsm-daemon` | Socket.IO stream + passport HTTP transfer |
| `mcsm-config` | `~/.config/mcsm/config.toml` |
| `mcsm-core` | Domain services shared by CLI/TUI |
| `mcsm-cli` | clap commands |
| `mcsm-tui` | ratatui UI |
| `bin/mcsm` | Entry |

Panel envelope: `{ "status": 200, "data": ..., "time": <ms> }`.

File passport from Panel is `{ password, addr, prefix? }` — build URLs via `FilePassport::upload_url` / `download_url` (add `http://` when missing).

## Common workflows

### Build / run

```bash
cargo build -p mcsm
./target/debug/mcsm --help
./target/debug/mcsm          # TUI
```

### Configure against a Panel

```bash
mcsm config set-url http://127.0.0.1:33333
mcsm config set-key <APIKEY>
mcsm config set-daemon <daemonId>
```

Or one-shot: `mcsm --url ... --apikey ... --json <cmd>`.

### Local full integration test

Project ships a disposable MCSManager under `.local-mcsm/` (gitignored) and:

```bash
./scripts/full_local_test.sh
```

Expect many PASS; known skips:
- **file-upload**: MCSManager 10.18.1 webpack+formidable packaging bug (`plugins/octetstream.js`)
- **env-images / env-containers**: no Docker on host

When testing schedule interval tasks, `time` must be **≥ 3** seconds.

### Extending API coverage

1. Add DTO in `mcsm-protocol` if needed.
2. Add method on `mcsm-panel::PanelClient` + `HttpPanelClient` (mirror `IdeaProjects/MCSManager/panel/src/app/routers/*`).
3. Wire `mcsm-core` service.
4. Expose clap subcommand in `mcsm-cli` and/or TUI screen.
5. Update `docs/CONTRACT.md` and `docs/FEATURE_PARITY.md`.

Truth sources for routes: MCSManager `panel/src/app/routers/*` and `frontend/src/services/apis/*`.

## Parallel / subagent work

After S0 contract freeze: change only your crate. Do not edit another crate's internals. Protocol-breaking changes must be explicit and documented.

## References

- Full crate API map: [references/contract.md](references/contract.md)
- CLI command tree: [references/cli.md](references/cli.md)
- Project docs: `docs/CONTRACT.md`, `docs/FEATURE_PARITY.md`, `README.md`
