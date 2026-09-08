# Contract summary (load when changing crate boundaries)

## Binary

- `mcsm` with no argv → TUI
- `mcsm <subcommand>` → CLI

## Auth

| Mode | How |
|------|-----|
| API Key | Header `X-Request-Api-Key` + query `apikey=` |
| Session | Cookie + query `token=` + `X-Requested-With: XMLHttpRequest` |
| Stream / files | Panel passport → Daemon WS/HTTP |

## Graph

```
bin/mcsm → mcsm-cli + mcsm-tui
mcsm-cli / mcsm-tui → mcsm-core
mcsm-core → mcsm-panel + mcsm-daemon + mcsm-config
* → mcsm-protocol
```

## Envelope

```json
{ "status": 200, "data": {}, "time": 0 }
```

`status != 200` → `McsmError::Api`.

## File passport

Panel returns `{ "password", "addr", "remoteMappings"? }`.

- Upload: `http://{addr}{prefix}/upload/{password}` (multipart field `file`)
- Download: `http://{addr}{prefix}/download/{password}/{fileName}`

Use `mcsm_protocol::stream::FilePassport` helpers — do not treat bare `localhost:34444` as a URL.

## Out of scope

- WEB layout designer / decorative cards
- Disabled Panel `socket_router`
- Shop iframe (exchange HTTP only)

Canonical file in repo: `docs/CONTRACT.md`.
