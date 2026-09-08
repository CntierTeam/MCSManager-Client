use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Screen};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_header(f, chunks[0], app);
    draw_body(f, chunks[1], app);
    draw_footer(f, chunks[2], app);
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " MCSManager ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:?}", app.screen),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("  "),
        Span::raw(&app.status),
    ]))
    .block(Block::default().borders(Borders::ALL).title("mcsm"));
    f.render_widget(title, area);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let help = match app.screen {
        Screen::Terminal => "Enter send via API  Esc back  (live stream: mcsm terminal attach)",
        _ => "1 Inst  2 Nodes  3 Overview  4 Users  5 Settings  h Help  r Refresh  q Quit",
    };
    let p = Paragraph::new(help).block(Block::default().borders(Borders::ALL).title("keys"));
    f.render_widget(p, area);
}

fn draw_body(f: &mut Frame, area: Rect, app: &App) {
    match app.screen {
        Screen::Home => {
            let text = format!(
                "MCSManager TUI\n\nPanel: {}\nAPI key set: {}\n\nPress 1-5 to navigate, h for help.",
                if app.config.panel_url.is_empty() {
                    "(not set)"
                } else {
                    &app.config.panel_url
                },
                !app.config.api_key.is_empty()
            );
            f.render_widget(
                Paragraph::new(text)
                    .wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title("home")),
                area,
            );
        }
        Screen::Help => {
            let text = "\
Navigation mirrors WEB sidebar:
  1 Instances — list, Enter=terminal tab, o=open, s=stop
  2 Nodes — daemon list
  3 Overview — panel overview JSON
  4 Users — user search
  5 Settings — shows local config tip

Configure:
  mcsm config set-url http://127.0.0.1:23333
  mcsm config set-key <APIKEY>
  mcsm config set-daemon <daemonId>

CLI mode: pass any subcommand, e.g. mcsm instance list --global
";
            f.render_widget(
                Paragraph::new(text)
                    .wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title("help")),
                area,
            );
        }
        Screen::Instances => draw_list(
            f,
            area,
            "instances (daemon\\tuuid\\tname)",
            &app.instances,
            app.selected,
        ),
        Screen::Nodes => draw_list(f, area, "nodes", &app.nodes, app.selected),
        Screen::Overview => {
            f.render_widget(
                Paragraph::new(app.overview_text.as_str())
                    .wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title("overview")),
                area,
            );
        }
        Screen::Users => {
            f.render_widget(
                Paragraph::new(app.users_text.as_str())
                    .wrap(Wrap { trim: false })
                    .block(Block::default().borders(Borders::ALL).title("users")),
                area,
            );
        }
        Screen::Settings => {
            let text = format!(
                "Local config\nurl={}\ndaemon={:?}\napi_key_set={}\n\nEdit via CLI: mcsm config ... / mcsm settings get",
                app.config.panel_url,
                app.config.default_daemon_id,
                !app.config.api_key.is_empty()
            );
            f.render_widget(
                Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL).title("settings")),
                area,
            );
        }
        Screen::Terminal => {
            let mut lines: Vec<Line> = app
                .term_lines
                .iter()
                .rev()
                .take(area.height.saturating_sub(4) as usize)
                .rev()
                .map(|l| Line::from(l.as_str()))
                .collect();
            lines.push(Line::from(format!("> {}_", app.input)));
            f.render_widget(
                Paragraph::new(lines)
                    .block(Block::default().borders(Borders::ALL).title("terminal")),
                area,
            );
        }
    }
}

fn draw_list(f: &mut Frame, area: Rect, title: &str, items: &[String], selected: usize) {
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(s.as_str()).style(style)
        })
        .collect();
    let list = List::new(list_items).block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(list, area);
}
