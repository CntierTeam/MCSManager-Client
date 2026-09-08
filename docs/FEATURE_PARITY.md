# WEB feature parity checklist

Mapped from MCSManager WEB → `mcsm` CLI/TUI.

| WEB area | CLI | TUI | Status |
|----------|-----|-----|--------|
| Install / login / logout / status | `auth *` | Home status | done |
| API Key / 2FA | `auth apikey`, `bind2fa`, `confirm2fa` | via CLI | done |
| Overview | `overview` | screen 3 | done |
| Nodes CRUD / reconnect / system | `node *` | screen 2 | done |
| Docker images/containers | `env images\|containers` | — | done (CLI) |
| Instance list / get / CRUD | `instance *` | screen 1 | done |
| open/stop/restart/kill/cmd | `instance open\|stop\|...` | o/s keys | done |
| Batch multi_* | `instance batch` | — | done |
| Terminal stream | `terminal attach` | snapshot + cmd | done |
| Output log | `terminal log` | — | done |
| Files ls/mkdir/touch/rm/upload/download | `file *` | — | done |
| Schedule | `schedule *` | — | done |
| Users | `user *` | screen 4 | done |
| Settings | `settings get\|set` | screen 5 tip | done |
| Audit | `audit *` | — | done |
| Market / quick install | `market *` | — | done |
| Java | `java list` | — | done |
| Mod list/search | `mod *` | — | done |
| Exchange / redeem | `exchange *` | — | done |
| Config local | `config *` | — | done |
| Layout designer / decorative cards | — | — | out of scope |
| Shop iframe | HTTP only via exchange | — | out of scope |
| SSO browser flow | `raw` / sso_config | — | partial (config read) |

Process config / async tasks / install_instance: available via `mcsm-panel` trait + `MarketService::install` / panel methods; CLI can extend with `raw` if needed.

Daemon direct key connect: `mcsm_daemon::DefaultDaemonClient::connect_with_key`.
