use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use russh::server::{Auth, Handler, Msg, Server, Session};
use russh::{Channel, ChannelId, CryptoVec, MethodSet};
use russh_keys::key::KeyPair;
use ss7_billing::BillingDb;
use tokio::sync::Mutex;

use crate::app::{App, Screen};
use crate::ui;

pub async fn run_ssh_server(addr: &str, db: Arc<BillingDb>) -> Result<(), Box<dyn std::error::Error>> {
    let key = KeyPair::generate_ed25519();

    let config = russh::server::Config {
        auth_rejection_time: std::time::Duration::from_secs(1),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keys: vec![key],
        ..Default::default()
    };

    let config = Arc::new(config);
    let mut server = SshServer { db };

    server.run_on_address(config, addr).await?;
    Ok(())
}

struct SshServer {
    db: Arc<BillingDb>,
}

impl Server for SshServer {
    type Handler = SshSession;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        SshSession {
            db: self.db.clone(),
            channels: Arc::new(Mutex::new(HashMap::new())),
            app: Arc::new(Mutex::new(None)),
            terminal_size: Arc::new(Mutex::new((80, 24))),
        }
    }
}

struct SshSession {
    db: Arc<BillingDb>,
    channels: Arc<Mutex<HashMap<ChannelId, Channel<Msg>>>>,
    app: Arc<Mutex<Option<App>>>,
    terminal_size: Arc<Mutex<(u32, u32)>>,
}

impl SshSession {
    async fn render(&self, session: &mut Session) {
        let app_lock = self.app.lock().await;
        let (cols, rows) = *self.terminal_size.lock().await;

        if let Some(ref app) = *app_lock {
            let mut buf = Vec::new();
            let backend = ratatui::backend::CrosstermBackend::new(&mut buf);
            let area = ratatui::layout::Rect::new(0, 0, cols as u16, rows as u16);
            let mut terminal = ratatui::Terminal::with_options(
                backend,
                ratatui::TerminalOptions {
                    viewport: ratatui::Viewport::Fixed(area),
                },
            )
            .unwrap();

            terminal
                .draw(|f| {
                    ui::draw(f, app);
                })
                .unwrap();

            drop(terminal);

            let channels = self.channels.lock().await;
            for (id, _) in channels.iter() {
                let mut output = Vec::new();
                output.extend_from_slice(b"\x1b[2J\x1b[H");
                output.extend_from_slice(&buf);
                session.data(*id, CryptoVec::from_slice(&output));
            }
        }
    }
}

#[async_trait]
impl Handler for SshSession {
    type Error = russh::Error;

