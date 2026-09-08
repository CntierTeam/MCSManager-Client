use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use mcsm_config::Config;
use mcsm_core::{
    AppContext, InstanceService, NodeService, OverviewService, TerminalService, UserService,
};
use mcsm_daemon::StreamEvent;
use mcsm_protocol::ids::{DaemonId, InstanceUuid};
use mcsm_protocol::service::{GlobalInstanceQuery, InstanceListQuery};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::Duration;

use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Instances,
    Nodes,
    Overview,
    Users,
    Settings,
    Terminal,
    Help,
}

pub struct App {
    pub screen: Screen,
    pub ctx: Option<AppContext>,
    pub config: Config,
    pub status: String,
    pub instances: Vec<String>,
    pub nodes: Vec<String>,
    pub overview_text: String,
    pub users_text: String,
    pub selected: usize,
    pub input: String,
    pub term_lines: Vec<String>,
    pub term_uuid: Option<String>,
    pub term_daemon: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(ctx: Option<AppContext>, config: Config) -> Self {
        let status = if ctx.is_some() {
            format!("Connected: {}", config.panel_url)
        } else if config.panel_url.is_empty() {
            "Not configured. Press h for help. Set URL via `mcsm config set-url`.".into()
        } else {
            format!("Config present but auth failed for {}", config.panel_url)
        };
        Self {
            screen: Screen::Home,
            ctx,
            config,
            status,
            instances: Vec::new(),
            nodes: Vec::new(),
            overview_text: String::new(),
            users_text: String::new(),
            selected: 0,
            input: String::new(),
            term_lines: Vec::new(),
            term_uuid: None,
            term_daemon: None,
            should_quit: false,
        }
    }
}

