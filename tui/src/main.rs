mod error;
mod types;

use std::fmt::Write as _;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use crate::error::{AppError, AppResult};
use crate::types::{
    ActionForm, ActionKind, ApiCampaignSummary, ApiClaimPayload, App, AppTerminal, CommandResult,
    Focus,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use k256::{PublicKey, SecretKey, elliptic_curve::sec1::ToEncodedPoint};
use rand::{RngCore, rngs::OsRng};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
};
use reqwest::{StatusCode, blocking::Client};

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

struct CampaignLookupResult {
    campaign: ApiCampaignSummary,
    claim_status: ClaimStatus,
}

enum ClaimStatus {
    Eligible(ApiClaimPayload),
    Missing,
    Error(String),
}

struct ResolvedWallet {
    account_label: String,
    address: String,
    pub_key_x: [u8; 32],
    pub_key_y: [u8; 32],
}

struct AutoWitnessSecrets {
    message: [u8; 8],
    stealth_tweak: [u8; 32],
    c_scalar: [u8; 32],
    s_scalar: [u8; 32],
    nullifier_x: [u8; 32],
    nullifier_y: [u8; 32],
}

impl App {
    fn run(&mut self) {
        let action_kind = self.current_form().kind;
        let create_wallet_saves_to_keystore = action_kind == ActionKind::CreateWallet
            && create_wallet_should_save(self.current_form());

        match execute_action(self.current_form()) {
            Ok(result) => {
                let auto_filled_address = if matches!(
                    action_kind,
                    ActionKind::CheckAddress
                        | ActionKind::CreateWallet
                        | ActionKind::GenerateZkWitness
                ) && result.success
                {
                    extract_wallet_address(&result.output)
                } else {
                    None
                };

                self.last_command = result.command_preview;
                self.output = if action_kind == ActionKind::ListAccounts && result.success {
                    format_saved_accounts_output(&result.output)
                } else {
                    result.output
                };
                self.status = if result.success {
                    "Completed".to_string()
                } else {
                    "Completed with issues".to_string()
                };

                if let Some(address) = auto_filled_address {
                    self.set_form_field_value(
                        ActionKind::CampaignExplorer,
                        "wallet_address",
                        address,
                    );
                    self.status = "Completed and copied address into Campaign Explorer".to_string();
                }

                if action_kind == ActionKind::CreateWallet
                    && result.success
                    && !create_wallet_saves_to_keystore
                {
                    self.output.push_str(
                        "\n\nNot saved: this keypair was only printed to the output. Add a password plus account name or keystore dir if you want Foundry to store it.",
                    );
                    self.status = "Completed, but the new wallet was not saved".to_string();
                }
            }
            Err(err) => {
                self.last_command = fallback_command_preview(self.current_form());
                self.output = err.to_string();
                self.status = "Request failed".to_string();
                focus_fields_for_error(self, &err.to_string());
            }
        }
    }
}

