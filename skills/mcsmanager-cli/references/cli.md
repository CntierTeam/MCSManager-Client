# CLI command tree

```text
mcsm auth login|logout|whoami|status|apikey|bind2fa|confirm2fa
mcsm overview
mcsm node list|add|edit|rm|reconnect|system
mcsm instance list|get|create|update|rm|open|stop|restart|kill|cmd|batch|config-list|config-get
mcsm terminal attach|log
mcsm file ls|mkdir|touch|rm|upload|download
mcsm schedule list|add|rm
mcsm user list|create|rm|update
mcsm settings get|set
mcsm audit search|recent
mcsm market list|install
mcsm java list <uuid>
mcsm mod list|search
mcsm env images|containers
mcsm exchange call|redeem
mcsm config show|set-url|set-key|set-daemon|path
```

Global flags: `--url`, `--apikey`, `--json`, `--config`.

Env: `MCSM_URL`, `MCSM_APIKEY`.

## Examples

```bash
mcsm --url http://127.0.0.1:23333 --apikey KEY overview --json
mcsm instance list --global --json
mcsm instance open <uuid> --daemon <daemonId>
mcsm terminal attach <uuid> --daemon <daemonId>
mcsm file ls <uuid> --target /
mcsm schedule add <uuid> --name t --time 5 --action command --payload list --type-code 1 --count 1
```

Schedule interval (`type-code` / type `1`): `time` must be **≥ 3** seconds.

When unsure of flags: `mcsm <command> --help` — do not invent options.
