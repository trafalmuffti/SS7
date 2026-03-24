use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::{App, InputField, Screen, CALEA_TYPES, PBX_TYPES};

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Dashboard => draw_dashboard(f, app),
        Screen::AccountList => draw_account_list(f, app),
        Screen::CreateAccount => draw_create_account(f, app),
        Screen::AccountDetail => draw_account_detail(f, app),
        Screen::SearchByName => draw_search_by_name(f, app),
        Screen::SearchByBalance => draw_search_by_balance(f, app),
        Screen::CreditDebit => draw_credit_debit(f, app),
        Screen::Confirm => draw_confirm(f, app),
        Screen::CaleaList => draw_calea_list(f, app),
        Screen::CaleaCreate => draw_calea_create(f, app),
        Screen::CaleaDetail => draw_calea_detail(f, app),
        Screen::PbxList => draw_pbx_list(f, app),
        Screen::PbxCreate => draw_pbx_create(f, app),
        Screen::PbxDetail => draw_pbx_detail(f, app),
    }

    // Status bar at bottom
    if !app.status_message.is_empty() {
        let area = f.area();
        let status_area = Rect::new(0, area.height.saturating_sub(1), area.width, 1);
        let status = Paragraph::new(app.status_message.as_str())
            .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
        f.render_widget(status, status_area);
    }
}