pub async fn run_app(ctx: Option<AppContext>, config: Config) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(ctx, config);
    let res = event_loop(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

async fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    refresh_current(app).await;

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') if app.screen != Screen::Terminal => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('1') => {
                        app.screen = Screen::Instances;
                        refresh_current(app).await;
                    }
                    KeyCode::Char('2') => {
                        app.screen = Screen::Nodes;
                        refresh_current(app).await;
                    }
                    KeyCode::Char('3') => {
                        app.screen = Screen::Overview;
                        refresh_current(app).await;
                    }
                    KeyCode::Char('4') => {
                        app.screen = Screen::Users;
                        refresh_current(app).await;
                    }
                    KeyCode::Char('5') => app.screen = Screen::Settings,
                    KeyCode::Char('h') | KeyCode::Char('?') => app.screen = Screen::Help,
                    KeyCode::Char('0') => app.screen = Screen::Home,
                    KeyCode::Down | KeyCode::Char('j') => {
                        let len = list_len(app);
                        if len > 0 {
                            app.selected = (app.selected + 1).min(len - 1);
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.selected > 0 {
                            app.selected -= 1;
                        }
                    }
                    KeyCode::Char('r') => refresh_current(app).await,
                    KeyCode::Enter if app.screen == Screen::Instances => {
                        if let Some(line) = app.instances.get(app.selected).cloned() {
                            // format: daemonId\tuuid\tname
                            let parts: Vec<_> = line.split('\t').collect();
                            if parts.len() >= 2 {
                                app.term_daemon = Some(parts[0].to_string());
                                app.term_uuid = Some(parts[1].to_string());
                                app.screen = Screen::Terminal;
                                app.term_lines.clear();
                                app.status = format!("Terminal {}", parts[1]);
                                attach_and_drain(app).await;
                            }
                        }
                    }
                    KeyCode::Char('o') if app.screen == Screen::Instances => {
                        instance_action(app, "open").await;
                    }
                    KeyCode::Char('s') if app.screen == Screen::Instances => {
                        instance_action(app, "stop").await;
                    }
                    KeyCode::Esc if app.screen == Screen::Terminal => {
                        app.screen = Screen::Instances;
                    }
                    KeyCode::Char(c) if app.screen == Screen::Terminal => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace if app.screen == Screen::Terminal => {
                        app.input.pop();
                    }
                    KeyCode::Enter if app.screen == Screen::Terminal => {
                        let cmd = std::mem::take(&mut app.input);
                        send_term_command(app, &cmd).await;
                    }
                    _ => {}
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn list_len(app: &App) -> usize {
    match app.screen {
        Screen::Instances => app.instances.len(),
        Screen::Nodes => app.nodes.len(),
        _ => 0,
    }
}

async fn refresh_current(app: &mut App) {
    let Some(ctx) = app.ctx.clone() else {
        app.status = "No authenticated context".into();
        return;
    };
    match app.screen {
        Screen::Instances => match load_instances(&ctx, &app.config).await {
            Ok(list) => {
                app.instances = list;
                app.status = format!("{} instances", app.instances.len());
            }
            Err(e) => app.status = format!("instances error: {e}"),
        },
        Screen::Nodes => match NodeService(&ctx).list().await {
            Ok(nodes) => {
                app.nodes = nodes
                    .into_iter()
                    .map(|n| {
                        format!(
                            "{}\t{}:{}\t{}\t{}",
                            n.uuid,
                            n.ip,
                            n.port,
                            n.remarks,
                            if n.available { "online" } else { "offline" }
                        )
                    })
                    .collect();
                app.status = format!("{} nodes", app.nodes.len());
            }
            Err(e) => app.status = format!("nodes error: {e}"),
        },
        Screen::Overview => match OverviewService(&ctx).get().await {
            Ok(o) => {
                app.overview_text =
                    serde_json::to_string_pretty(&o).unwrap_or_else(|_| format!("{o:?}"));
                app.status = "overview loaded".into();
            }
            Err(e) => app.status = format!("overview error: {e}"),
        },
        Screen::Users => {
            let q = mcsm_protocol::auth::UserSearchQuery {
                userName: None,
                page: Some(1),
                page_size: Some(50),
            };
            match UserService(&ctx).search(&q).await {
                Ok(page) => {
                    app.users_text =
                        serde_json::to_string_pretty(&page).unwrap_or_else(|_| "{}".into());
                    app.status = "users loaded".into();
                }
                Err(e) => app.status = format!("users error: {e}"),
            }
        }
        _ => {}
    }
}

async fn load_instances(ctx: &AppContext, cfg: &Config) -> anyhow::Result<Vec<String>> {
    let svc = InstanceService(ctx);
    if let Some(daemon) = &cfg.default_daemon_id {
        let q = InstanceListQuery {
            daemonId: daemon.clone(),
            page: 1,
            page_size: 100,
            instance_name: None,
            status: None,
            tag: None,
        };
        let page = svc.list_on_node(&q).await?;
        Ok(page
            .data
            .into_iter()
            .map(|inst| {
                let uuid = inst
                    .get("instanceUuid")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let name = inst
                    .get("config")
                    .and_then(|c| c.get("nickname"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                format!("{daemon}\t{uuid}\t{name}")
            })
            .collect())
    } else {
        let q = GlobalInstanceQuery {
            page: 1,
            page_size: 50,
            instance_name: None,
            status: None,
        };
        let raw = svc.list_global(&q).await?;
        let mut out = Vec::new();
        if let Some(map) = raw.as_object() {
            for (daemon, entry) in map {
                if let Some(arr) = entry.get("instances").and_then(|v| v.as_array()) {
                    for inst in arr {
                        let uuid = inst
                            .get("instanceUuid")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        let name = inst
                            .get("config")
                            .and_then(|c| c.get("nickname"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        out.push(format!("{daemon}\t{uuid}\t{name}"));
                    }
                }
            }
        }
        Ok(out)
    }
}

async fn instance_action(app: &mut App, action: &str) {
    let Some(ctx) = app.ctx.clone() else { return };
    let Some(line) = app.instances.get(app.selected).cloned() else {
        return;
    };
    let parts: Vec<_> = line.split('\t').collect();
    if parts.len() < 2 {
        return;
    }
    let d = DaemonId::new(parts[0]);
    let u = InstanceUuid::new(parts[1]);
    let svc = InstanceService(&ctx);
    let res = match action {
        "open" => svc.open(&d, &u).await,
        "stop" => svc.stop(&d, &u).await,
        "restart" => svc.restart(&d, &u).await,
        "kill" => svc.kill(&d, &u).await,
        _ => return,
    };
    app.status = match res {
        Ok(_) => format!("{action} ok"),
        Err(e) => format!("{action} failed: {e}"),
    };
}

async fn attach_and_drain(app: &mut App) {
    let Some(ctx) = app.ctx.clone() else { return };
    let (Some(uuid), Some(daemon)) = (app.term_uuid.clone(), app.term_daemon.clone()) else {
        return;
    };
    match TerminalService(&ctx)
        .attach(&DaemonId::new(daemon), &InstanceUuid::new(uuid))
        .await
    {
        Ok(mut session) => {
            app.term_lines.push("connected".into());
            // Drain a few initial events non-blocking-ish
            for _ in 0..20 {
                match tokio::time::timeout(Duration::from_millis(50), session.next()).await {
                    Ok(Some(StreamEvent::Stdout(s))) => app.term_lines.push(s),
                    Ok(Some(_)) => {}
                    _ => break,
                }
            }
            // Keep session open is complex in sync UI loop; close for now after snapshot.
            // Full interactive stream is available via `mcsm terminal attach`.
            session.close().await;
            app.term_lines
                .push("snapshot done; use CLI `mcsm terminal attach` for live REPL".into());
        }
        Err(e) => app.term_lines.push(format!("attach failed: {e}")),
    }
}

async fn send_term_command(app: &mut App, cmd: &str) {
    let Some(ctx) = app.ctx.clone() else { return };
    let (Some(uuid), Some(daemon)) = (app.term_uuid.clone(), app.term_daemon.clone()) else {
        return;
    };
    match InstanceService(&ctx)
        .command(&DaemonId::new(daemon), &InstanceUuid::new(uuid), cmd)
        .await
    {
        Ok(_) => app.term_lines.push(format!("> {cmd}")),
        Err(e) => app.term_lines.push(format!("cmd error: {e}")),
    }
}
