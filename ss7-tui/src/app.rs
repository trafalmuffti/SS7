use ss7_billing::{
    Account, AccountStatus, BillingDb, ForwardingRule, ForwardingStatus, ForwardingType,
    InterceptStatus, InterceptTarget, InterceptType,
};
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
    // CALEA screens
    CaleaList,
    CaleaCreate,
    CaleaDetail,
    // PBX screens
    PbxList,
    PbxCreate,
    PbxDetail,
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
    // CALEA fields
    CaleaWarrantId,
    CaleaDestIp,
    CaleaDestPort,
    CaleaType,
    // PBX fields
    PbxDestAddress,
    PbxForwardType,
    PbxTimeout,
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
    // CALEA state
    pub intercepts: Vec<InterceptTarget>,
    pub selected_intercept: Option<InterceptTarget>,
    pub input_warrant_id: String,
    pub input_dest_ip: String,
    pub input_dest_port: String,
    pub calea_type_index: usize,
    pub intercept_count: usize,
    // PBX state
    pub forwarding_rules: Vec<ForwardingRule>,
    pub selected_forwarding: Option<ForwardingRule>,
    pub input_pbx_dest: String,
    pub pbx_type_index: usize,
    pub input_pbx_timeout: String,
    pub forwarding_count: usize,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteAccount(String),
    SuspendAccount(String),
    ActivateAccount(String),
    CloseAccount(String),
    DeleteIntercept(String),
    ToggleIntercept(String, bool),
    DeleteForwarding(String),
    ToggleForwarding(String, bool),
}

pub const CALEA_TYPES: &[InterceptType] = &[
    InterceptType::Full,
    InterceptType::SignalingOnly,
    InterceptType::Sms,
    InterceptType::Data,
];

pub const PBX_TYPES: &[ForwardingType] = &[
    ForwardingType::Unconditional,
    ForwardingType::Busy,
    ForwardingType::NoAnswer,
    ForwardingType::NotReachable,
];