fn execute_action(form: &ActionForm) -> AppResult<CommandResult> {
    match form.kind {
        ActionKind::CampaignExplorer => run_campaign_lookup(form),
        ActionKind::GenerateZkWitness => run_generate_zk_witness(form),
        _ => {
            let args = build_command(form)?;
            run_cast_command(&args)
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
    let [_, logo_area, _] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(logo_lines.len() as u16),
            Constraint::Fill(1),
        ])
        .areas(inner);
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
    frame.render_widget(logo, logo_area);
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
                mask_sensitive_value(&field.value)
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
                mask_sensitive_value(&field.value)
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

fn mask_sensitive_value(value: &str) -> String {
    "*".repeat(value.chars().count())
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
        ActionKind::CampaignExplorer => Err(AppError::message(
            "Campaign Explorer is handled through the proof API, not cast.",
        )),
        ActionKind::GenerateZkWitness => Err(AppError::message(
            "Generate ZK Witness is handled through the Noir circuit flow, not cast.",
        )),
        ActionKind::ListAccounts => {
            let keystore_dir = normalize_path(form.value("keystore_dir"));
            let mut args = vec!["wallet".to_string(), "list".to_string()];
            if !keystore_dir.is_empty() {
                args.push("--dir".to_string());
                args.push(keystore_dir);
            }
            Ok(args)
        }
        ActionKind::CheckAddress => build_wallet_address_command(form),
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

fn fallback_command_preview(form: &ActionForm) -> String {
    match form.kind {
        ActionKind::CampaignExplorer => {
            let api_base = normalize_api_base_url(form.value("api_base_url"))
                .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
            format!("GET {api_base}/campaigns")
        }
        _ => form.command_label.to_string(),
    }
}

fn create_wallet_should_save(form: &ActionForm) -> bool {
    !form.value("account_name").is_empty()
        || !form.value("keystore_dir").is_empty()
        || !form.value("password").is_empty()
}

fn focus_fields_for_error(app: &mut App, message: &str) {
    let lowered = message.to_ascii_lowercase();
    let key = if lowered.contains("api base url") {
        Some("api_base_url")
    } else if lowered.contains("campaign id") {
        Some("campaign_id")
    } else if lowered.contains("wallet address") {
        Some("wallet_address")
    } else if lowered.contains("wallet account") {
        Some("wallet_account")
    } else if lowered.contains("saved account") || lowered.contains("account name") {
        Some("account_name")
    } else if lowered.contains("keystore path") {
        Some("keystore_path")
    } else if lowered.contains("circuit dir") {
        Some("circuit_dir")
    } else if lowered.contains("witness name") {
        Some("witness_name")
    } else if lowered.contains("keystore dir") {
        Some("keystore_dir")
    } else if lowered.contains("password") {
        Some("password")
    } else if lowered.contains("private key") {
        Some("private_key")
    } else if lowered.contains("number") {
        Some("number")
    } else {
        app.current_form().fields.first().map(|field| field.key)
    };

    if let Some(key) = key {
        app.select_field_by_key(key);
    } else {
        app.move_focus_right();
    }
}

fn build_wallet_address_command(form: &ActionForm) -> AppResult<Vec<String>> {
    let private_key = form.value("private_key");
    let account_name = form.value("account_name");
    let password = form.value("password");
    let keystore_path = normalize_path(form.value("keystore_path"));

    let mut args = vec!["wallet".to_string(), "address".to_string()];
    if !private_key.is_empty() {
        args.push("--private-key".to_string());
        args.push(private_key.to_string());
        return Ok(args);
    }

    build_saved_wallet_address_command(account_name, &keystore_path, password)
}

fn required_value(form: &ActionForm, key: &str, message: &str) -> AppResult<String> {
    let value = form.value(key);
    if value.is_empty() {
        return Err(AppError::message(message.to_string()));
    }
    Ok(value.to_string())
}

fn run_campaign_lookup(form: &ActionForm) -> AppResult<CommandResult> {
    let api_base = normalize_api_base_url(&required_value(
        form,
        "api_base_url",
        "API base URL is required.",
    )?)?;
    let campaigns_url = format!("{api_base}/campaigns");
    let wallet_address = resolve_campaign_wallet_address(form)?;

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|source| AppError::http("failed to build the proof API client", source))?;

    let campaigns_response = client
        .get(&campaigns_url)
        .send()
        .map_err(|source| AppError::http(format!("failed to fetch {campaigns_url}"), source))?;
    let campaigns =
        parse_json_response::<Vec<ApiCampaignSummary>>(campaigns_response, &campaigns_url)?;

    if let Some(address) = wallet_address.as_deref() {
        let lookups = campaigns
            .into_iter()
            .map(|campaign| lookup_claim_for_campaign(&client, &api_base, address, campaign))
            .collect::<Vec<_>>();
        let has_errors = lookups
            .iter()
            .any(|lookup| matches!(lookup.claim_status, ClaimStatus::Error(_)));

        Ok(CommandResult {
            command_preview: format!("GET {campaigns_url} + claim lookups for {address}"),
            output: format_campaign_lookup_output(&api_base, address, &lookups),
            success: !has_errors,
        })
    } else {
        Ok(CommandResult {
            command_preview: format!("GET {campaigns_url}"),
            output: format_campaign_catalog_output(&api_base, &campaigns),
            success: true,
        })
    }
}

fn run_generate_zk_witness(form: &ActionForm) -> AppResult<CommandResult> {
    let api_base = normalize_api_base_url(&required_value(
        form,
        "api_base_url",
        "API base URL is required.",
    )?)?;
    let campaign_id = required_value(form, "campaign_id", "Campaign ID is required.")?;
    let circuit_dir = normalize_circuit_dir(&required_value(
        form,
        "circuit_dir",
        "Circuit dir is required.",
    )?)?;
    let witness_name = if form.value("witness_name").is_empty() {
        "claim_witness".to_string()
    } else {
        form.value("witness_name").to_string()
    };

    let wallet = resolve_wallet_for_zk(form)?;
    let claim = fetch_claim_payload_for_campaign(&api_base, &campaign_id, &wallet.address)?;
    let prover_path = circuit_dir.join("Prover.toml");
    let secrets = generate_auto_witness_secrets()?;
    let prover_contents = build_prover_toml_contents(&wallet, &claim, &secrets)?;

    fs::write(&prover_path, prover_contents).map_err(|source| {
        AppError::io(format!("failed to write {}", prover_path.display()), source)
    })?;

    let execute_result = run_nargo_execute(&circuit_dir, &witness_name)?;
    let witness_path = circuit_dir
        .join("target")
        .join(format!("{witness_name}.gz"));
    let bytecode_path = circuit_dir.join("target").join("stealthdrop.json");
    let mut output = String::new();
    let _ = writeln!(output, "Resolved wallet: {}", wallet.account_label);
    let _ = writeln!(output, "Resolved address: {}", wallet.address);
    let _ = writeln!(output, "Campaign ID: {campaign_id}");
    let _ = writeln!(output, "Claim leaf address: {}", claim.leaf_address);
    let _ = writeln!(output, "Prover file: {}", prover_path.display());
    let _ = writeln!(output, "Witness path: {}", witness_path.display());
    let _ = writeln!(output, "Bytecode path: {}", bytecode_path.display());
    let _ = writeln!(output, "Message hex: {}", bytes_to_hex(&secrets.message));
    let _ = writeln!(
        output,
        "Stealth tweak: {}",
        bytes_to_hex(&secrets.stealth_tweak)
    );
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "nargo execute output:\n{}",
        execute_result.output.trim()
    );
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "Next proof step: bb prove -s ultra_honk -b {} -w {} -o {}/proof",
        bytecode_path.display(),
        witness_path.display(),
        circuit_dir.join("target").display()
    );
    let _ = writeln!(
        output,
        "Note: the installed bb CLI still needs extra VK setup on this machine, so this action currently wires the proven path through Noir witness generation."
    );

    Ok(CommandResult {
        command_preview: format!(
            "write {} + nargo execute {}",
            prover_path.display(),
            witness_name
        ),
        output,
        success: execute_result.success,
    })
}