    async fn auth_password(
        &mut self,
        user: &str,
        password: &str,
    ) -> Result<Auth, Self::Error> {
        if user == "root" && password == "apple" {
            Ok(Auth::Accept)
        } else {
            Ok(Auth::Reject {
                proceed_with_methods: Some(MethodSet::PASSWORD),
            })
        }
    }

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        let id = channel.id();
        self.channels.lock().await.insert(id, channel);
        Ok(true)
    }

    async fn pty_request(
        &mut self,
        _channel_id: ChannelId,
        _term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        *self.terminal_size.lock().await = (col_width, row_height);

        let app = App::new(self.db.clone());
        *self.app.lock().await = Some(app);

        self.render(session).await;
        Ok(())
    }

    async fn shell_request(
        &mut self,
        _channel_id: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.app.lock().await.is_none() {
            let app = App::new(self.db.clone());
            *self.app.lock().await = Some(app);
        }
        self.render(session).await;
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        _channel_id: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        *self.terminal_size.lock().await = (col_width, row_height);
        self.render(session).await;
        Ok(())
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let mut app_lock = self.app.lock().await;
        let app = match app_lock.as_mut() {
            Some(app) => app,
            None => return Ok(()),
        };

        let mut should_quit = false;

        // Handle escape sequences (arrow keys: ESC [ A/B/C/D) first
        if data.len() >= 3 && data[0] == 27 && data[1] == b'[' {
            match data[2] {
                b'A' => app.move_selection_up(),
                b'B' => app.move_selection_down(),
                _ => {}
            }
        } else {
            for byte in data {
                match *byte {
                    // Ctrl-C or Ctrl-D
                    3 | 4 => {
                        should_quit = true;
                        break;
                    }
                    // ESC (standalone, not part of a sequence)
                    27 => {
                        match app.screen {
                            Screen::Dashboard => {}
                            Screen::Confirm => {
                                app.screen = Screen::AccountDetail;
                            }
                            Screen::CreditDebit => {
                                app.screen = Screen::AccountDetail;
                            }
                            Screen::AccountDetail => {
                                app.screen = app.previous_screen.clone();
                                if app.screen == Screen::AccountList {
                                    app.refresh_accounts();
                                }
                            }
                            Screen::CreateAccount => {
                                app.screen = Screen::Dashboard;
                                app.refresh_dashboard();
                            }
                            _ => {
                                app.screen = Screen::Dashboard;
                                app.refresh_dashboard();
                            }
                        }
                    }
                    // Enter
                    13 => match app.screen {
                        Screen::CreateAccount => app.create_account(),
                        Screen::SearchByName => app.do_search_by_name(),
                        Screen::SearchByBalance => app.do_search_by_balance(),
                        Screen::CreditDebit => app.apply_credit_debit(),
                        Screen::AccountList => {
                            if !app.accounts.is_empty() {
                                app.select_account();
                            }
                        }
                        _ => {}
                    },
                    // Tab
                    9 => app.next_field(),
                    // Backspace / DEL
                    127 | 8 => app.backspace(),
                    // Space
                    b' ' => {
                        match app.screen {
                            Screen::AccountList | Screen::SearchByName | Screen::SearchByBalance => {
                                if !app.accounts.is_empty() {
                                    app.select_account();
                                }
                            }
                            _ => app.type_char(' '),
                        }
                    }
                    // Regular printable characters
                    c if c >= 32 && c < 127 => {
                        let ch = c as char;
                        match app.screen {
                            Screen::Dashboard => match ch {
                                '1' => app.go_to_account_list(),
                                '2' => app.go_to_create_account(),
                                '3' => app.go_to_search_by_name(),
                                '4' => app.go_to_search_by_balance(),
                                'q' | 'Q' => {
                                    should_quit = true;
                                }
                                _ => {}
                            },
                            Screen::AccountList => match ch {
                                'c' | 'C' => app.go_to_create_account(),
                                _ => {}
                            },
                            Screen::AccountDetail => match ch {
                                '+' => app.go_to_credit_debit(true),
                                '-' => app.go_to_credit_debit(false),
                                's' | 'S' => app.request_status_change(ss7_billing::AccountStatus::Suspended),
                                'a' | 'A' => app.request_status_change(ss7_billing::AccountStatus::Active),
                                'x' | 'X' => app.request_status_change(ss7_billing::AccountStatus::Closed),
                                'd' | 'D' => app.request_delete(),
                                _ => {}
                            },
                            Screen::Confirm => match ch {
                                'y' | 'Y' => app.execute_confirm(),
                                'n' | 'N' => {
                                    app.screen = Screen::AccountDetail;
                                }
                                _ => {}
                            },
                            Screen::CreateAccount | Screen::CreditDebit => {
                                app.type_char(ch);
                            }
                            Screen::SearchByName => {
                                app.type_char(ch);
                            }
                            Screen::SearchByBalance => {
                                app.type_char(ch);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        drop(app_lock);

        if should_quit {
            let channels = self.channels.lock().await;
            for (id, _) in channels.iter() {
                session.data(*id, CryptoVec::from_slice(b"\x1b[2J\x1b[H\r\nGoodbye!\r\n"));
                session.close(*id);
            }
            return Ok(());
        }

        self.render(session).await;
        Ok(())
    }
}