impl App {
    pub fn new(db: Arc<BillingDb>) -> Self {
        let account_count = db.get_account_count().unwrap_or(0);
        let total_balance = db.get_total_balance().unwrap_or(0.0);
        let intercept_count = db.get_intercept_count().unwrap_or(0);
        let forwarding_count = db.get_forwarding_count().unwrap_or(0);

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
            // CALEA
            intercepts: Vec::new(),
            selected_intercept: None,
            input_warrant_id: String::new(),
            input_dest_ip: String::new(),
            input_dest_port: String::new(),
            calea_type_index: 0,
            intercept_count,
            // PBX
            forwarding_rules: Vec::new(),
            selected_forwarding: None,
            input_pbx_dest: String::new(),
            pbx_type_index: 0,
            input_pbx_timeout: "20".to_string(),
            forwarding_count,
        }
    }

    pub fn refresh_dashboard(&mut self) {
        self.account_count = self.db.get_account_count().unwrap_or(0);
        self.total_balance = self.db.get_total_balance().unwrap_or(0.0);
        self.intercept_count = self.db.get_intercept_count().unwrap_or(0);
        self.forwarding_count = self.db.get_forwarding_count().unwrap_or(0);
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
                ConfirmAction::DeleteIntercept(id) => {
                    match self.db.delete_intercept(&id) {
                        Ok(()) => {
                            self.status_message = "Intercept deleted".to_string();
                            self.selected_intercept = None;
                            self.refresh_dashboard();
                            self.go_to_calea_list();
                            return;
                        }
                        Err(e) => self.status_message = format!("Error: {}", e),
                    }
                }
                ConfirmAction::ToggleIntercept(id, activate) => {
                    let status = if activate {
                        InterceptStatus::Active
                    } else {
                        InterceptStatus::Inactive
                    };
                    match self.db.update_intercept_status(&id, status) {
                        Ok(()) => {
                            let word = if activate { "activated" } else { "deactivated" };
                            self.status_message = format!("Intercept {}", word);
                            if let Ok(updated) = self.db.get_intercept(&id) {
                                self.selected_intercept = Some(updated);
                            }
                        }
                        Err(e) => self.status_message = format!("Error: {}", e),
                    }
                }
                ConfirmAction::DeleteForwarding(id) => {
                    match self.db.delete_forwarding_rule(&id) {
                        Ok(()) => {
                            self.status_message = "Forwarding rule deleted".to_string();
                            self.selected_forwarding = None;
                            self.refresh_dashboard();
                            self.go_to_pbx_list();
                            return;
                        }
                        Err(e) => self.status_message = format!("Error: {}", e),
                    }
                }
                ConfirmAction::ToggleForwarding(id, activate) => {
                    let status = if activate {
                        ForwardingStatus::Active
                    } else {
                        ForwardingStatus::Inactive
                    };
                    match self.db.update_forwarding_status(&id, status) {
                        Ok(()) => {
                            let word = if activate { "activated" } else { "deactivated" };
                            self.status_message = format!("Forwarding rule {}", word);
                            if let Ok(updated) = self.db.get_forwarding_rule(&id) {
                                self.selected_forwarding = Some(updated);
                            }
                        }
                        Err(e) => self.status_message = format!("Error: {}", e),
                    }
                }
            }
        }
        // Return to appropriate detail screen
        match self.screen {
            Screen::Confirm => {
                if self.selected_intercept.is_some() && self.selected_account.is_none() {
                    self.screen = Screen::CaleaDetail;
                } else if self.selected_forwarding.is_some() && self.selected_account.is_none() {
                    self.screen = Screen::PbxDetail;
                } else {
                    self.screen = Screen::AccountDetail;
                }
            }
            _ => {}
        }
    }

    // --- CALEA Methods ---

    pub fn go_to_calea_list(&mut self) {
        self.intercepts = self.db.list_intercepts().unwrap_or_default();
        self.selected_index = 0;
        self.selected_account = None;
        self.screen = Screen::CaleaList;
    }

    pub fn go_to_calea_create_for_account(&mut self) {
        self.input_warrant_id.clear();
        self.input_dest_ip.clear();
        self.input_dest_port = "9500".to_string();
        self.calea_type_index = 0;
        self.active_field = InputField::CaleaWarrantId;
        self.screen = Screen::CaleaCreate;
    }

    pub fn create_intercept(&mut self) {
        if self.input_warrant_id.is_empty() || self.input_dest_ip.is_empty() {
            self.status_message = "Warrant ID and destination IP are required".to_string();
            return;
        }

        let port: u16 = match self.input_dest_port.parse() {
            Ok(p) if p > 0 => p,
            _ => {
                self.status_message = "Invalid port number".to_string();
                return;
            }
        };

        if let Some(ref account) = self.selected_account {
            let intercept_type = CALEA_TYPES[self.calea_type_index].clone();
            let target = InterceptTarget::new(
                account.id.clone(),
                self.input_warrant_id.clone(),
                intercept_type,
                self.input_dest_ip.clone(),
                port,
            );

            match self.db.create_intercept(&target) {
                Ok(()) => {
                    self.status_message = format!(
                        "CALEA intercept created: {} -> {}:{}",
                        account.msisdn, target.dest_ip, target.dest_port
                    );
                    self.refresh_dashboard();
                    self.screen = Screen::AccountDetail;
                }
                Err(e) => {
                    self.status_message = format!("Error: {}", e);
                }
            }
        }
    }

    pub fn select_intercept(&mut self) {
        if let Some(intercept) = self.intercepts.get(self.selected_index) {
            self.selected_intercept = Some(intercept.clone());
            self.selected_account = None;
            self.screen = Screen::CaleaDetail;
        }
    }

    pub fn request_delete_intercept(&mut self) {
        if let Some(ref intercept) = self.selected_intercept {
            self.confirm_action = format!("Delete intercept (warrant {})?", intercept.warrant_id);
            self.confirm_callback = Some(ConfirmAction::DeleteIntercept(intercept.id.clone()));
            self.screen = Screen::Confirm;
        }
    }

    pub fn request_toggle_intercept(&mut self) {
        if let Some(ref intercept) = self.selected_intercept {
            let activate = intercept.status == InterceptStatus::Inactive;
            let action = if activate { "Activate" } else { "Deactivate" };
            self.confirm_action = format!("{} intercept (warrant {})?", action, intercept.warrant_id);
            self.confirm_callback = Some(ConfirmAction::ToggleIntercept(intercept.id.clone(), activate));
            self.screen = Screen::Confirm;
        }
    }

    // --- PBX Methods ---

    pub fn go_to_pbx_list(&mut self) {
        self.forwarding_rules = self.db.list_forwarding_rules().unwrap_or_default();
        self.selected_index = 0;
        self.selected_account = None;
        self.screen = Screen::PbxList;
    }

    pub fn go_to_pbx_create_for_account(&mut self) {
        self.input_pbx_dest.clear();
        self.pbx_type_index = 0;
        self.input_pbx_timeout = "20".to_string();
        self.active_field = InputField::PbxDestAddress;
        self.screen = Screen::PbxCreate;
    }

    pub fn create_forwarding_rule(&mut self) {
        if self.input_pbx_dest.is_empty() {
            self.status_message = "Destination address is required".to_string();
            return;
        }

        let timeout: u32 = self.input_pbx_timeout.parse().unwrap_or(20);

        if let Some(ref account) = self.selected_account {
            let fwd_type = PBX_TYPES[self.pbx_type_index].clone();
            let rule = ForwardingRule::new(
                account.id.clone(),
                account.msisdn.clone(),
                self.input_pbx_dest.clone(),
                fwd_type,
                timeout,
            );

            let map_op = rule.ss7_map_operation();
            match self.db.create_forwarding_rule(&rule) {
                Ok(()) => {
                    self.status_message = format!(
                        "PBX rule created: {} -> {} [SS7: {}]",
                        account.msisdn, rule.dest_address, map_op
                    );
                    self.refresh_dashboard();
                    self.screen = Screen::AccountDetail;
                }
                Err(e) => {
                    self.status_message = format!("Error: {}", e);
                }
            }
        }
    }

    pub fn select_forwarding(&mut self) {
        if let Some(rule) = self.forwarding_rules.get(self.selected_index) {
            self.selected_forwarding = Some(rule.clone());
            self.selected_account = None;
            self.screen = Screen::PbxDetail;
        }
    }

    pub fn request_delete_forwarding(&mut self) {
        if let Some(ref rule) = self.selected_forwarding {
            self.confirm_action = format!(
                "Delete forwarding {} -> {}?",
                rule.source_msisdn, rule.dest_address
            );
            self.confirm_callback = Some(ConfirmAction::DeleteForwarding(rule.id.clone()));
            self.screen = Screen::Confirm;
        }
    }

    pub fn request_toggle_forwarding(&mut self) {
        if let Some(ref rule) = self.selected_forwarding {
            let activate = rule.status == ForwardingStatus::Inactive;
            let action = if activate { "Activate" } else { "Deactivate" };
            self.confirm_action = format!(
                "{} forwarding {} -> {}?",
                action, rule.source_msisdn, rule.dest_address
            );
            self.confirm_callback = Some(ConfirmAction::ToggleForwarding(rule.id.clone(), activate));
            self.screen = Screen::Confirm;
        }
    }

    // --- Navigation ---

    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_selection_down(&mut self) {
        let len = match self.screen {
            Screen::CaleaList => self.intercepts.len(),
            Screen::PbxList => self.forwarding_rules.len(),
            _ => self.accounts.len(),
        };
        if len > 0 && self.selected_index < len - 1 {
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
            // CALEA create: cycle through fields
            (Screen::CaleaCreate, InputField::CaleaWarrantId) => InputField::CaleaDestIp,
            (Screen::CaleaCreate, InputField::CaleaDestIp) => InputField::CaleaDestPort,
            (Screen::CaleaCreate, InputField::CaleaDestPort) => InputField::CaleaType,
            (Screen::CaleaCreate, InputField::CaleaType) => InputField::CaleaWarrantId,
            // PBX create: cycle through fields
            (Screen::PbxCreate, InputField::PbxDestAddress) => InputField::PbxForwardType,
            (Screen::PbxCreate, InputField::PbxForwardType) => InputField::PbxTimeout,
            (Screen::PbxCreate, InputField::PbxTimeout) => InputField::PbxDestAddress,
            _ => self.active_field.clone(),
        };
    }

    pub fn cycle_calea_type(&mut self) {
        self.calea_type_index = (self.calea_type_index + 1) % CALEA_TYPES.len();
    }

    pub fn cycle_pbx_type(&mut self) {
        self.pbx_type_index = (self.pbx_type_index + 1) % PBX_TYPES.len();
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
            InputField::CaleaWarrantId => self.input_warrant_id.push(c),
            InputField::CaleaDestIp => self.input_dest_ip.push(c),
            InputField::CaleaDestPort => self.input_dest_port.push(c),
            InputField::CaleaType => self.cycle_calea_type(),
            InputField::PbxDestAddress => self.input_pbx_dest.push(c),
            InputField::PbxForwardType => self.cycle_pbx_type(),
            InputField::PbxTimeout => self.input_pbx_timeout.push(c),
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
            InputField::CaleaWarrantId => { self.input_warrant_id.pop(); }
            InputField::CaleaDestIp => { self.input_dest_ip.pop(); }
            InputField::CaleaDestPort => { self.input_dest_port.pop(); }
            InputField::CaleaType => {}
            InputField::PbxDestAddress => { self.input_pbx_dest.pop(); }
            InputField::PbxForwardType => {}
            InputField::PbxTimeout => { self.input_pbx_timeout.pop(); }
        }
    }
}
