# MCSManager Client

面向 [MCSManager](https://github.com/MCSManager/MCSManager) 的 Rust CLI / TUI 客户端。

- **无参数** → 启动 TUI
- **带子命令** → 走 CLI
- 协议约定见 [`docs/CONTRACT.md`](docs/CONTRACT.md)
- WEB 功能对齐清单见 [`docs/FEATURE_PARITY.md`](docs/FEATURE_PARITY.md)

许可证：仓库根目录 [`LICENSE`](LICENSE)（GNU GPLv3）。

预编译包见 [Releases](https://github.com/CntierTeam/MCSManager-Client/releases)：`main` 每次推送更新 **Continuous** 预发布；打 `v*` tag 发布正式版。

## 要求

- Rust 1.75+（见 [`rust-toolchain.toml`](rust-toolchain.toml)）
- 可访问的 MCSManager Panel（默认端口常为 `23333`）
- 推荐使用 Panel 用户的 **API Key** 鉴权

## 构建

```bash
cargo build --release -p mcsm
./target/release/mcsm --help
```

安装到 PATH（可选）：

```bash
cargo install --path bin/mcsm
```

## 配置

配置文件默认：`~/.config/mcsm/config.toml`

```bash
mcsm config set-url http://127.0.0.1:23333
mcsm config set-key YOUR_API_KEY
mcsm config set-daemon DAEMON_UUID   # 可选：默认节点
mcsm config show
```

也可用全局参数覆盖：`--url`、`--apikey`（或环境变量 `MCSM_URL` / `MCSM_APIKEY`）。多数子命令支持 `--json` 输出机器可读结果。

## 快速示例

```bash
mcsm                          # TUI
mcsm overview --json
mcsm instance list --global
mcsm instance open <uuid> --daemon <daemonId>
mcsm terminal attach <uuid> --daemon <daemonId>
mcsm file ls <uuid> --target /
mcsm node list
mcsm auth status
```

## 主要命令

| 命令 | 说明 |
|------|------|
| `auth` | 登录 / 登出 / 状态 / API Key / 2FA |
| `overview` | 面板总览 |
| `node` | 节点 CRUD、重连、系统信息 |
| `instance` | 实例列表 / 启停 / 批量 / CRUD |
| `terminal` | 终端附着、输出日志 |
| `file` | 文件浏览、上传下载 |
| `schedule` | 计划任务 |
| `user` | 用户管理 |
| `settings` | 面板设置 |
| `audit` | 审计日志 |
| `market` | 应用市场 / 快速安装 |
| `java` | Java 运行时列表 |
| `mod` | Mod 搜索 / 列表 |
| `exchange` | 兑换码 |
| `env` | Docker 镜像 / 容器 |
| `config` | 本地客户端配置 |

完整帮助：`mcsm <command> --help`。

## Workspace 结构

| Crate | 职责 |
|-------|------|
| `mcsm-protocol` | 共享 DTO / HTTP 信封 |
| `mcsm-panel` | Panel HTTP 客户端 |
| `mcsm-daemon` | Daemon WebSocket / passport 传文件 |
| `mcsm-config` | 本地配置读写 |
| `mcsm-core` | 领域服务（CLI/TUI 唯一业务入口） |
| `mcsm-cli` | Clap 子命令 |
| `mcsm-tui` | Ratatui 界面 |
| `mcsm` | 二进制入口（`bin/mcsm`） |

硬规则：CLI / TUI **不得**直接发 HTTP/WS，一律经 `mcsm-core`；跨 crate 类型只放在 `mcsm-protocol`。

## 本地联调

若本机已有 MCSManager Panel + Daemon，可跑：

```bash
./scripts/full_local_test.sh
```

脚本会调用已构建的 `mcsm` 对常见 API 做冒烟检查（需事先配置好 URL / Key）。

## Codex Skill

项目内 skill：[`skills/mcsmanager-cli/`](skills/mcsmanager-cli/)

```bash
./scripts/install-codex-skill.sh        # 复制到 ~/.codex/skills/mcsmanager-cli
./scripts/install-codex-skill.sh link   # 软链（改仓库即生效）
```

## 架构简述

```
mcsm (bin)
├── 无 argv → mcsm-tui
└── 有子命令 → mcsm-cli
        └── mcsm-core
              ├── mcsm-panel  → Panel HTTP（优先 API Key）
              └── mcsm-daemon → 终端流 / 文件 passport
```

Panel 响应信封一般为 `{ status, data, time }`；终端与传文件走 Panel 签发的 passport 直连 Daemon。

## 相关链接

- 上游面板：[MCSManager/MCSManager](https://github.com/MCSManager/MCSManager)
- 本仓库：[CntierTeam/MCSManager-Client](https://github.com/CntierTeam/MCSManager-Client)
