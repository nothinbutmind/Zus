mod error;
mod types;

use std::{env, io, process::Command, time::Duration};

use crate::error::{AppError, AppResult};
use crate::types::{ActionForm, ActionKind, App, AppTerminal, CommandResult, Focus};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
};

const LOGO: &[&str] = &[
    "███████╗██╗   ██╗███████╗",
    "╚══███╔╝██║   ██║██╔════╝",
    "  ███╔╝ ██║   ██║███████╗",
    " ███╔╝  ██║   ██║╚════██║",
    "███████╗╚██████╔╝███████║",
    "╚══════╝ ╚═════╝ ╚══════╝",
    "      your privacy fren",
];

const SMALL_LOGO: &[&str] = &[
    "██████╗ ██╗   ██╗███████╗",
    "██████╔╝╚██████╔╝███████╗",
    "╚═════╝  ╚═════╝ ╚══════╝",
    "your privacy fren",
];

const HELP_TEXT: &str =
    "Up/Down: move  Tab/Left/Right: focus  Type: edit  Enter: run  Esc: clear output  q: quit";

impl App {
    fn run(&mut self) {
        match build_command(self.current_form()) {
            Ok(args) => match run_cast_command(&args) {
                Ok(result) => {
                    self.last_command = result.command_preview;
                    self.output = result.output;
                    self.status = if result.success {
                        "Command finished".to_string()
                    } else {
                        "cast wallet returned a non-zero status".to_string()
                    };
                }
                Err(err) => {
                    self.last_command = format!("cast {}", args.join(" "));
                    self.output = err.to_string();
                    self.status = "Failed to launch cast".to_string();
                }
            },
            Err(err) => {
                self.output = err.to_string();
                self.status = "Missing or invalid input".to_string();
            }
        }
    }
}

fn main() -> AppResult<()> {
    let mut terminal = setup_terminal()?;
    let app_result = run_app(&mut terminal);
    restore_terminal(&mut terminal)?;
    app_result
}

fn setup_terminal() -> AppResult<AppTerminal> {
    enable_raw_mode().map_err(|source| AppError::io("failed to enable raw mode", source))?;
    execute!(io::stdout(), EnterAlternateScreen)
        .map_err(|source| AppError::io("failed to enter alternate screen", source))?;
    let backend = CrosstermBackend::new(io::stdout());
    Terminal::new(backend).map_err(|source| AppError::io("failed to initialize terminal", source))
}

fn restore_terminal(terminal: &mut AppTerminal) -> AppResult<()> {
    disable_raw_mode().map_err(|source| AppError::io("failed to disable raw mode", source))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|source| AppError::io("failed to leave alternate screen", source))?;
    terminal
        .show_cursor()
        .map_err(|source| AppError::io("failed to show cursor", source))?;
    Ok(())
}

fn run_app(terminal: &mut AppTerminal) -> AppResult<()> {
    let mut app = App::new();

    loop {
        terminal
            .draw(|frame| render(frame, &app))
            .map_err(|source| AppError::io("failed to draw terminal frame", source))?;

        if !event::poll(Duration::from_millis(100))
            .map_err(|source| AppError::io("failed while polling terminal events", source))?
        {
            continue;
        }

        let Event::Key(key) = event::read()
            .map_err(|source| AppError::io("failed to read terminal event", source))?
        else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        match key.code {
            KeyCode::Char('q') => break,
            KeyCode::Left => app.move_focus_left(),
            KeyCode::Right | KeyCode::Tab => app.move_focus_right(),
            KeyCode::Esc => app.clear_output(),
            KeyCode::Up if app.focus == Focus::Actions => app.select_prev_action(),
            KeyCode::Down if app.focus == Focus::Actions => app.select_next_action(),
            KeyCode::Up if app.focus == Focus::Fields => app.select_prev_field(),
            KeyCode::Down if app.focus == Focus::Fields => app.select_next_field(),
            KeyCode::Backspace if app.focus == Focus::Fields => app.backspace(),
            KeyCode::Enter => app.run(),
            KeyCode::Char(ch) if app.focus == Focus::Fields => app.insert_char(ch),
            _ => {}
        }
    }

    Ok(())
}

fn render(frame: &mut Frame, app: &App) {
    let compact = frame.area().height < 28 || frame.area().width < 90;
    let [hero_area, body_area, footer_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if compact { 6 } else { 10 }),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .areas(frame.area());

    render_hero(frame, hero_area, compact);
    render_body(frame, body_area, app);
    render_footer(frame, footer_area, app);
}

