---
name: mcsmanager-cli
description: >-
  Operate MCSManager Panel via the `mcsm` binary by running it for the user
  (TUI with no args, CLI with subcommands). Covers install from Releases,
  config/API Key, common panel workflows (nodes, instances, power, files,
  terminal, users, schedules), and TUI keys. Prefer shell execution over
  pasting recipes. Trigger on: mcsm, MCSManager-Client, MCSManager CLI/TUI,
  Panel API Key, instance open/stop, terminal attach, file ls/upload.
license: GPL-3.0-only
metadata:
  short-description: 代跑 mcsm（面板实例/文件/终端）
---

# MCSManager Client (`mcsm`)

产品：**`mcsm`** — [MCSManager](https://github.com/MCSManager/MCSManager) Panel 的 CLI/TUI 客户端。

你是 **操作员**：用户要装客户端、配面板、列节点/实例、开关机、管文件、挂终端、管用户/计划任务 → **自己在 shell 执行 `mcsm`**，不要只拼命令给用户。

本 skill 是 **execute-first**：代跑产品，不是开发 Rust crates。细节命令树见 [references/cli.md](references/cli.md)；鉴权见 [references/auth.md](references/auth.md)。

Repo: https://github.com/CntierTeam/MCSManager-Client

## Agent 硬规则

1. **执行优先**：能跑就跑。二进制：`mcsm` 或 `~/.local/bin/mcsm`；没有就先装（见下方 Install）。
2. **禁止**用「组装指令 / 操作手册 / SAMPLE / YOUR_CLI / 长篇理科说明」代替执行。短句说明 → 立刻跑 → 根据输出继续。
3. **无参数 → TUI**；**有子命令 → CLI**。Agent 代控优先 **CLI + `--json`**；用户明确要全屏 UI 再开 `mcsm`。
4. 命令名永远 **`mcsm`**，禁止 `SAMPLE` / `YOUR_CLI`。缺 URL、API Key、daemonId、实例 UUID 时只问缺的那一项，问完继续跑。
5. **不要发明 flags/endpoints**。不确定就跑 `mcsm <cmd> --help`，以输出和 [references/cli.md](references/cli.md) 为准。
6. 优先 **API Key**（`mcsm config set-key`）；session login 次要。许多 instance/file/terminal 命令需要 `--daemon <daemonId>`（或 `mcsm config set-daemon`）。
7. **Secrets**：不回显完整 API Key；优先一次性写入 config。破坏性操作（`instance kill|rm`、`file rm`、删用户）意图不清时先确认。
8. Mock/自测仅在用户明确要求或本机无面板可达时；真机意图就真跑 Panel。

## 标准代跑流

```bash
command -v mcsm || ~/.local/bin/mcsm --help
mcsm config show || true
mcsm auth status --json
mcsm overview --json
mcsm node list --json
mcsm instance list --global --json
```

未配置则先：

```bash
mcsm config set-url 'http://127.0.0.1:23333'   # 换成用户的 Panel URL
mcsm config set-key '<API_KEY>'                 # 不回显到对话摘要
mcsm config set-daemon '<DAEMON_UUID>'          # 可选默认节点
```

## 意图 → 怎么跑

| 用户意图 | 执行 |
|----------|------|
| 装 / 升级客户端 | 跑下方 install 脚本，再 `mcsm --help` |
| 配 URL / Key | `config set-url` / `set-key` / `show` / `auth status --json` |
| 看总览 / 节点 | `overview --json`；`node list\|add\|edit\|rm\|reconnect\|system` |
| 列实例 / 查详情 | `instance list --global --json`；`instance get <uuid> --daemon <id> --json` |
| 开 / 停 / 重启 / 强杀 | `instance open\|stop\|restart\|kill <uuid> --daemon <id>` |
| 发控制台命令 | `instance cmd <uuid> --daemon <id> "list"` |
| 挂终端 / 看日志 | `terminal attach\|log <uuid> --daemon <id>`（流式控制台优先 attach） |
| 文件 | `file ls\|mkdir\|touch\|rm\|upload\|download`（要 `--target` / `--local`） |
| 计划任务 | `schedule list\|add\|rm`（interval：`--time` ≥ 3 秒） |
| 用户 / 设置 / 审计 | `user …`；`settings get\|set`；`audit recent\|search --json` |
| 市场 / Java / mod / Docker | `market`；`java list`；`mod search`；`env images\|containers` |
| 要 TUI | 启动无参 `mcsm`，并告知键位 |

## Install（仅当本机没有 mcsm）

Prebuilts: [Releases](https://github.com/CntierTeam/MCSManager-Client/releases)（`main` → Continuous；`v*` → 正式版）。

```bash
curl -fsSL https://raw.githubusercontent.com/CntierTeam/MCSManager-Client/main/scripts/install.sh | bash
# Continuous / pin:
# bash -s -- --continuous
# bash -s -- --version v0.1.0
command -v mcsm && mcsm --help
```

Windows（PowerShell）：`irm …/install.ps1 | iex`（asset: **windows-amd64**）。

## Config 速查

默认：`~/.config/mcsm/config.toml`（`mcsm config path`）。

| Flag | Env | 用途 |
|------|-----|------|
| `--url` | `MCSM_URL` | Panel base URL |
| `--apikey` | `MCSM_APIKEY` | API Key |
| `--json` | — | JSON 输出 |
| `--config` | — | 另一份 config |

One-shot：`mcsm --url http://panel.example.com --apikey KEY --json overview`

## TUI 键位（用户自己玩时）

| Key | Action |
|-----|--------|
| `1`–`5` | Instances / Nodes / Overview / Users / Settings |
| `h` / `?` | Help |
| `0` | Home |
| `j`/`k` 或方向键 | Move |
| `r` | Refresh |
| Enter（Instances） | 打开终端视图 |
| `o` / `s` | Open / stop |
| `q` | Quit（非终端） |
| Esc（Terminal） | Back |

## Troubleshooting

| 症状 | 处理 |
|------|------|
| 未配置 / overview 空 | `config set-url` + `set-key` |
| 401 / 403 | 检查 Panel API Key；`auth status` |
| 缺 daemon | `--daemon <id>` 或 `config set-daemon` |
| schedule 被拒 | `--time` ≥ 3 |
| `mcsm` not found | 装二进制并把 `~/.local/bin` 加 PATH |

https://github.com/CntierTeam/MCSManager-Client