fn resolve_wallet_for_zk(form: &ActionForm) -> AppResult<ResolvedWallet> {
    let account_name = required_value(
        form,
        "wallet_account",
        "Wallet account is required for ZK witness generation.",
    )?;
    let password = required_value(
        form,
        "password",
        "Password is required for ZK witness generation.",
    )?;
    let keystore_path = normalize_path(form.value("keystore_path"));

    let address_result = run_cast_command(&build_saved_wallet_address_command(
        &account_name,
        &keystore_path,
        &password,
    )?)?;
    if !address_result.success {
        return Err(AppError::message(format!(
            "failed to resolve wallet address from Foundry keystore: {}",
            address_result.output.trim()
        )));
    }

    let public_key_result = run_cast_command(&build_saved_wallet_public_key_command(
        &account_name,
        &keystore_path,
        &password,
    )?)?;
    if !public_key_result.success {
        return Err(AppError::message(format!(
            "failed to resolve wallet public key from Foundry keystore: {}",
            public_key_result.output.trim()
        )));
    }

    let address = extract_wallet_address(&address_result.output).ok_or_else(|| {
        AppError::message(
            "cast returned output, but I could not extract a wallet address from it.".to_string(),
        )
    })?;
    let public_key_bytes = extract_public_key_bytes(&public_key_result.output)?;

    Ok(ResolvedWallet {
        account_label: normalize_foundry_account_name(&account_name),
        address,
        pub_key_x: public_key_bytes[..32]
            .try_into()
            .map_err(|_| AppError::message("failed to parse wallet public key x coordinate"))?,
        pub_key_y: public_key_bytes[32..]
            .try_into()
            .map_err(|_| AppError::message("failed to parse wallet public key y coordinate"))?,
    })
}