fn draw_dashboard(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(14),
            Constraint::Min(0),
        ])
        .split(f.area());

    let header_text = vec![
        Line::from(vec![Span::styled(
            "  SS7 BILLING SYSTEM  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Total Accounts:       "),
            Span::styled(
                format!("{}", app.account_count),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  Total Balance:        "),
            Span::styled(
                format!("${:.2}", app.total_balance),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  Active Intercepts:    "),
            Span::styled(
                format!("{}", app.intercept_count),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  Active Fwd Rules:     "),
            Span::styled(
                format!("{}", app.forwarding_count),
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title(" Dashboard "));
    f.render_widget(header, chunks[0]);

    let menu_items = vec![
        ListItem::new("  [1] View All Accounts"),
        ListItem::new("  [2] Create New Account"),
        ListItem::new("  [3] Search by Name"),
        ListItem::new("  [4] Search by Balance Range"),
        ListItem::new(""),
        ListItem::new(Span::styled(
            "  [5] CALEA Intercepts",
            Style::default().fg(Color::Red),
        )),
        ListItem::new(Span::styled(
            "  [6] PBX Call Forwarding",
            Style::default().fg(Color::Magenta),
        )),
        ListItem::new(""),
        ListItem::new("  [q] Quit"),
    ];
    let menu = List::new(menu_items)
        .block(Block::default().borders(Borders::ALL).title(" Menu "))
        .style(Style::default().fg(Color::White));
    f.render_widget(menu, chunks[1]);
}

fn draw_account_list(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let title = Paragraph::new("  Accounts List")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    if app.accounts.is_empty() {
        let empty = Paragraph::new("  No accounts found. Press 'c' to create one.")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(empty, chunks[1]);
    } else {
        let header_cells = ["Name", "MSISDN", "Balance", "Status"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1);

        let rows: Vec<Row> = app
            .accounts
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let style = if i == app.selected_index {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                };
                let status_color = match a.status {
                    ss7_billing::AccountStatus::Active => Color::Green,
                    ss7_billing::AccountStatus::Suspended => Color::Yellow,
                    ss7_billing::AccountStatus::Closed => Color::Red,
                };
                Row::new(vec![
                    Cell::from(a.name.clone()),
                    Cell::from(a.msisdn.clone()),
                    Cell::from(format!("${:.2}", a.balance)),
                    Cell::from(a.status.as_str()).style(Style::default().fg(status_color)),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(30),
                Constraint::Percentage(25),
                Constraint::Percentage(20),
                Constraint::Percentage(25),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} account(s) ", app.accounts.len())),
        );

        f.render_widget(table, chunks[1]);
    }

    let help = Paragraph::new(" Up/Down: Navigate | Enter: Select | c: Create | Esc: Back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

fn draw_create_account(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.area());

    let title = Paragraph::new("  Create New Account")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    draw_input_field(f, " Name ", &app.input_name, app.active_field == InputField::Name, chunks[1]);
    draw_input_field(f, " MSISDN (Phone Number) ", &app.input_msisdn, app.active_field == InputField::Msisdn, chunks[2]);
    draw_input_field(f, " IMSI ", &app.input_imsi, app.active_field == InputField::Imsi, chunks[3]);

    let help = Paragraph::new(" Tab: Next Field | Enter: Create | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[4]);
}

fn draw_account_detail(f: &mut Frame, app: &App) {
    if let Some(ref account) = app.selected_account {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(12),
                Constraint::Length(12),
                Constraint::Min(0),
            ])
            .split(f.area());

        let title = Paragraph::new(format!("  Account: {}", account.name))
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let status_color = match account.status {
            ss7_billing::AccountStatus::Active => Color::Green,
            ss7_billing::AccountStatus::Suspended => Color::Yellow,
            ss7_billing::AccountStatus::Closed => Color::Red,
        };

        let details = vec![
            Line::from(vec![
                Span::raw("  ID:       "),
                Span::styled(&account.id, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("  Name:     "),
                Span::styled(&account.name, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("  MSISDN:   "),
                Span::styled(&account.msisdn, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("  IMSI:     "),
                Span::styled(&account.imsi, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("  Balance:  "),
                Span::styled(
                    format!("${:.2}", account.balance),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Currency: "),
                Span::styled(&account.currency, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("  Status:   "),
                Span::styled(
                    account.status.as_str(),
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Created:  "),
                Span::styled(
                    account.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
        ];
        let detail_widget = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title(" Details "));
        f.render_widget(detail_widget, chunks[1]);

        let actions = vec![
            ListItem::new("  [+] Credit Balance          [-] Debit Balance"),
            ListItem::new("  [s] Suspend Account         [a] Activate Account"),
            ListItem::new("  [x] Close Account           [d] Delete Account"),
            ListItem::new(""),
            ListItem::new(Line::from(Span::styled(
                "  [i] Add CALEA Intercept",
                Style::default().fg(Color::Red),
            ))),
            ListItem::new(Line::from(Span::styled(
                "  [f] Add PBX Forwarding Rule",
                Style::default().fg(Color::Magenta),
            ))),
        ];
        let actions_list = List::new(actions)
            .block(Block::default().borders(Borders::ALL).title(" Actions "))
            .style(Style::default().fg(Color::White));
        f.render_widget(actions_list, chunks[2]);

        let help = Paragraph::new(" Esc: Back to list")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(help, chunks[3]);
    }
}

fn draw_search_by_name(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    let title = Paragraph::new("  Search Accounts by Name")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let search = Paragraph::new(format!("  {}", app.input_search))
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search Query ")
                .border_style(Style::default().fg(Color::Yellow)),
        );
    f.render_widget(search, chunks[1]);

    if !app.accounts.is_empty() {
        draw_results_table(f, app, chunks[2]);
    } else {
        let msg = Paragraph::new("  Type a name and press Enter to search")
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        f.render_widget(msg, chunks[2]);
    }

    let help = Paragraph::new(" Enter: Search | Up/Down: Navigate results | Space: Select | Esc: Back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}

fn draw_search_by_balance(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    let title = Paragraph::new("  Search Accounts by Balance Range")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    draw_input_field(f, " Min Balance ($) ", &app.input_balance_min, app.active_field == InputField::BalanceMin, chunks[1]);
    draw_input_field(f, " Max Balance ($) ", &app.input_balance_max, app.active_field == InputField::BalanceMax, chunks[2]);

    if !app.accounts.is_empty() {
        draw_results_table(f, app, chunks[3]);
    } else {
        let msg = Paragraph::new("  Enter range and press Enter to search")
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        f.render_widget(msg, chunks[3]);
    }

    let help = Paragraph::new(" Tab: Switch field | Enter: Search | Up/Down: Navigate | Space: Select | Esc: Back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[4]);
}

fn draw_results_table(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["Name", "MSISDN", "Balance", "Status"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .accounts
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let style = if i == app.selected_index {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            Row::new(vec![
                Cell::from(a.name.clone()),
                Cell::from(a.msisdn.clone()),
                Cell::from(format!("${:.2}", a.balance)),
                Cell::from(a.status.as_str()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(20),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} result(s) ", app.accounts.len())),
    );
    f.render_widget(table, area);
}

fn draw_credit_debit(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);

    let action = if app.is_credit { "Credit" } else { "Debit" };
    let color = if app.is_credit { Color::Green } else { Color::Red };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let title = Paragraph::new(format!("  {} Account", action))
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    let amount = Paragraph::new(format!("  ${}", app.input_amount))
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Amount ")
                .border_style(Style::default().fg(Color::Yellow)),
        );
    f.render_widget(amount, chunks[1]);

    let help = Paragraph::new(" Enter: Apply | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

fn draw_confirm(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let msg = Paragraph::new(format!("  {}", app.confirm_action))
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title(" Confirm "))
        .wrap(Wrap { trim: false });
    f.render_widget(msg, chunks[0]);

    let buttons = Paragraph::new("  [y] Yes   [n] No")
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(buttons, chunks[1]);
}

// --- CALEA Screens ---

fn draw_calea_list(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let title = Paragraph::new("  CALEA Lawful Intercept Targets")
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    if app.intercepts.is_empty() {
        let empty = Paragraph::new("  No intercept targets configured.")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(empty, chunks[1]);
    } else {
        let header_cells = ["Warrant", "Type", "Dest IP:Port", "Status"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1);

        let rows: Vec<Row> = app
            .intercepts
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let style = if i == app.selected_index {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                };
                let status_color = match t.status {
                    ss7_billing::InterceptStatus::Active => Color::Green,
                    ss7_billing::InterceptStatus::Inactive => Color::DarkGray,
                };
                Row::new(vec![
                    Cell::from(t.warrant_id.clone()),
                    Cell::from(t.intercept_type.as_str()),
                    Cell::from(format!("{}:{}", t.dest_ip, t.dest_port)),
                    Cell::from(t.status.as_str()).style(Style::default().fg(status_color)),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(30),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} intercept(s) ", app.intercepts.len())),
        );
        f.render_widget(table, chunks[1]);
    }

    let help = Paragraph::new(" Up/Down: Navigate | Enter: View Details | Esc: Back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

fn draw_calea_create(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(f.area());

    let acct_name = app
        .selected_account
        .as_ref()
        .map(|a| format!("{} ({})", a.name, a.msisdn))
        .unwrap_or_default();

    let title = Paragraph::new(format!("  New CALEA Intercept for: {}", acct_name))
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    draw_input_field(
        f,
        " Warrant/Authorization ID ",
        &app.input_warrant_id,
        app.active_field == InputField::CaleaWarrantId,
        chunks[1],
    );

    draw_input_field(
        f,
        " Destination IP (Mediation Device) ",
        &app.input_dest_ip,
        app.active_field == InputField::CaleaDestIp,
        chunks[2],
    );

    draw_input_field(
        f,
        " Destination Port ",
        &app.input_dest_port,
        app.active_field == InputField::CaleaDestPort,
        chunks[3],
    );

    let type_active = app.active_field == InputField::CaleaType;
    let type_style = if type_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let type_label = CALEA_TYPES[app.calea_type_index].label();
    let type_widget = Paragraph::new(format!("  {} (press any key to cycle)", type_label))
        .style(type_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Intercept Type ")
                .border_style(type_style),
        );
    f.render_widget(type_widget, chunks[4]);

    let info = Paragraph::new(
        "  Packets matching this subscriber will be mirrored to the\n  specified mediation device via the SS7 signaling layer.",
    )
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL).title(" Info "));
    f.render_widget(info, chunks[5]);

    let help = Paragraph::new(" Tab: Next Field | Enter: Create | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[6]);
}

fn draw_calea_detail(f: &mut Frame, app: &App) {
    if let Some(ref intercept) = app.selected_intercept {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(14),
                Constraint::Length(6),
                Constraint::Min(0),
            ])
            .split(f.area());

        let title = Paragraph::new(format!("  CALEA Intercept: {}", intercept.warrant_id))
            .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let status_color = match intercept.status {
            ss7_billing::InterceptStatus::Active => Color::Green,
            ss7_billing::InterceptStatus::Inactive => Color::DarkGray,
        };

        let details = vec![
            Line::from(vec![
                Span::raw("  ID:            "),
                Span::styled(&intercept.id, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("  Warrant:       "),
                Span::styled(&intercept.warrant_id, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("  Account ID:    "),
                Span::styled(&intercept.account_id, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("  Type:          "),
                Span::styled(
                    intercept.intercept_type.label(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Dest IP:       "),
                Span::styled(
                    &intercept.dest_ip,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Dest Port:     "),
                Span::styled(
                    format!("{}", intercept.dest_port),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Status:        "),
                Span::styled(
                    intercept.status.as_str(),
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Created:       "),
                Span::styled(
                    intercept.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  SS7 Action:    "),
                Span::styled(
                    format!(
                        "MAP-TRACE subscriber={} -> {}:{}",
                        intercept.account_id, intercept.dest_ip, intercept.dest_port
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
        ];
        let detail_widget = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title(" Intercept Details "));
        f.render_widget(detail_widget, chunks[1]);

        let actions = vec![
            ListItem::new("  [t] Toggle Active/Inactive"),
            ListItem::new("  [d] Delete Intercept"),
        ];
        let actions_list = List::new(actions)
            .block(Block::default().borders(Borders::ALL).title(" Actions "))
            .style(Style::default().fg(Color::White));
        f.render_widget(actions_list, chunks[2]);

        let help = Paragraph::new(" Esc: Back to list")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(help, chunks[3]);
    }
}

// --- PBX Screens ---

fn draw_pbx_list(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let title = Paragraph::new("  PBX Call Forwarding Rules (SS7 MAP)")
        .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    if app.forwarding_rules.is_empty() {
        let empty = Paragraph::new("  No forwarding rules configured.")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(empty, chunks[1]);
    } else {
        let header_cells = ["From", "To", "Type", "Status"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1);

        let rows: Vec<Row> = app
            .forwarding_rules
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let style = if i == app.selected_index {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                };
                let status_color = match r.status {
                    ss7_billing::ForwardingStatus::Active => Color::Green,
                    ss7_billing::ForwardingStatus::Inactive => Color::DarkGray,
                };
                Row::new(vec![
                    Cell::from(r.source_msisdn.clone()),
                    Cell::from(r.dest_address.clone()),
                    Cell::from(r.forwarding_type.as_str()),
                    Cell::from(r.status.as_str()).style(Style::default().fg(status_color)),
                ])
                .style(style)
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(25),
                Constraint::Percentage(30),
                Constraint::Percentage(25),
                Constraint::Percentage(20),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} rule(s) ", app.forwarding_rules.len())),
        );
        f.render_widget(table, chunks[1]);
    }

    let help = Paragraph::new(" Up/Down: Navigate | Enter: View Details | Esc: Back")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}

fn draw_pbx_create(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(f.area());

    let acct_name = app
        .selected_account
        .as_ref()
        .map(|a| format!("{} ({})", a.name, a.msisdn))
        .unwrap_or_default();

    let title = Paragraph::new(format!("  New PBX Forwarding Rule for: {}", acct_name))
        .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    draw_input_field(
        f,
        " Destination (MSISDN or SIP URI) ",
        &app.input_pbx_dest,
        app.active_field == InputField::PbxDestAddress,
        chunks[1],
    );

    let type_active = app.active_field == InputField::PbxForwardType;
    let type_style = if type_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let type_label = PBX_TYPES[app.pbx_type_index].label();
    let type_widget = Paragraph::new(format!("  {} (press any key to cycle)", type_label))
        .style(type_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Forwarding Type ")
                .border_style(type_style),
        );
    f.render_widget(type_widget, chunks[2]);

    draw_input_field(
        f,
        " No-Answer Timeout (seconds) ",
        &app.input_pbx_timeout,
        app.active_field == InputField::PbxTimeout,
        chunks[3],
    );

    let ss7_op = PBX_TYPES[app.pbx_type_index].ss7_opcode();
    let dest_preview = if app.input_pbx_dest.is_empty() {
        "<destination>".to_string()
    } else {
        app.input_pbx_dest.clone()
    };
    let info = Paragraph::new(vec![
        Line::from(format!(
            "  SS7 MAP: {} ForwardedToNumber={} BasicService=allServices",
            ss7_op, dest_preview
        )),
    ])
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL).title(" SS7 Operation Preview "));
    f.render_widget(info, chunks[4]);

    let help = Paragraph::new(" Tab: Next Field | Enter: Create | Esc: Cancel")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[5]);
}

fn draw_pbx_detail(f: &mut Frame, app: &App) {
    if let Some(ref rule) = app.selected_forwarding {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(14),
                Constraint::Length(6),
                Constraint::Min(0),
            ])
            .split(f.area());

        let title = Paragraph::new(format!(
            "  PBX Forward: {} -> {}",
            rule.source_msisdn, rule.dest_address
        ))
        .style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let status_color = match rule.status {
            ss7_billing::ForwardingStatus::Active => Color::Green,
            ss7_billing::ForwardingStatus::Inactive => Color::DarkGray,
        };

        let details = vec![
            Line::from(vec![
                Span::raw("  ID:            "),
                Span::styled(&rule.id, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("  Source:        "),
                Span::styled(&rule.source_msisdn, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::raw("  Destination:   "),
                Span::styled(
                    &rule.dest_address,
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Type:          "),
                Span::styled(
                    rule.forwarding_type.label(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw("  NA Timeout:    "),
                Span::styled(
                    format!("{}s", rule.no_answer_timeout),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Status:        "),
                Span::styled(
                    rule.status.as_str(),
                    Style::default().fg(status_color).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Created:       "),
                Span::styled(
                    rule.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  SS7 MAP:       "),
                Span::styled(
                    rule.ss7_map_operation(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
        ];
        let detail_widget = Paragraph::new(details)
            .block(Block::default().borders(Borders::ALL).title(" Forwarding Rule Details "));
        f.render_widget(detail_widget, chunks[1]);

        let actions = vec![
            ListItem::new("  [t] Toggle Active/Inactive"),
            ListItem::new("  [d] Delete Rule"),
        ];
        let actions_list = List::new(actions)
            .block(Block::default().borders(Borders::ALL).title(" Actions "))
            .style(Style::default().fg(Color::White));
        f.render_widget(actions_list, chunks[2]);

        let help = Paragraph::new(" Esc: Back to list")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(help, chunks[3]);
    }
}

// --- Helpers ---

fn draw_input_field(f: &mut Frame, title: &str, value: &str, active: bool, area: Rect) {
    let style = if active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let widget = Paragraph::new(format!("  {}", value))
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(style),
        );
    f.render_widget(widget, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