fn render_hero(frame: &mut Frame, area: Rect, compact: bool) {
    let hero_block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ZUS WALLET ARCADE ", Style::default().fg(Color::Yellow)),
            Span::raw(" ratatui x Foundry cast "),
        ]))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .style(Style::default().fg(Color::Blue));
    let inner = hero_block.inner(area);
    frame.render_widget(hero_block, area);
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        inner,
    );

    let logo_lines = if compact { SMALL_LOGO } else { LOGO };
    let logo = Paragraph::new(Text::from(
        logo_lines
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let style = if idx + 1 == logo_lines.len() {
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::Black)
                        .add_modifier(Modifier::ITALIC)
                } else {
                    Style::default()
                        .fg(Color::Blue)
                        .bg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                };
                Line::from(Span::styled(*line, style))
            })
            .collect::<Vec<_>>(),
    ))
    .alignment(Alignment::Center);
    frame.render_widget(logo, inner);
}

fn render_body(frame: &mut Frame, area: Rect, app: &App) {
    if area.height < 12 || area.width < 95 {
        let [actions_area, rest_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(8)])
            .areas(area);
        render_actions(frame, actions_area, app);
        render_form_and_output(frame, rest_area, app, true);
    } else {
        let [left, right] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(32), Constraint::Percentage(68)])
            .areas(area);

        render_actions(frame, left, app);
        render_form_and_output(frame, right, app, false);
    }
}

fn render_actions(frame: &mut Frame, area: Rect, app: &App) {
    let items = app
        .forms
        .iter()
        .enumerate()
        .map(|(index, form)| {
            let selected = index == app.selected_action;
            let title_style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(vec![
                Line::from(Span::styled(form.label, title_style)),
                Line::from(Span::styled(
                    format!("  {}", form.command_label),
                    Style::default().fg(Color::Blue),
                )),
            ])
        })
        .collect::<Vec<_>>();

    let title = if app.focus == Focus::Actions {
        " Wallet Actions [focused] "
    } else {
        " Wallet Actions "
    };

    let widget = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(widget, area);
}

fn render_form_and_output(frame: &mut Frame, area: Rect, app: &App, compact: bool) {
    let field_height = if compact {
        4_u16.max(app.current_form().fields.len() as u16 + 2)
    } else {
        6_u16.max((app.current_form().fields.len() as u16 * 2) + 2)
    };

    let [info_area, fields_area, output_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if compact { 4 } else { 6 }),
            Constraint::Length(field_height),
            Constraint::Min(if compact { 3 } else { 6 }),
        ])
        .areas(area);

    render_form_info(frame, info_area, app);
    render_fields(frame, fields_area, app);
    render_output(frame, output_area, app);
}

fn render_form_info(frame: &mut Frame, area: Rect, app: &App) {
    let form = app.current_form();
    let info = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("Selected: ", Style::default().fg(Color::Yellow)),
            Span::styled(form.label, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Command: ", Style::default().fg(Color::Yellow)),
            Span::raw(form.command_label),
        ]),
        Line::from(vec![
            Span::styled("About: ", Style::default().fg(Color::Yellow)),
            Span::raw(form.description),
        ]),
    ]))
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .title(" Command Deck ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );
    frame.render_widget(info, area);
}

