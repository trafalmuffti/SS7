use ss7_billing::{Account, AccountStatus, BillingDb};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Dashboard,
    AccountList,
    CreateAccount,
    AccountDetail,
    SearchByName,
    SearchByBalance,
    CreditDebit,
    Confirm,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputField {
    Name,
    Msisdn,
    Imsi,
    Amount,
    SearchQuery,
    BalanceMin,
    BalanceMax,
}

pub struct App {
    pub db: Arc<BillingDb>,
    pub screen: Screen,
    pub previous_screen: Screen,
    pub accounts: Vec<Account>,
    pub selected_index: usize,
    pub selected_account: Option<Account>,
    pub input_name: String,
    pub input_msisdn: String,
    pub input_imsi: String,
    pub input_amount: String,
    pub input_search: String,
    pub input_balance_min: String,
    pub input_balance_max: String,
    pub active_field: InputField,
    pub is_credit: bool,
    pub status_message: String,
    pub confirm_action: String,
    pub confirm_callback: Option<ConfirmAction>,
    pub account_count: usize,
    pub total_balance: f64,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteAccount(String),
    SuspendAccount(String),
    ActivateAccount(String),
    CloseAccount(String),
}

impl App {
    pub fn new(db: Arc<BillingDb>) -> Self {
        let account_count = db.get_account_count().unwrap_or(0);
        let total_balance = db.get_total_balance().unwrap_or(0.0);

        App {
            db,
            screen: Screen::Dashboard,
            previous_screen: Screen::Dashboard,
            accounts: Vec::new(),
            selected_index: 0,
            selected_account: None,
            input_name: String::new(),
            input_msisdn: String::new(),
            input_imsi: String::new(),
            input_amount: String::new(),
            input_search: String::new(),
            input_balance_min: String::new(),
            input_balance_max: String::new(),
            active_field: InputField::Name,
            is_credit: true,
            status_message: String::new(),
            confirm_action: String::new(),
            confirm_callback: None,
            account_count,
            total_balance,
            scroll_offset: 0,
        }
    }

    pub fn refresh_dashboard(&mut self) {
        self.account_count = self.db.get_account_count().unwrap_or(0);
        self.total_balance = self.db.get_total_balance().unwrap_or(0.0);
    }

    pub fn refresh_accounts(&mut self) {
        self.accounts = self.db.list_accounts().unwrap_or_default();
    }

    pub fn go_to_account_list(&mut self) {
        self.refresh_accounts();
        self.selected_index = 0;
        self.scroll_offset = 0;
        self.screen = Screen::AccountList;
    }

    pub fn go_to_create_account(&mut self) {
        self.input_name.clear();
        self.input_msisdn.clear();
        self.input_imsi.clear();
        self.active_field = InputField::Name;
        self.screen = Screen::CreateAccount;
    }

    pub fn go_to_search_by_name(&mut self) {
        self.input_search.clear();
        self.accounts.clear();
        self.selected_index = 0;
        self.active_field = InputField::SearchQuery;
        self.screen = Screen::SearchByName;
    }

    pub fn go_to_search_by_balance(&mut self) {
        self.input_balance_min.clear();
        self.input_balance_max.clear();
        self.accounts.clear();
        self.selected_index = 0;
        self.active_field = InputField::BalanceMin;
        self.screen = Screen::SearchByBalance;
    }

    pub fn select_account(&mut self) {
        if let Some(account) = self.accounts.get(self.selected_index) {
            self.selected_account = Some(account.clone());
            self.previous_screen = self.screen.clone();
            self.screen = Screen::AccountDetail;
        }
    }

    pub fn create_account(&mut self) {
        if self.input_name.is_empty() || self.input_msisdn.is_empty() || self.input_imsi.is_empty()
        {
            self.status_message = "All fields are required".to_string();
            return;
        }

        let account =
            Account::new(self.input_name.clone(), self.input_msisdn.clone(), self.input_imsi.clone());

        match self.db.create_account(&account) {
            Ok(()) => {
                self.status_message = format!("Account '{}' created successfully", account.name);
                self.refresh_dashboard();
                self.go_to_account_list();
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
            }
        }
    }

    pub fn do_search_by_name(&mut self) {
        match self.db.search_accounts_by_name(&self.input_search) {
            Ok(accounts) => {
                let count = accounts.len();
                self.accounts = accounts;
                self.selected_index = 0;
                self.status_message = format!("Found {} account(s)", count);
            }
            Err(e) => {
                self.status_message = format!("Search error: {}", e);
            }
        }
    }

    pub fn do_search_by_balance(&mut self) {
        let min: f64 = self.input_balance_min.parse().unwrap_or(0.0);
        let max: f64 = self.input_balance_max.parse().unwrap_or(f64::MAX);

        match self.db.search_accounts_by_balance(min, max) {
            Ok(accounts) => {
                let count = accounts.len();
                self.accounts = accounts;
                self.selected_index = 0;
                self.status_message = format!("Found {} account(s) in range ${:.2} - ${:.2}", count, min, max);
            }
            Err(e) => {
                self.status_message = format!("Search error: {}", e);
            }
        }
    }

    pub fn go_to_credit_debit(&mut self, is_credit: bool) {
        self.is_credit = is_credit;
        self.input_amount.clear();
        self.active_field = InputField::Amount;
        self.screen = Screen::CreditDebit;
    }