fn fetch_claim_payload_for_campaign(
    api_base: &str,
    campaign_id: &str,
    wallet_address: &str,
) -> AppResult<ApiClaimPayload> {
    let claim_url = format!("{api_base}/campaigns/{campaign_id}/claim/{wallet_address}");
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|source| AppError::http("failed to build the proof API client", source))?;
    let response = client
        .get(&claim_url)
        .send()
        .map_err(|source| AppError::http(format!("failed to fetch {claim_url}"), source))?;
    parse_json_response::<ApiClaimPayload>(response, &claim_url)
}

fn build_prover_toml_contents(
    wallet: &ResolvedWallet,
    claim: &ApiClaimPayload,
    secrets: &AutoWitnessSecrets,
) -> AppResult<String> {
    let mut prover = String::new();
    prover.push_str(&format_u8_array_toml("c", &secrets.c_scalar));
    prover.push_str(&format_field_scalar_toml(
        "eligible_index",
        &claim.noir_inputs.eligible_index,
    ));
    prover.push_str(&format_string_array_toml(
        "eligible_path",
        &claim.noir_inputs.eligible_path,
    ));
    prover.push_str(&format_field_scalar_toml(
        "eligible_root",
        &claim.noir_inputs.eligible_root,
    ));
    prover.push_str(&format_u8_array_toml("message", &secrets.message));
    prover.push_str(&format_u8_array_toml("nullifier_x", &secrets.nullifier_x));
    prover.push_str(&format_u8_array_toml("nullifier_y", &secrets.nullifier_y));
    prover.push_str(&format_u8_array_toml("pub_key_x", &wallet.pub_key_x));
    prover.push_str(&format_u8_array_toml("pub_key_y", &wallet.pub_key_y));
    prover.push_str(&format_u8_array_toml("s", &secrets.s_scalar));
    prover.push_str(&format_u8_array_toml(
        "stealth_tweak",
        &secrets.stealth_tweak,
    ));
    Ok(prover)
}

fn resolve_campaign_wallet_address(form: &ActionForm) -> AppResult<Option<String>> {
    let wallet_identifier = form.value("wallet_address");
    let password = form.value("password");
    let keystore_path = normalize_path(form.value("keystore_path"));

    if wallet_identifier.is_empty() && keystore_path.is_empty() {
        return Ok(None);
    }

    if !wallet_identifier.is_empty() {
        if let Ok(address) = normalize_wallet_address(wallet_identifier) {
            return Ok(Some(address));
        }
    }

    let args = build_saved_wallet_address_command(wallet_identifier, &keystore_path, password)?;
    let result = run_cast_command(&args)?;
    if !result.success {
        return Err(AppError::message(format!(
            "failed to resolve wallet address from Foundry account or keystore: {}",
            result.output.trim()
        )));
    }

    extract_wallet_address(&result.output)
        .ok_or_else(|| {
            AppError::message(
                "cast returned output, but I could not extract a wallet address from it."
                    .to_string(),
            )
        })
        .map(Some)
}

