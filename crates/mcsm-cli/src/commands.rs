use crate::output::{print_ok, print_result};
use crate::{ctx_from_cli, load_config};
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use mcsm_core::*;
use mcsm_daemon::StreamEvent;
use mcsm_protocol::auth::*;
use mcsm_protocol::files::FileListQuery;
use mcsm_protocol::ids::{DaemonId, InstanceUuid};
use mcsm_protocol::overview::AuditQuery;
use mcsm_protocol::schedule::CreateSchedule;
use mcsm_protocol::service::*;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(name = "mcsm", version, about = "MCSManager CLI/TUI")]
pub struct Cli {
    /// Panel base URL (overrides config)
    #[arg(long, global = true, env = "MCSM_URL")]
    pub url: Option<String>,

    /// API key (overrides config)
    #[arg(long, global = true, env = "MCSM_APIKEY")]
    pub apikey: Option<String>,

    /// Config file path
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// JSON output
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Authentication
    Auth {
        #[command(subcommand)]
        action: AuthCmd,
    },
    /// Panel overview
    Overview,
    /// Node (daemon) management
    Node {
        #[command(subcommand)]
        action: NodeCmd,
    },
    /// Instance management
    Instance {
        #[command(subcommand)]
        action: InstanceCmd,
    },
    /// Interactive terminal stream
    Terminal {
        #[command(subcommand)]
        action: TerminalCmd,
    },
    /// File manager
    File {
        #[command(subcommand)]
        action: FileCmd,
    },
    /// Schedules
    Schedule {
        #[command(subcommand)]
        action: ScheduleCmd,
    },
    /// Users
    User {
        #[command(subcommand)]
        action: UserCmd,
    },
    /// Panel settings
    Settings {
        #[command(subcommand)]
        action: SettingsCmd,
    },
    /// Audit logs
    Audit {
        #[command(subcommand)]
        action: AuditCmd,
    },
    /// Market / quick install
    Market {
        #[command(subcommand)]
        action: MarketCmd,
    },
    /// Java runtimes
    Java {
        #[command(subcommand)]
        action: JavaCmd,
    },
    /// Mods
    Mod {
        #[command(subcommand)]
        action: ModCmd,
    },
    /// Docker environment
    Env {
        #[command(subcommand)]
        action: EnvCmd,
    },
    /// Exchange / redeem
    Exchange {
        #[command(subcommand)]
        action: ExchangeCmd,
    },
    /// Local CLI config
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum AuthCmd {
    Login {
        username: String,
        password: String,
        #[arg(long)]
        code: Option<String>,
    },
    Logout,
    Whoami,
    Status,
    ApiKey {
        #[arg(long)]
        enable: bool,
    },
    Bind2fa,
    Confirm2fa { code: String },
}

#[derive(Debug, Clone, Subcommand)]
pub enum NodeCmd {
    List,
    Add {
        ip: String,
        port: u16,
        #[arg(long)]
        api_key: String,
        #[arg(long, default_value = "")]
        prefix: String,
        #[arg(long, default_value = "")]
        remarks: String,
    },
    Rm { uuid: String },
    Reconnect { uuid: String },
    System,
    Edit {
        uuid: String,
        #[arg(long)]
        json_body: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum InstanceCmd {
    List {
        #[arg(long)]
        daemon: Option<String>,
        #[arg(long, default_value_t = 1)]
        page: u32,
        #[arg(long, default_value_t = 20)]
        page_size: u32,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        global: bool,
    },
    Get {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Open {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Stop {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Restart {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Kill {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Cmd {
        uuid: String,
        command: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Create {
        #[arg(long)]
        daemon: Option<String>,
        #[arg(long)]
        json_body: String,
    },
    Update {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
        #[arg(long)]
        json_body: String,
    },
    Rm {
        uuids: Vec<String>,
        #[arg(long)]
        daemon: Option<String>,
        #[arg(long)]
        delete_file: bool,
    },
    Batch {
        #[arg(long)]
        action: String,
        #[arg(long)]
        json_body: String,
    },
    /// List process config files
    ConfigList {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    /// Get a process config file
    ConfigGet {
        uuid: String,
        #[arg(long)]
        file: String,
        #[arg(long)]
        daemon: Option<String>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum TerminalCmd {
    Attach {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Log {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum FileCmd {
    Ls {
        uuid: String,
        #[arg(long, default_value = "/")]
        target: String,
        #[arg(long)]
        daemon: Option<String>,
        #[arg(long, default_value_t = 1)]
        page: u32,
    },
    Mkdir {
        uuid: String,
        target: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Touch {
        uuid: String,
        target: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Rm {
        uuid: String,
        targets: Vec<String>,
        #[arg(long)]
        daemon: Option<String>,
    },
    Upload {
        uuid: String,
        local: PathBuf,
        #[arg(long, default_value = "/")]
        dir: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Download {
        uuid: String,
        remote: String,
        dest: PathBuf,
        #[arg(long)]
        daemon: Option<String>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ScheduleCmd {
    List {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Add {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
        #[arg(long)]
        name: String,
        #[arg(long)]
        time: String,
        #[arg(long)]
        action: String,
        #[arg(long, default_value = "")]
        payload: String,
        #[arg(long, default_value_t = 1)]
        type_code: i32,
        #[arg(long, default_value_t = -1)]
        count: i32,
    },
    Rm {
        uuid: String,
        name: String,
        #[arg(long)]
        daemon: Option<String>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum UserCmd {
    List {
        #[arg(long)]
        name: Option<String>,
        #[arg(long, default_value_t = 1)]
        page: u32,
    },
    Create {
        username: String,
        password: String,
        #[arg(long, default_value_t = 1)]
        permission: i32,
    },
    Rm { uuid: String },
    Update {
        #[arg(long)]
        json_body: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum SettingsCmd {
    Get,
    Set {
        #[arg(long)]
        json_body: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum AuditCmd {
    Search {
        #[arg(long)]
        keyword: Option<String>,
        #[arg(long)]
        operator: Option<String>,
        #[arg(long, default_value_t = 1)]
        page: u32,
    },
    Recent,
}

#[derive(Debug, Clone, Subcommand)]
pub enum MarketCmd {
    List,
    Install {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
        #[arg(long)]
        json_body: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum JavaCmd {
    List {
        /// Instance UUID (required by Panel java_manager permission middleware)
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ModCmd {
    List {
        uuid: String,
        #[arg(long)]
        daemon: Option<String>,
    },
    Search {
        #[arg(long)]
        json_query: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum EnvCmd {
    Images {
        #[arg(long)]
        daemon: Option<String>,
    },
    Containers {
        #[arg(long)]
        daemon: Option<String>,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ExchangeCmd {
    Call {
        #[arg(long)]
        json_body: String,
    },
    Redeem {
        #[arg(long)]
        json_body: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCmd {
    Show,
    SetUrl { url: String },
    SetKey { key: String },
    SetDaemon { id: String },
    Path,
}

pub async fn dispatch(cli: Cli) -> Result<()> {
    let json = cli.json;
    let Some(command) = cli.command.clone() else {
        bail!("internal: no command (bin should launch TUI)");
    };
    match command {
        Command::Config { action } => config_cmd(action, &cli, json).await,
        Command::Auth { action } => auth_cmd(action, &cli, json).await,
        Command::Overview => {
            let ctx = ctx_from_cli(&cli)?;
            let v = OverviewService(&ctx).get().await?;
            print_result(json, &v);
            Ok(())
        }
        Command::Node { action } => node_cmd(action, &cli, json).await,
        Command::Instance { action } => instance_cmd(action, &cli, json).await,
        Command::Terminal { action } => terminal_cmd(action, &cli, json).await,
        Command::File { action } => file_cmd(action, &cli, json).await,
        Command::Schedule { action } => schedule_cmd(action, &cli, json).await,
        Command::User { action } => user_cmd(action, &cli, json).await,
        Command::Settings { action } => settings_cmd(action, &cli, json).await,
        Command::Audit { action } => audit_cmd(action, &cli, json).await,
        Command::Market { action } => market_cmd(action, &cli, json).await,
        Command::Java { action } => java_cmd(action, &cli, json).await,
        Command::Mod { action } => mod_cmd(action, &cli, json).await,
        Command::Env { action } => env_cmd(action, &cli, json).await,
        Command::Exchange { action } => exchange_cmd(action, &cli, json).await,
    }
}

async fn config_cmd(action: ConfigCmd, cli: &Cli, json: bool) -> Result<()> {
    match action {
        ConfigCmd::Show => {
            let cfg = load_config(cli)?;
            print_result(json, &cfg);
        }
        ConfigCmd::SetUrl { url } => {
            let mut cfg = load_config(cli)?;
            cfg.panel_url = url;
            save_cfg(cli, &cfg)?;
            print_ok(json, "panel_url saved");
        }
        ConfigCmd::SetKey { key } => {
            let mut cfg = load_config(cli)?;
            cfg.api_key = key;
            save_cfg(cli, &cfg)?;
            print_ok(json, "api_key saved");
        }
        ConfigCmd::SetDaemon { id } => {
            let mut cfg = load_config(cli)?;
            cfg.default_daemon_id = Some(id);
            save_cfg(cli, &cfg)?;
            print_ok(json, "default_daemon_id saved");
        }
        ConfigCmd::Path => {
            let p = mcsm_config::config_path()?;
            print_ok(json, &p.display().to_string());
        }
    }
    Ok(())
}

fn save_cfg(cli: &Cli, cfg: &mcsm_config::Config) -> Result<()> {
    if let Some(path) = &cli.config {
        mcsm_config::save_to(path, cfg)?;
    } else {
        mcsm_config::save(cfg)?;
    }
    Ok(())
}

async fn auth_cmd(action: AuthCmd, cli: &Cli, json: bool) -> Result<()> {
    match action {
        AuthCmd::Status => {
            let cfg = load_config(cli)?;
            let ctx = mcsm_core::AppContextBuilder::new(cfg).build_unauthenticated()?;
            let v = AuthService(&ctx).status().await?;
            print_result(json, &v);
        }
        AuthCmd::Login {
            username,
            password,
            code,
        } => {
            let mut cfg = load_config(cli)?;
            let ctx = mcsm_core::AppContextBuilder::new(cfg.clone()).build_unauthenticated()?;
            let token = AuthService(&ctx)
                .login(&username, &password, code.as_deref())
                .await?;
            cfg.session_token = Some(token.clone());
            cfg.username = Some(username);
            save_cfg(cli, &cfg)?;
            print_result(json, &token);
        }
        AuthCmd::Logout => {
            let ctx = ctx_from_cli(cli)?;
            let _ = ctx.panel.logout().await;
            let mut cfg = load_config(cli)?;
            cfg.session_token = None;
            save_cfg(cli, &cfg)?;
            print_ok(json, "logged out");
        }
        AuthCmd::Whoami => {
            let ctx = ctx_from_cli(cli)?;
            let v = AuthService(&ctx).me().await?;
            print_result(json, &v);
        }
        AuthCmd::ApiKey { enable } => {
            let ctx = ctx_from_cli(cli)?;
            let v = AuthService(&ctx).set_api_key(enable).await?;
            print_result(json, &v);
        }
        AuthCmd::Bind2fa => {
            let ctx = ctx_from_cli(cli)?;
            let v = AuthService(&ctx).bind2fa().await?;
            print_result(json, &v);
        }
        AuthCmd::Confirm2fa { code } => {
            let ctx = ctx_from_cli(cli)?;
            let v = AuthService(&ctx).confirm2fa(&code).await?;
            print_result(json, &v);
        }
    }
    Ok(())
}

async fn node_cmd(action: NodeCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = NodeService(&ctx);
    match action {
        NodeCmd::List => print_result(json, &svc.list().await?),
        NodeCmd::Add {
            ip,
            port,
            api_key,
            prefix,
            remarks,
        } => {
            let req = AddNode {
                ip,
                port,
                apiKey: api_key,
                prefix,
                remarks,
                extra: Value::Null,
            };
            print_result(json, &svc.add(&req).await?);
        }
        NodeCmd::Rm { uuid } => print_result(json, &svc.delete(&DaemonId::new(uuid)).await?),
        NodeCmd::Reconnect { uuid } => {
            print_result(json, &svc.reconnect(&DaemonId::new(uuid)).await?)
        }
        NodeCmd::System => print_result(json, &svc.system().await?),
        NodeCmd::Edit { uuid, json_body } => {
            let fields: Value = serde_json::from_str(&json_body)?;
            print_result(
                json,
                &svc.edit(&DaemonId::new(uuid), &EditNode { fields })
                    .await?,
            );
        }
    }
    Ok(())
}

fn daemon_id(ctx: &mcsm_core::AppContext, d: Option<String>) -> Result<DaemonId> {
    Ok(DaemonId::new(ctx.daemon_or(d.as_deref())?))
}

async fn instance_cmd(action: InstanceCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = InstanceService(&ctx);
    match action {
        InstanceCmd::List {
            daemon,
            page,
            page_size,
            name,
            global,
        } => {
            if global {
                let q = GlobalInstanceQuery {
                    page,
                    page_size,
                    instance_name: name,
                    status: None,
                };
                print_result(json, &svc.list_global(&q).await?);
            } else {
                let d = daemon_id(&ctx, daemon)?;
                let q = InstanceListQuery {
                    daemonId: d.0,
                    page,
                    page_size,
                    instance_name: name,
                    status: None,
                    tag: None,
                };
                print_result(json, &svc.list_on_node(&q).await?);
            }
        }
        InstanceCmd::Get { uuid, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(json, &svc.get(&d, &InstanceUuid::new(uuid)).await?);
        }
        InstanceCmd::Open { uuid, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(json, &svc.open(&d, &InstanceUuid::new(uuid)).await?);
        }
        InstanceCmd::Stop { uuid, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(json, &svc.stop(&d, &InstanceUuid::new(uuid)).await?);
        }
        InstanceCmd::Restart { uuid, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(json, &svc.restart(&d, &InstanceUuid::new(uuid)).await?);
        }
        InstanceCmd::Kill { uuid, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(json, &svc.kill(&d, &InstanceUuid::new(uuid)).await?);
        }
        InstanceCmd::Cmd {
            uuid,
            command,
            daemon,
        } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(
                json,
                &svc.command(&d, &InstanceUuid::new(uuid), &command)
                    .await?,
            );
        }
        InstanceCmd::Create { daemon, json_body } => {
            let d = daemon_id(&ctx, daemon)?;
            let config: Value = serde_json::from_str(&json_body)?;
            print_result(
                json,
                &svc.create(
                    &d,
                    &mcsm_protocol::instance::CreateInstance { config },
                )
                .await?,
            );
        }
        InstanceCmd::Update {
            uuid,
            daemon,
            json_body,
        } => {
            let d = daemon_id(&ctx, daemon)?;
            let config: Value = serde_json::from_str(&json_body)?;
            print_result(
                json,
                &svc.update(
                    &d,
                    &InstanceUuid::new(uuid),
                    &mcsm_protocol::instance::UpdateInstance { config },
                )
                .await?,
            );
        }
        InstanceCmd::Rm {
            uuids,
            daemon,
            delete_file,
        } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(
                json,
                &svc.delete(
                    &d,
                    &mcsm_protocol::instance::DeleteInstances {
                        uuids,
                        deleteFile: delete_file,
                    },
                )
                .await?,
            );
        }
        InstanceCmd::Batch { action, json_body } => {
            let body: Value = serde_json::from_str(&json_body)?;
            let v = match action.as_str() {
                "open" => svc.multi_open(&body).await?,
                "stop" => svc.multi_stop(&body).await?,
                "kill" => svc.multi_kill(&body).await?,
                "restart" => svc.multi_restart(&body).await?,
                other => bail!("unknown batch action: {other}"),
            };
            print_result(json, &v);
        }
        InstanceCmd::ConfigList { uuid, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            // Panel expects body.files; empty list is valid for universal instances.
            let body = serde_json::json!({ "files": [] });
            print_result(
                json,
                &ctx
                    .panel
                    .raw(
                        "POST",
                        "/api/protected_instance/process_config/list",
                        &[
                            ("daemonId".into(), d.0.clone()),
                            ("uuid".into(), uuid),
                        ],
                        Some(&body),
                    )
                    .await?,
            );
        }
        InstanceCmd::ConfigGet { uuid, file, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(
                json,
                &ctx
                    .panel
                    .process_config_get(&d, &InstanceUuid::new(uuid), &file)
                    .await?,
            );
        }
    }
    Ok(())
}

async fn terminal_cmd(action: TerminalCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = TerminalService(&ctx);
    match action {
        TerminalCmd::Log { uuid, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            let log = svc.outputlog(&d, &InstanceUuid::new(uuid)).await?;
            if json {
                print_result(true, &log);
            } else {
                print!("{log}");
            }
        }
        TerminalCmd::Attach { uuid, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            let mut session = svc.attach(&d, &InstanceUuid::new(uuid)).await?;
            println!("Attached. Type commands; Ctrl+C or :q to quit.");
            let stdout = io::stdout();
            let mut out = stdout.lock();
            let stdin = io::stdin();
            let mut lines = stdin.lock().lines();

            loop {
                tokio::select! {
                    ev = session.next() => {
                        match ev {
                            Some(StreamEvent::Stdout(s)) => {
                                write!(out, "{s}")?;
                                out.flush()?;
                            }
                            Some(StreamEvent::Opened) => writeln!(out, "[opened]")?,
                            Some(StreamEvent::Stopped) => writeln!(out, "[stopped]")?,
                            Some(_) => {}
                            None => break,
                        }
                    }
                    line = async { lines.next() } => {
                        match line {
                            Some(Ok(l)) if l == ":q" => break,
                            Some(Ok(l)) => {
                                session.send_input(&format!("{l}\n")).await?;
                            }
                            _ => break,
                        }
                    }
                }
            }
            session.close().await;
            if json {
                print_ok(true, "detached");
            }
        }
    }
    Ok(())
}

async fn file_cmd(action: FileCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = FileService(&ctx);
    match action {
        FileCmd::Ls {
            uuid,
            target,
            daemon,
            page,
        } => {
            let d = daemon_id(&ctx, daemon)?;
            let q = FileListQuery {
                target,
                page,
                page_size: 100,
                file_name: None,
            };
            print_result(json, &svc.list(&d, &InstanceUuid::new(uuid), &q).await?);
        }
        FileCmd::Mkdir {
            uuid,
            target,
            daemon,
        } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(json, &svc.mkdir(&d, &InstanceUuid::new(uuid), &target).await?);
        }
        FileCmd::Touch {
            uuid,
            target,
            daemon,
        } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(json, &svc.touch(&d, &InstanceUuid::new(uuid), &target).await?);
        }
        FileCmd::Rm {
            uuid,
            targets,
            daemon,
        } => {
            let d = daemon_id(&ctx, daemon)?;
            let body = serde_json::json!({ "targets": targets });
            print_result(json, &svc.delete(&d, &InstanceUuid::new(uuid), &body).await?);
        }
        FileCmd::Upload {
            uuid,
            local,
            dir,
            daemon,
        } => {
            let d = daemon_id(&ctx, daemon)?;
            svc.upload(&d, &InstanceUuid::new(uuid), &dir, &local)
                .await?;
            print_ok(json, "uploaded");
        }
        FileCmd::Download {
            uuid,
            remote,
            dest,
            daemon,
        } => {
            let d = daemon_id(&ctx, daemon)?;
            svc.download(&d, &InstanceUuid::new(uuid), &remote, &dest)
                .await?;
            print_ok(json, "downloaded");
        }
    }
    Ok(())
}

async fn schedule_cmd(action: ScheduleCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = ScheduleService(&ctx);
    match action {
        ScheduleCmd::List { uuid, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(json, &svc.list(&d, &InstanceUuid::new(uuid)).await?);
        }
        ScheduleCmd::Add {
            uuid,
            daemon,
            name,
            time,
            action,
            payload,
            type_code,
            count,
        } => {
            let d = daemon_id(&ctx, daemon)?;
            let body = CreateSchedule {
                name,
                count,
                time,
                actions: vec![mcsm_protocol::schedule::ScheduleAction {
                    action_type: action,
                    payload,
                }],
                type_code,
            };
            print_result(
                json,
                &svc.create(&d, &InstanceUuid::new(uuid), &body).await?,
            );
        }
        ScheduleCmd::Rm { uuid, name, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(
                json,
                &svc.delete(&d, &InstanceUuid::new(uuid), &name).await?,
            );
        }
    }
    Ok(())
}

async fn user_cmd(action: UserCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = UserService(&ctx);
    match action {
        UserCmd::List { name, page } => {
            let q = UserSearchQuery {
                userName: name,
                page: Some(page),
                page_size: Some(20),
            };
            print_result(json, &svc.search(&q).await?);
        }
        UserCmd::Create {
            username,
            password,
            permission,
        } => {
            print_result(
                json,
                &svc.create(&CreateUserRequest {
                    username,
                    password,
                    permission,
                })
                .await?,
            );
        }
        UserCmd::Rm { uuid } => print_result(json, &svc.delete(&uuid).await?),
        UserCmd::Update { json_body } => {
            let req: EditUserRequest = serde_json::from_str(&json_body)?;
            print_result(json, &svc.edit(&req).await?);
        }
    }
    Ok(())
}

async fn settings_cmd(action: SettingsCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = SettingsService(&ctx);
    match action {
        SettingsCmd::Get => print_result(json, &svc.get().await?),
        SettingsCmd::Set { json_body } => {
            let s: mcsm_protocol::overview::PanelSetting = serde_json::from_str(&json_body)?;
            print_result(json, &svc.put(&s).await?);
        }
    }
    Ok(())
}

async fn audit_cmd(action: AuditCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = AuditService(&ctx);
    match action {
        AuditCmd::Recent => print_result(json, &svc.recent().await?),
        AuditCmd::Search {
            keyword,
            operator,
            page,
        } => {
            let q = AuditQuery {
                keyword,
                operator,
                page: Some(page),
                ..Default::default()
            };
            print_result(json, &svc.search(&q).await?);
        }
    }
    Ok(())
}

async fn market_cmd(action: MarketCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = MarketService(&ctx);
    match action {
        MarketCmd::List => print_result(json, &svc.quick_install_list().await?),
        MarketCmd::Install {
            uuid,
            daemon,
            json_body,
        } => {
            let d = daemon_id(&ctx, daemon)?;
            let body: Value = serde_json::from_str(&json_body)?;
            print_result(
                json,
                &svc.install(&d, &InstanceUuid::new(uuid), &body).await?,
            );
        }
    }
    Ok(())
}

async fn java_cmd(action: JavaCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    match action {
        JavaCmd::List { uuid, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            // Panel validates instanceId query for permission even for list.
            let v = ctx
                .panel
                .raw(
                    "GET",
                    "/api/java_manager/list",
                    &[
                        ("daemonId".into(), d.0),
                        ("instanceId".into(), uuid),
                    ],
                    None,
                )
                .await?;
            print_result(json, &v);
        }
    }
    Ok(())
}

async fn mod_cmd(action: ModCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = ModService(&ctx);
    match action {
        ModCmd::List { uuid, daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(json, &svc.list(&d, &InstanceUuid::new(uuid)).await?);
        }
        ModCmd::Search { json_query } => {
            let q: Value = serde_json::from_str(&json_query)?;
            print_result(json, &svc.search(&q).await?);
        }
    }
    Ok(())
}

async fn env_cmd(action: EnvCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = EnvService(&ctx);
    match action {
        EnvCmd::Images { daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(json, &svc.images(&d).await?);
        }
        EnvCmd::Containers { daemon } => {
            let d = daemon_id(&ctx, daemon)?;
            print_result(json, &svc.containers(&d).await?);
        }
    }
    Ok(())
}

async fn exchange_cmd(action: ExchangeCmd, cli: &Cli, json: bool) -> Result<()> {
    let ctx = ctx_from_cli(cli)?;
    let svc = ExchangeService(&ctx);
    match action {
        ExchangeCmd::Call { json_body } => {
            let body: mcsm_protocol::exchange::ExchangeRequest = serde_json::from_str(&json_body)
                .with_context(|| "parse exchange body")?;
            print_result(json, &svc.exchange(&body).await?);
        }
        ExchangeCmd::Redeem { json_body } => {
            let body: mcsm_protocol::exchange::RedeemBody = serde_json::from_str(&json_body)?;
            print_result(json, &svc.redeem(&body).await?);
        }
    }
    Ok(())
}