    pub fn apply_credit_debit(&mut self) {
        let amount: f64 = match self.input_amount.parse() {
            Ok(v) if v > 0.0 => v,
            _ => {
                self.status_message = "Invalid amount".to_string();
                return;
            }
        };

        if let Some(ref account) = self.selected_account {
            let result = if self.is_credit {
                self.db.credit(&account.id, amount)
            } else {
                self.db.debit(&account.id, amount)
            };

            match result {
                Ok(new_balance) => {
                    let action = if self.is_credit { "Credited" } else { "Debited" };
                    self.status_message =
                        format!("{} ${:.2}. New balance: ${:.2}", action, amount, new_balance);
                    // Refresh the selected account
                    if let Ok(updated) = self.db.get_account(&account.id) {
                        self.selected_account = Some(updated);
                    }
                    self.refresh_dashboard();
                    self.screen = Screen::AccountDetail;
                }
                Err(e) => {
                    self.status_message = format!("Error: {}", e);
                }
            }
        }
    }

    pub fn request_delete(&mut self) {
        if let Some(ref account) = self.selected_account {
            self.confirm_action = format!("Delete account '{}'?", account.name);
            self.confirm_callback = Some(ConfirmAction::DeleteAccount(account.id.clone()));
            self.screen = Screen::Confirm;
        }
    }

    pub fn request_status_change(&mut self, status: AccountStatus) {
        if let Some(ref account) = self.selected_account {
            let action = match status {
                AccountStatus::Active => "Activate",
                AccountStatus::Suspended => "Suspend",
                AccountStatus::Closed => "Close",
            };
            self.confirm_action = format!("{} account '{}'?", action, account.name);
            self.confirm_callback = Some(match status {
                AccountStatus::Active => ConfirmAction::ActivateAccount(account.id.clone()),
                AccountStatus::Suspended => ConfirmAction::SuspendAccount(account.id.clone()),
                AccountStatus::Closed => ConfirmAction::CloseAccount(account.id.clone()),
            });
            self.screen = Screen::Confirm;
        }
    }

    pub fn execute_confirm(&mut self) {
        if let Some(action) = self.confirm_callback.take() {
            match action {
                ConfirmAction::DeleteAccount(id) => match self.db.delete_account(&id) {
                    Ok(()) => {
                        self.status_message = "Account deleted".to_string();
                        self.selected_account = None;
                        self.refresh_dashboard();
                        self.go_to_account_list();
                        return;
                    }
                    Err(e) => self.status_message = format!("Error: {}", e),
                },
                ConfirmAction::SuspendAccount(id) => {
                    match self.db.update_status(&id, AccountStatus::Suspended) {
                        Ok(()) => {
                            self.status_message = "Account suspended".to_string();
                            if let Ok(updated) = self.db.get_account(&id) {
                                self.selected_account = Some(updated);
                            }
                        }
                        Err(e) => self.status_message = format!("Error: {}", e),
                    }
                }
                ConfirmAction::ActivateAccount(id) => {
                    match self.db.update_status(&id, AccountStatus::Active) {
                        Ok(()) => {
                            self.status_message = "Account activated".to_string();
                            if let Ok(updated) = self.db.get_account(&id) {
                                self.selected_account = Some(updated);
                            }
                        }
                        Err(e) => self.status_message = format!("Error: {}", e),
                    }
                }
                ConfirmAction::CloseAccount(id) => {
                    match self.db.update_status(&id, AccountStatus::Closed) {
                        Ok(()) => {
                            self.status_message = "Account closed".to_string();
                            if let Ok(updated) = self.db.get_account(&id) {
                                self.selected_account = Some(updated);
                            }
                        }
                        Err(e) => self.status_message = format!("Error: {}", e),
                    }
                }
            }
        }
        self.screen = Screen::AccountDetail;
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.accounts.is_empty() && self.selected_index < self.accounts.len() - 1 {
            self.selected_index += 1;
        }
    }

    pub fn next_field(&mut self) {
        self.active_field = match (&self.screen, &self.active_field) {
            (Screen::CreateAccount, InputField::Name) => InputField::Msisdn,
            (Screen::CreateAccount, InputField::Msisdn) => InputField::Imsi,
            (Screen::CreateAccount, InputField::Imsi) => InputField::Name,
            (Screen::SearchByBalance, InputField::BalanceMin) => InputField::BalanceMax,
            (Screen::SearchByBalance, InputField::BalanceMax) => InputField::BalanceMin,
            _ => self.active_field.clone(),
        };
    }

    pub fn type_char(&mut self, c: char) {
        match self.active_field {
            InputField::Name => self.input_name.push(c),
            InputField::Msisdn => self.input_msisdn.push(c),
            InputField::Imsi => self.input_imsi.push(c),
            InputField::Amount => self.input_amount.push(c),
            InputField::SearchQuery => self.input_search.push(c),
            InputField::BalanceMin => self.input_balance_min.push(c),
            InputField::BalanceMax => self.input_balance_max.push(c),
        }
    }

    pub fn backspace(&mut self) {
        match self.active_field {
            InputField::Name => { self.input_name.pop(); }
            InputField::Msisdn => { self.input_msisdn.pop(); }
            InputField::Imsi => { self.input_imsi.pop(); }
            InputField::Amount => { self.input_amount.pop(); }
            InputField::SearchQuery => { self.input_search.pop(); }
            InputField::BalanceMin => { self.input_balance_min.pop(); }
            InputField::BalanceMax => { self.input_balance_max.pop(); }
        }
    }
}
