use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::{App, InputField, Screen};

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
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Min(0),
        ])
        .split(f.area());

    // Header
    let header_text = vec![
        Line::from(vec![
            Span::styled(
                "  SS7 BILLING SYSTEM  ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Total Accounts: "),
            Span::styled(
                format!("{}", app.account_count),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  Total Balance:  "),
            Span::styled(
                format!("${:.2}", app.total_balance),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title(" Dashboard "));
    f.render_widget(header, chunks[0]);

    // Menu
    let menu_items = vec![
        ListItem::new("  [1] View All Accounts"),
        ListItem::new("  [2] Create New Account"),
        ListItem::new("  [3] Search by Name"),
        ListItem::new("  [4] Search by Balance Range"),
        ListItem::new("  [q] Quit"),
    ];
    let menu = List::new(menu_items)
        .block(Block::default().borders(Borders::ALL).title(" Menu "))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));
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

    let help = Paragraph::new(" ↑/↓: Navigate | Enter: Select | c: Create | Esc: Back")
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

    let name_style = if app.active_field == InputField::Name {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let name = Paragraph::new(format!("  {}", app.input_name))
        .style(name_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Name ")
                .border_style(name_style),
        );
    f.render_widget(name, chunks[1]);

    let msisdn_style = if app.active_field == InputField::Msisdn {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let msisdn = Paragraph::new(format!("  {}", app.input_msisdn))
        .style(msisdn_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" MSISDN (Phone Number) ")
                .border_style(msisdn_style),
        );
    f.render_widget(msisdn, chunks[2]);

    let imsi_style = if app.active_field == InputField::Imsi {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let imsi = Paragraph::new(format!("  {}", app.input_imsi))
        .style(imsi_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" IMSI ")
                .border_style(imsi_style),
        );
    f.render_widget(imsi, chunks[3]);

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
                Constraint::Length(8),
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
            ListItem::new("  [+] Credit Balance"),
            ListItem::new("  [-] Debit Balance"),
            ListItem::new("  [s] Suspend Account"),
            ListItem::new("  [a] Activate Account"),
            ListItem::new("  [x] Close Account"),
            ListItem::new("  [d] Delete Account"),
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

    let help = Paragraph::new(" Enter: Search | ↑/↓: Navigate results | Space: Select | Esc: Back")
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

    let min_style = if app.active_field == InputField::BalanceMin {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let min_input = Paragraph::new(format!("  {}", app.input_balance_min))
        .style(min_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Min Balance ($) ")
                .border_style(min_style),
        );
    f.render_widget(min_input, chunks[1]);

    let max_style = if app.active_field == InputField::BalanceMax {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let max_input = Paragraph::new(format!("  {}", app.input_balance_max))
        .style(max_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Max Balance ($) ")
                .border_style(max_style),
        );
    f.render_widget(max_input, chunks[2]);

    if !app.accounts.is_empty() {
        draw_results_table(f, app, chunks[3]);
    } else {
        let msg = Paragraph::new("  Enter range and press Enter to search")
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        f.render_widget(msg, chunks[3]);
    }

    let help = Paragraph::new(" Tab: Switch field | Enter: Search | ↑/↓: Navigate | Space: Select | Esc: Back")
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
