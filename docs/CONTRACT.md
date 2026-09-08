# MCSManager-CLI Contract

Frozen public API between workspace crates. Subagents may only change their own crate internals; cross-crate type changes go through `mcsm-protocol` + this document.

## Binary behavior

- Binary: `mcsm`
- **No argv → TUI** (`mcsm-tui`)
- **Any subcommand/args → CLI** (`mcsm-cli` / clap)

## Auth & transport

| Mode | How |
|------|-----|
| API Key (preferred) | Header `X-Request-Api-Key` + query `apikey=` |
| Session | Cookie jar + query `token=` + `X-Requested-With: XMLHttpRequest` |
| Stream / upload / download | Panel issues passport → client talks to Daemon HTTP/WS |

Panel envelope: `{ "status": 200, "data": ..., "time": <ms> }` (`mcsm_protocol::ApiEnvelope`).

## Crate graph

```
bin/mcsm → mcsm-cli + mcsm-tui
mcsm-cli / mcsm-tui → mcsm-core
mcsm-core → mcsm-panel + mcsm-daemon + mcsm-config
mcsm-panel / mcsm-daemon / mcsm-config → mcsm-protocol
```

**Rule:** CLI/TUI must not call HTTP/WS directly; only via `mcsm-core` services.

## `mcsm-protocol`

- `ApiEnvelope<T>`, `McsmError`, `DaemonId`, `InstanceUuid`, `UserUuid`
- Domain modules: `auth`, `overview`, `service`, `instance`, `files`, `schedule`, `environment`, `java`, `mod_mgr`, `exchange`, `daemon`, `stream`
- Stream event constants in `stream` module

## `mcsm-panel::PanelClient`

Full method list mirrors Panel routers under `/api/*` (auth, overview, service, instance, protected_instance, files, protected_schedule, environment, java_manager, mod, exchange). Escape hatch: `raw(method, path, query, body)`.

Implementation: `HttpPanelClient`.

## `mcsm-daemon`

- `DaemonClient::connect_with_key` — ops/debug direct Socket.IO + `auth` event
- `DaemonClient::connect_stream` — passport stream session
- `upload` / `download` — passport HTTP
- Transport: Engine.IO v4 + Socket.IO v4 over `tokio-tungstenite` (see `socketio.rs`)

## `mcsm-core` services

`AuthService`, `OverviewService`, `NodeService`, `InstanceService`, `TerminalService`, `FileService`, `ScheduleService`, `UserService`, `SettingsService`, `AuditService`, `MarketService`, `JavaService`, `ModService`, `EnvService`, `ExchangeService`

Shared: `AppContext` / `AppContextBuilder`.

## CLI tree

```
mcsm auth|overview|node|instance|terminal|file|schedule|user|settings|audit|market|java|mod|env|exchange|config
```

Global: `--url`, `--apikey`, `--json`, `--config`.

## Out of scope (by design)

- WEB layout designer / decorative cards
- Disabled Panel `socket_router`
- Shop iframe (HTTP exchange API only)

## Source of truth in MCSManager

- `panel/src/app/routers/*`
- `frontend/src/services/apis/*`
