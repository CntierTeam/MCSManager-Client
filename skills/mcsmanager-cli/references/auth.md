# Auth & config (product)

## Preferred: API Key

1. In MCSManager Panel, enable an API Key for your user.
2. Save it locally:

```bash
mcsm config set-url https://panel.example.com
mcsm config set-key YOUR_API_KEY
mcsm auth status --json
```

Global one-shot (prefer config file for repeated use):

```bash
mcsm --url https://panel.example.com --apikey KEY overview --json
```

Env equivalents: `MCSM_URL`, `MCSM_APIKEY`.

## Optional: session login

```bash
mcsm auth login <username> <password>
mcsm auth login <username> <password> --code <2fa>
mcsm auth whoami --json
mcsm auth logout
```

API Key remains preferred for scripts and agents.

## Default daemon (node)

Many instance / file / terminal commands require a daemon id:

```bash
mcsm node list --json
mcsm config set-daemon <daemonId>
# or per-command:
mcsm instance open <uuid> --daemon <daemonId>
```

## Config file

- Path: `~/.config/mcsm/config.toml` (see `mcsm config path`)
- Show: `mcsm config show`
- Do not commit real keys; do not paste full keys into chat summaries.