fn build_saved_wallet_address_command(
    account_name: &str,
    keystore_path: &str,
    password: &str,
) -> AppResult<Vec<String>> {
    let mut args = vec!["wallet".to_string(), "address".to_string()];

    if !keystore_path.is_empty() {
        if password.is_empty() {
            return Err(AppError::message(
                "Password is required for a saved Foundry account or keystore path.",
            ));
        }
        args.push("--keystore".to_string());
        args.push(keystore_path.to_string());
        args.push("--password".to_string());
        args.push(password.to_string());
        return Ok(args);
    }

    if !account_name.is_empty() {
        let normalized_account_name = normalize_foundry_account_name(account_name);
        if password.is_empty() {
            return Err(AppError::message(
                "Password is required for a saved Foundry account or keystore path.",
            ));
        }
        args.push("--keystore".to_string());
        args.push(format!(
            "{}/{}",
            default_foundry_keystore_dir(),
            normalized_account_name
        ));
        args.push("--password".to_string());
        args.push(password.to_string());
        return Ok(args);
    }

    Err(AppError::message(
        "Provide a private key, a saved account name, or a keystore path.",
    ))
}

fn build_saved_wallet_public_key_command(
    account_name: &str,
    keystore_path: &str,
    password: &str,
) -> AppResult<Vec<String>> {
    let mut args = vec!["wallet".to_string(), "public-key".to_string()];

    if !keystore_path.is_empty() {
        if password.is_empty() {
            return Err(AppError::message(
                "Password is required for a saved Foundry account or keystore path.",
            ));
        }
        args.push("--keystore".to_string());
        args.push(keystore_path.to_string());
        args.push("--password".to_string());
        args.push(password.to_string());
        return Ok(args);
    }

    if !account_name.is_empty() {
        if password.is_empty() {
            return Err(AppError::message(
                "Password is required for a saved Foundry account or keystore path.",
            ));
        }
        args.push("--keystore".to_string());
        args.push(format!(
            "{}/{}",
            default_foundry_keystore_dir(),
            normalize_foundry_account_name(account_name)
        ));
        args.push("--password".to_string());
        args.push(password.to_string());
        return Ok(args);
    }

    Err(AppError::message(
        "Provide a saved account name or keystore path.",
    ))
}

fn normalize_circuit_dir(raw: &str) -> AppResult<PathBuf> {
    let normalized = PathBuf::from(normalize_path(raw));
    if !normalized.exists() {
        return Err(AppError::message(format!(
            "Circuit dir does not exist: {}",
            normalized.display()
        )));
    }
    if !normalized.join("Nargo.toml").exists() {
        return Err(AppError::message(format!(
            "Circuit dir does not contain Nargo.toml: {}",
            normalized.display()
        )));
    }
    Ok(normalized)
}

fn normalize_api_base_url(raw: &str) -> AppResult<String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(AppError::message("API base URL is required."));
    }
    Ok(trimmed.to_string())
}

fn normalize_wallet_address(raw: &str) -> AppResult<String> {
    let trimmed = raw.trim();
    if trimmed.len() != 42 || !trimmed.starts_with("0x") {
        return Err(AppError::message(format!(
            "Wallet address must look like 0x followed by 40 hex characters: {trimmed}"
        )));
    }

    if !trimmed[2..]
        .chars()
        .all(|character| character.is_ascii_hexdigit())
    {
        return Err(AppError::message(format!(
            "Wallet address contains non-hex characters: {trimmed}"
        )));
    }

    Ok(format!("0x{}", trimmed[2..].to_ascii_lowercase()))
}

fn normalize_foundry_account_name(raw: &str) -> String {
    raw.trim()
        .strip_suffix(" (Local)")
        .unwrap_or(raw.trim())
        .trim()
        .to_string()
}