fn render_fields(frame: &mut Frame, area: Rect, app: &App) {
    let form = app.current_form();
    let lines = form
        .fields
        .iter()
        .enumerate()
        .flat_map(|(index, field)| {
            let selected = app.focus == Focus::Fields && index == app.selected_field;
            let label_style = if selected {
                Style::default().fg(Color::Black).bg(Color::Green)
            } else {
                Style::default().fg(Color::Green)
            };

            let value = if field.value.is_empty() {
                field.hint.to_string()
            } else if field.sensitive {
                "*".repeat(field.value.chars().count().max(4))
            } else {
                field.value.clone()
            };

            let value_style = if field.value.is_empty() {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            let required = if field.required { " *" } else { "" };

            vec![
                Line::from(vec![
                    Span::styled(field.label, label_style),
                    Span::styled(required, Style::default().fg(Color::Yellow)),
                    Span::raw(": "),
                    Span::styled(value, value_style),
                ]),
                Line::from(""),
            ]
        })
        .collect::<Vec<_>>();

    let title = if app.focus == Focus::Fields {
        " Wallet Fields [focused] "
    } else {
        " Wallet Fields "
    };

    let widget = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .padding(Padding::horizontal(1)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(widget, area);

    if app.focus == Focus::Fields {
        if let Some(field) = app.current_field() {
            let row = app.selected_field as u16 * 2;
            let cursor_content = if field.value.is_empty() {
                String::new()
            } else if field.sensitive {
                "*".repeat(field.value.chars().count().max(4))
            } else {
                field.value.clone()
            };
            let cursor_x = area
                .x
                .saturating_add(
                    field.label.len() as u16 + 5 + cursor_content.chars().count() as u16,
                )
                .min(area.right().saturating_sub(2));
            let cursor_y = area.y.saturating_add(1 + row);
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

fn render_output(frame: &mut Frame, area: Rect, app: &App) {
    let widget = Paragraph::new(app.output.as_str())
        .block(
            Block::default()
                .title(format!(" Output | {} ", app.last_command))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(widget, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let footer = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Yellow)),
            Span::styled(&app.status, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::styled(HELP_TEXT, Style::default().fg(Color::Gray))),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(Clear, area.inner(Margin::new(0, 0)));
    frame.render_widget(footer, area);
}

fn build_command(form: &ActionForm) -> AppResult<Vec<String>> {
    match form.kind {
        ActionKind::CheckAddress => {
            let private_key = required_value(form, "private_key", "Private key is required.")?;
            Ok(vec![
                "wallet".to_string(),
                "address".to_string(),
                "--private-key".to_string(),
                private_key,
            ])
        }
        ActionKind::CreateWallet => {
            let account_name = form.value("account_name");
            let keystore_dir = normalize_path(form.value("keystore_dir"));
            let password = form.value("password");
            let number = if form.value("number").is_empty() {
                "1"
            } else {
                form.value("number")
            };
            let parsed_number: usize = number
                .parse()
                .map_err(|_| AppError::message("Number must be a positive integer."))?;
            if parsed_number == 0 {
                return Err(AppError::message("Number must be at least 1."));
            }

            let mut args = vec!["wallet".to_string(), "new".to_string()];
            if parsed_number != 1 {
                args.push("--number".to_string());
                args.push(parsed_number.to_string());
            }

            let should_save =
                !account_name.is_empty() || !keystore_dir.is_empty() || !password.is_empty();
            if should_save {
                if password.is_empty() {
                    return Err(AppError::message(
                        "Password is required when saving a new wallet to a keystore.",
                    ));
                }
                let target_dir = if keystore_dir.is_empty() {
                    default_foundry_keystore_dir()
                } else {
                    keystore_dir
                };
                args.push(target_dir);
                if !account_name.is_empty() {
                    args.push(account_name.to_string());
                }
                args.push("--unsafe-password".to_string());
                args.push(password.to_string());
            }

            Ok(args)
        }
        ActionKind::ImportWallet => {
            let account_name = required_value(form, "account_name", "Account name is required.")?;
            let private_key = required_value(form, "private_key", "Private key is required.")?;
            let password = required_value(form, "password", "Password is required.")?;
            let keystore_dir = normalize_path(form.value("keystore_dir"));

            let mut args = vec!["wallet".to_string(), "import".to_string()];
            if !keystore_dir.is_empty() {
                args.push("--keystore-dir".to_string());
                args.push(keystore_dir);
            }
            args.push(account_name);
            args.push("--private-key".to_string());
            args.push(private_key);
            args.push("--unsafe-password".to_string());
            args.push(password);
            Ok(args)
        }
    }
}

fn required_value(form: &ActionForm, key: &str, message: &str) -> AppResult<String> {
    let value = form.value(key);
    if value.is_empty() {
        return Err(AppError::message(message.to_string()));
    }
    Ok(value.to_string())
}

fn run_cast_command(args: &[String]) -> AppResult<CommandResult> {
    let output = Command::new("cast")
        .args(args)
        .output()
        .map_err(|source| AppError::command_launch(format!("cast {}", args.join(" ")), source))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = match (stdout.trim(), stderr.trim()) {
        ("", "") => "(no output)".to_string(),
        ("", stderr) => stderr.to_string(),
        (stdout, "") => stdout.to_string(),
        (stdout, stderr) => format!("{stdout}\n\n{stderr}"),
    };

    Ok(CommandResult {
        command_preview: format!("cast {}", format_command_preview(args)),
        output: combined,
        success: output.status.success(),
    })
}

fn format_command_preview(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg.contains(' ') {
                format!("\"{arg}\"")
            } else {
                arg.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_path(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed == "~" {
        return env::var("HOME").unwrap_or_else(|_| "~".to_string());
    }
    if let Some(rest) = trimmed.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    trimmed.to_string()
}

fn default_foundry_keystore_dir() -> String {
    env::var("HOME")
        .map(|home| format!("{home}/.foundry/keystores"))
        .unwrap_or_else(|_| "~/.foundry/keystores".to_string())
}