fn extract_public_key_bytes(output: &str) -> AppResult<[u8; 64]> {
    let raw = output
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("0x"))
        .ok_or_else(|| AppError::message("could not find a public key in cast output"))?;

    let trimmed = raw.trim_start_matches("0x");
    let without_prefix = if trimmed.len() == 130 && trimmed.starts_with("04") {
        &trimmed[2..]
    } else {
        trimmed
    };

    let bytes = parse_hex_bytes(without_prefix, 64, "Public key")?;
    bytes
        .try_into()
        .map_err(|_| AppError::message("public key must decode to exactly 64 bytes".to_string()))
}

fn parse_hex_bytes(raw: &str, expected_len: usize, label: &str) -> AppResult<Vec<u8>> {
    let trimmed = raw.trim().trim_start_matches("0x");
    if trimmed.len() != expected_len * 2 {
        return Err(AppError::message(format!(
            "{label} must be exactly {} hex characters",
            expected_len * 2
        )));
    }

    if !trimmed
        .chars()
        .all(|character| character.is_ascii_hexdigit())
    {
        return Err(AppError::message(format!(
            "{label} contains non-hex characters"
        )));
    }

    (0..expected_len)
        .map(|index| {
            let start = index * 2;
            u8::from_str_radix(&trimmed[start..start + 2], 16)
                .map_err(|_| AppError::message(format!("{label} contains invalid hex byte data")))
        })
        .collect()
}

fn format_u8_array_toml(name: &str, bytes: &[u8]) -> String {
    let body = bytes
        .iter()
        .map(|byte| format!("\"{byte}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name} = [{body}]\n")
}

fn format_string_array_toml(name: &str, values: &[String]) -> String {
    let body = values
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name} = [{body}]\n")
}

fn format_field_scalar_toml(name: &str, value: &str) -> String {
    format!("{name} = \"{}\"\n", value.trim())
}

fn generate_auto_witness_secrets() -> AppResult<AutoWitnessSecrets> {
    let mut rng = OsRng;

    let mut message = [0_u8; 8];
    rng.fill_bytes(&mut message);

    let stealth_tweak = secret_key_bytes();
    let c_scalar = secret_key_bytes();
    let s_scalar = secret_key_bytes();
    let nullifier_secret = SecretKey::random(&mut rng);
    let nullifier_public = PublicKey::from_secret_scalar(&nullifier_secret.to_nonzero_scalar());
    let encoded = nullifier_public.to_encoded_point(false);
    let x = encoded
        .x()
        .ok_or_else(|| AppError::message("failed to derive nullifier x coordinate"))?;
    let y = encoded
        .y()
        .ok_or_else(|| AppError::message("failed to derive nullifier y coordinate"))?;
    let mut nullifier_x = [0_u8; 32];
    let mut nullifier_y = [0_u8; 32];
    nullifier_x.copy_from_slice(x);
    nullifier_y.copy_from_slice(y);

    Ok(AutoWitnessSecrets {
        message,
        stealth_tweak,
        c_scalar,
        s_scalar,
        nullifier_x,
        nullifier_y,
    })
}

fn secret_key_bytes() -> [u8; 32] {
    let mut rng = OsRng;
    let secret = SecretKey::random(&mut rng);
    secret.to_bytes().into()
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn run_nargo_execute(circuit_dir: &Path, witness_name: &str) -> AppResult<CommandResult> {
    let output = Command::new("nargo")
        .current_dir(circuit_dir)
        .args(["execute", witness_name, "--skip-brillig-constraints-check"])
        .stdin(Stdio::null())
        .output()
        .map_err(|source| {
            AppError::command_launch(
                format!(
                    "nargo execute {} --skip-brillig-constraints-check",
                    witness_name
                ),
                source,
            )
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = match (stdout.trim(), stderr.trim()) {
        ("", "") => "(no output)".to_string(),
        ("", stderr) => stderr.to_string(),
        (stdout, "") => stdout.to_string(),
        (stdout, stderr) => format!("{stdout}\n\n{stderr}"),
    };

    Ok(CommandResult {
        command_preview: format!(
            "nargo execute {} --skip-brillig-constraints-check",
            witness_name
        ),
        output: combined,
        success: output.status.success(),
    })
}

fn format_saved_accounts_output(output: &str) -> String {
    let accounts = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(normalize_foundry_account_name)
        .collect::<Vec<_>>();

    if accounts.is_empty() {
        return "(no saved Foundry accounts found)".to_string();
    }

    let mut formatted = String::from("Paste one of these names into Wallet / Account:\n");
    for account in accounts {
        let _ = writeln!(formatted, "- {account}");
    }
    formatted
}

fn extract_wallet_address(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        line.split_whitespace().find_map(|token| {
            let candidate = token.trim_matches(|character: char| {
                matches!(character, '"' | '\'' | ',' | ';' | '(' | ')')
            });
            normalize_wallet_address(candidate).ok()
        })
    })
}

fn parse_json_response<T>(response: reqwest::blocking::Response, url: &str) -> AppResult<T>
where
    T: serde::de::DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        let body = read_response_body(response);
        return Err(AppError::message(format!(
            "proof API returned {status} for {url}{}",
            format_response_body_suffix(&body)
        )));
    }

    response
        .json::<T>()
        .map_err(|source| AppError::http(format!("failed to decode JSON from {url}"), source))
}

fn read_response_body(response: reqwest::blocking::Response) -> String {
    response.text().unwrap_or_default()
}

fn format_response_body_suffix(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        let single_line = trimmed.lines().next().unwrap_or(trimmed);
        format!(" | {single_line}")
    }
}

fn lookup_claim_for_campaign(
    client: &Client,
    api_base: &str,
    wallet_address: &str,
    campaign: ApiCampaignSummary,
) -> CampaignLookupResult {
    let claim_url = format!(
        "{api_base}/campaigns/{}/claim/{}",
        campaign.campaign_id, wallet_address
    );

    let claim_status = match client.get(&claim_url).send() {
        Ok(response) => match response.status() {
            status if status.is_success() => match response.json::<ApiClaimPayload>() {
                Ok(claim) => ClaimStatus::Eligible(claim),
                Err(error) => ClaimStatus::Error(format!(
                    "failed to decode claim JSON from {claim_url}: {error}"
                )),
            },
            StatusCode::NOT_FOUND => ClaimStatus::Missing,
            status => {
                let body = read_response_body(response);
                ClaimStatus::Error(format!(
                    "claim lookup returned {status}{}",
                    format_response_body_suffix(&body)
                ))
            }
        },
        Err(error) => ClaimStatus::Error(format!("request failed: {error}")),
    };

    CampaignLookupResult {
        campaign,
        claim_status,
    }
}

fn format_campaign_catalog_output(api_base: &str, campaigns: &[ApiCampaignSummary]) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "API base: {api_base}");
    let _ = writeln!(output, "Campaigns loaded: {}", campaigns.len());

    if campaigns.is_empty() {
        let _ = writeln!(output);
        let _ = writeln!(output, "No campaigns were returned by the proof API.");
        return output;
    }

    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "Add a wallet address and press Enter again to fetch Noir claim inputs per campaign."
    );

    for (index, campaign) in campaigns.iter().enumerate() {
        write_campaign_header(&mut output, index, campaign);
        let _ = writeln!(output, "status: catalog only");
        let _ = writeln!(output);
    }

    output
}

fn format_campaign_lookup_output(
    api_base: &str,
    wallet_address: &str,
    lookups: &[CampaignLookupResult],
) -> String {
    let mut output = String::new();
    let eligible_count = lookups
        .iter()
        .filter(|lookup| matches!(lookup.claim_status, ClaimStatus::Eligible(_)))
        .count();
    let missing_count = lookups
        .iter()
        .filter(|lookup| matches!(lookup.claim_status, ClaimStatus::Missing))
        .count();
    let error_count = lookups
        .iter()
        .filter(|lookup| matches!(lookup.claim_status, ClaimStatus::Error(_)))
        .count();

    let _ = writeln!(output, "API base: {api_base}");
    let _ = writeln!(output, "Wallet address: {wallet_address}");
    let _ = writeln!(output, "Campaigns loaded: {}", lookups.len());
    let _ = writeln!(output, "Eligible campaigns: {eligible_count}");
    let _ = writeln!(output, "No-claim campaigns: {missing_count}");
    let _ = writeln!(output, "Errored lookups: {error_count}");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "Wallet-side Noir fields still come from your ratatui wallet flow: pub_key_x, pub_key_y, c, s, nullifier_x, nullifier_y, stealth_tweak, message."
    );

    for (index, lookup) in lookups.iter().enumerate() {
        write_campaign_header(&mut output, index, &lookup.campaign);
        match &lookup.claim_status {
            ClaimStatus::Eligible(claim) => {
                let _ = writeln!(output, "status: eligible");
                let _ = writeln!(output, "claim_campaign_id: {}", claim.campaign_id);
                let _ = writeln!(output, "claim_name: {}", claim.name);
                let _ = writeln!(
                    output,
                    "claim_campaign_creator_address: {}",
                    claim.campaign_creator_address
                );
                let _ = writeln!(output, "amount: {}", claim.amount);
                let _ = writeln!(output, "leaf_address: {}", claim.leaf_address);
                let _ = writeln!(output, "leaf_index: {}", claim.index);
                let _ = writeln!(output, "leaf_value: {}", claim.leaf_value);
                let _ = writeln!(output, "proof_path_len: {}", claim.proof.len());
                let _ = writeln!(output, "claim_merkle_root: {}", claim.merkle_root);
                let _ = writeln!(output, "claim_hash_algorithm: {}", claim.hash_algorithm);
                let _ = writeln!(output, "claim_leaf_encoding: {}", claim.leaf_encoding);
                let _ = writeln!(output, "eligible_root: {}", claim.noir_inputs.eligible_root);
                let _ = writeln!(
                    output,
                    "eligible_index: {}",
                    claim.noir_inputs.eligible_index
                );
                let _ = writeln!(output, "noir_leaf_value: {}", claim.noir_inputs.leaf_value);
                let _ = writeln!(output, "tree_depth: {}", claim.noir_inputs.tree_depth);
                let _ = writeln!(output, "eligible_path:");
                for (path_index, item) in claim.noir_inputs.eligible_path.iter().enumerate() {
                    let _ = writeln!(output, "  [{path_index}] {item}");
                }
            }
            ClaimStatus::Missing => {
                let _ = writeln!(output, "status: no claim for this wallet address");
            }
            ClaimStatus::Error(message) => {
                let _ = writeln!(output, "status: lookup error");
                let _ = writeln!(output, "error: {message}");
            }
        }
        let _ = writeln!(output);
    }

    output
}

fn write_campaign_header(output: &mut String, index: usize, campaign: &ApiCampaignSummary) {
    let _ = writeln!(output, "[{}] {}", index + 1, campaign.name);
    let _ = writeln!(output, "campaign_id: {}", campaign.campaign_id);
    let _ = writeln!(
        output,
        "campaign_creator_address: {}",
        campaign.campaign_creator_address
    );
    let _ = writeln!(output, "merkle_root: {}", campaign.merkle_root);
    let _ = writeln!(output, "leaf_count: {}", campaign.leaf_count);
    let _ = writeln!(output, "depth: {}", campaign.depth);
    let _ = writeln!(output, "hash_algorithm: {}", campaign.hash_algorithm);
    let _ = writeln!(output, "leaf_encoding: {}", campaign.leaf_encoding);
}

fn run_cast_command(args: &[String]) -> AppResult<CommandResult> {
    let output = Command::new("cast")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|source| {
            AppError::command_launch(format!("cast {}", format_command_preview(args)), source)
        })?;

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
    let mut mask_next = false;
    args.iter()
        .map(|arg| {
            if mask_next {
                mask_next = false;
                return "<hidden>".to_string();
            }

            if matches!(
                arg.as_str(),
                "--private-key" | "--unsafe-password" | "--password"
            ) {
                mask_next = true;
            }

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
