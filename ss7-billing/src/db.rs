use chrono::Utc;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

use crate::account::{Account, AccountStatus};
use crate::calea::{InterceptStatus, InterceptTarget, InterceptType};
use crate::cdr::{CallDetailRecord, CallType};
use crate::error::{BillingError, Result};
use crate::pbx::{ForwardingRule, ForwardingStatus, ForwardingType};

#[derive(Clone)]
pub struct BillingDb {
    conn: Arc<Mutex<Connection>>,
}

impl BillingDb {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = BillingDb {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = BillingDb {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                msisdn TEXT NOT NULL UNIQUE,
                imsi TEXT NOT NULL UNIQUE,
                balance REAL NOT NULL DEFAULT 0.0,
                currency TEXT NOT NULL DEFAULT 'USD',
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_accounts_name ON accounts(name);
            CREATE INDEX IF NOT EXISTS idx_accounts_msisdn ON accounts(msisdn);
            CREATE INDEX IF NOT EXISTS idx_accounts_balance ON accounts(balance);

            CREATE TABLE IF NOT EXISTS cdrs (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                call_type TEXT NOT NULL,
                calling_party TEXT NOT NULL,
                called_party TEXT NOT NULL,
                start_time TEXT NOT NULL,
                duration_seconds INTEGER NOT NULL,
                charge REAL NOT NULL,
                msc_address TEXT NOT NULL,
                cell_id TEXT NOT NULL,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            );

            CREATE INDEX IF NOT EXISTS idx_cdrs_account ON cdrs(account_id);
            CREATE INDEX IF NOT EXISTS idx_cdrs_time ON cdrs(start_time);

            CREATE TABLE IF NOT EXISTS calea_intercepts (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                warrant_id TEXT NOT NULL,
                intercept_type TEXT NOT NULL DEFAULT 'full',
                dest_ip TEXT NOT NULL,
                dest_port INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (account_id) REFERENCES accounts(id)
            );

            CREATE INDEX IF NOT EXISTS idx_calea_account ON calea_intercepts(account_id);
            CREATE INDEX IF NOT EXISTS idx_calea_status ON calea_intercepts(status);

            CREATE TABLE IF NOT EXISTS pbx_forwarding (
                id TEXT PRIMARY KEY,
                source_account_id TEXT NOT NULL,
                source_msisdn TEXT NOT NULL,
                dest_address TEXT NOT NULL,
                forwarding_type TEXT NOT NULL DEFAULT 'unconditional',
                no_answer_timeout INTEGER NOT NULL DEFAULT 20,
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (source_account_id) REFERENCES accounts(id)
            );

            CREATE INDEX IF NOT EXISTS idx_pbx_source ON pbx_forwarding(source_account_id);
            CREATE INDEX IF NOT EXISTS idx_pbx_source_msisdn ON pbx_forwarding(source_msisdn);
            CREATE INDEX IF NOT EXISTS idx_pbx_status ON pbx_forwarding(status);",
        )?;
        Ok(())
    }

    pub fn create_account(&self, account: &Account) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO accounts (id, name, msisdn, imsi, balance, currency, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                account.id,
                account.name,
                account.msisdn,
                account.imsi,
                account.balance,
                account.currency,
                account.status.as_str(),
                account.created_at.to_rfc3339(),
                account.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(ref err, _)
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                BillingError::DuplicateAccount(account.msisdn.clone())
            }
            _ => BillingError::Database(e),
        })?;
        Ok(())
    }

    pub fn get_account(&self, id: &str) -> Result<Account> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, msisdn, imsi, balance, currency, status, created_at, updated_at
             FROM accounts WHERE id = ?1",
        )?;
        Ok(stmt.query_row(params![id], |row| Ok(Self::row_to_account(row)))
            .map_err(|_| BillingError::AccountNotFound(id.to_string()))?)
    }

    pub fn get_account_by_msisdn(&self, msisdn: &str) -> Result<Account> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, msisdn, imsi, balance, currency, status, created_at, updated_at
             FROM accounts WHERE msisdn = ?1",
        )?;
        Ok(stmt.query_row(params![msisdn], |row| Ok(Self::row_to_account(row)))
            .map_err(|_| BillingError::AccountNotFound(msisdn.to_string()))?)
    }

    pub fn search_accounts_by_name(&self, query: &str) -> Result<Vec<Account>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, msisdn, imsi, balance, currency, status, created_at, updated_at
             FROM accounts WHERE name LIKE ?1 ORDER BY name",
        )?;
        let pattern = format!("%{}%", query);
        let rows = stmt
            .query_map(params![pattern], |row| Ok(Self::row_to_account(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn search_accounts_by_balance(
        &self,
        min_balance: f64,
        max_balance: f64,
    ) -> Result<Vec<Account>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, msisdn, imsi, balance, currency, status, created_at, updated_at
             FROM accounts WHERE balance >= ?1 AND balance <= ?2 ORDER BY balance DESC",
        )?;
        let rows = stmt
            .query_map(params![min_balance, max_balance], |row| {
                Ok(Self::row_to_account(row))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn list_accounts(&self) -> Result<Vec<Account>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, msisdn, imsi, balance, currency, status, created_at, updated_at
             FROM accounts ORDER BY name",
        )?;
        let rows = stmt
            .query_map([], |row| Ok(Self::row_to_account(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn update_balance(&self, id: &str, new_balance: f64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE accounts SET balance = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_balance, Utc::now().to_rfc3339(), id],
        )?;
        if updated == 0 {
            return Err(BillingError::AccountNotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn credit(&self, id: &str, amount: f64) -> Result<f64> {
        let account = self.get_account(id)?;
        if account.status == AccountStatus::Closed {
            return Err(BillingError::AccountSuspended(id.to_string()));
        }
        let new_balance = account.balance + amount;
        self.update_balance(id, new_balance)?;
        Ok(new_balance)
    }

    pub fn debit(&self, id: &str, amount: f64) -> Result<f64> {
        let account = self.get_account(id)?;
        if account.status != AccountStatus::Active {
            return Err(BillingError::AccountSuspended(id.to_string()));
        }
        if account.balance < amount {
            return Err(BillingError::InsufficientBalance {
                available: account.balance,
                required: amount,
            });
        }
        let new_balance = account.balance - amount;
        self.update_balance(id, new_balance)?;
        Ok(new_balance)
    }

    pub fn update_status(&self, id: &str, status: AccountStatus) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE accounts SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), Utc::now().to_rfc3339(), id],
        )?;
        if updated == 0 {
            return Err(BillingError::AccountNotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn delete_account(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM cdrs WHERE account_id = ?1", params![id])?;
        let deleted = conn.execute("DELETE FROM accounts WHERE id = ?1", params![id])?;
        if deleted == 0 {
            return Err(BillingError::AccountNotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn insert_cdr(&self, cdr: &CallDetailRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO cdrs (id, account_id, call_type, calling_party, called_party, start_time, duration_seconds, charge, msc_address, cell_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                cdr.id,
                cdr.account_id,
                cdr.call_type.as_str(),
                cdr.calling_party,
                cdr.called_party,
                cdr.start_time.to_rfc3339(),
                cdr.duration_seconds,
                cdr.charge,
                cdr.msc_address,
                cdr.cell_id,
            ],
        )?;
        Ok(())
    }

    pub fn get_cdrs_for_account(&self, account_id: &str) -> Result<Vec<CallDetailRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, account_id, call_type, calling_party, called_party, start_time, duration_seconds, charge, msc_address, cell_id
             FROM cdrs WHERE account_id = ?1 ORDER BY start_time DESC",
        )?;
        let rows = stmt
            .query_map(params![account_id], |row| {
                Ok(CallDetailRecord {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    call_type: CallType::from_str(&row.get::<_, String>(2)?),
                    calling_party: row.get(3)?,
                    called_party: row.get(4)?,
                    start_time: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    duration_seconds: row.get(6)?,
                    charge: row.get(7)?,
                    msc_address: row.get(8)?,
                    cell_id: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_account_count(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn get_total_balance(&self) -> Result<f64> {
        let conn = self.conn.lock().unwrap();
        let total: f64 = conn.query_row(
            "SELECT COALESCE(SUM(balance), 0.0) FROM accounts",
            [],
            |row| row.get(0),
        )?;
        Ok(total)
    }

    // --- CALEA Intercept Methods ---

    pub fn create_intercept(&self, target: &InterceptTarget) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO calea_intercepts (id, account_id, warrant_id, intercept_type, dest_ip, dest_port, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                target.id,
                target.account_id,
                target.warrant_id,
                target.intercept_type.as_str(),
                target.dest_ip,
                target.dest_port,
                target.status.as_str(),
                target.created_at.to_rfc3339(),
                target.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_intercepts(&self) -> Result<Vec<InterceptTarget>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, account_id, warrant_id, intercept_type, dest_ip, dest_port, status, created_at, updated_at
             FROM calea_intercepts ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| Ok(Self::row_to_intercept(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_intercepts_for_account(&self, account_id: &str) -> Result<Vec<InterceptTarget>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, account_id, warrant_id, intercept_type, dest_ip, dest_port, status, created_at, updated_at
             FROM calea_intercepts WHERE account_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map(params![account_id], |row| Ok(Self::row_to_intercept(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_intercept(&self, id: &str) -> Result<InterceptTarget> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, account_id, warrant_id, intercept_type, dest_ip, dest_port, status, created_at, updated_at
             FROM calea_intercepts WHERE id = ?1",
        )?;
        Ok(stmt
            .query_row(params![id], |row| Ok(Self::row_to_intercept(row)))
            .map_err(|_| BillingError::AccountNotFound(format!("intercept {}", id)))?)
    }

    pub fn update_intercept_status(&self, id: &str, status: InterceptStatus) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE calea_intercepts SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), Utc::now().to_rfc3339(), id],
        )?;
        if updated == 0 {
            return Err(BillingError::AccountNotFound(format!("intercept {}", id)));
        }
        Ok(())
    }

    pub fn delete_intercept(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute("DELETE FROM calea_intercepts WHERE id = ?1", params![id])?;
        if deleted == 0 {
            return Err(BillingError::AccountNotFound(format!("intercept {}", id)));
        }
        Ok(())
    }

    pub fn get_intercept_count(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM calea_intercepts WHERE status = 'active'",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    fn row_to_intercept(row: &rusqlite::Row) -> InterceptTarget {
        InterceptTarget {
            id: row.get(0).unwrap(),
            account_id: row.get(1).unwrap(),
            warrant_id: row.get(2).unwrap(),
            intercept_type: InterceptType::from_str(&row.get::<_, String>(3).unwrap()),
            dest_ip: row.get(4).unwrap(),
            dest_port: row.get::<_, u32>(5).unwrap() as u16,
            status: InterceptStatus::from_str(&row.get::<_, String>(6).unwrap()),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7).unwrap())
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8).unwrap())
                .unwrap()
                .with_timezone(&chrono::Utc),
        }
    }

    // --- PBX Forwarding Methods ---

    pub fn create_forwarding_rule(&self, rule: &ForwardingRule) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO pbx_forwarding (id, source_account_id, source_msisdn, dest_address, forwarding_type, no_answer_timeout, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                rule.id,
                rule.source_account_id,
                rule.source_msisdn,
                rule.dest_address,
                rule.forwarding_type.as_str(),
                rule.no_answer_timeout,
                rule.status.as_str(),
                rule.created_at.to_rfc3339(),
                rule.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_forwarding_rules(&self) -> Result<Vec<ForwardingRule>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_account_id, source_msisdn, dest_address, forwarding_type, no_answer_timeout, status, created_at, updated_at
             FROM pbx_forwarding ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| Ok(Self::row_to_forwarding(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_forwarding_rules_for_account(&self, account_id: &str) -> Result<Vec<ForwardingRule>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_account_id, source_msisdn, dest_address, forwarding_type, no_answer_timeout, status, created_at, updated_at
             FROM pbx_forwarding WHERE source_account_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map(params![account_id], |row| Ok(Self::row_to_forwarding(row)))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_forwarding_rule(&self, id: &str) -> Result<ForwardingRule> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_account_id, source_msisdn, dest_address, forwarding_type, no_answer_timeout, status, created_at, updated_at
             FROM pbx_forwarding WHERE id = ?1",
        )?;
        Ok(stmt
            .query_row(params![id], |row| Ok(Self::row_to_forwarding(row)))
            .map_err(|_| BillingError::AccountNotFound(format!("forwarding rule {}", id)))?)
    }

    pub fn update_forwarding_status(&self, id: &str, status: ForwardingStatus) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE pbx_forwarding SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), Utc::now().to_rfc3339(), id],
        )?;
        if updated == 0 {
            return Err(BillingError::AccountNotFound(format!("forwarding rule {}", id)));
        }
        Ok(())
    }

    pub fn delete_forwarding_rule(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute("DELETE FROM pbx_forwarding WHERE id = ?1", params![id])?;
        if deleted == 0 {
            return Err(BillingError::AccountNotFound(format!("forwarding rule {}", id)));
        }
        Ok(())
    }

    pub fn get_forwarding_count(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pbx_forwarding WHERE status = 'active'",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    fn row_to_forwarding(row: &rusqlite::Row) -> ForwardingRule {
        ForwardingRule {
            id: row.get(0).unwrap(),
            source_account_id: row.get(1).unwrap(),
            source_msisdn: row.get(2).unwrap(),
            dest_address: row.get(3).unwrap(),
            forwarding_type: ForwardingType::from_str(&row.get::<_, String>(4).unwrap()),
            no_answer_timeout: row.get(5).unwrap(),
            status: ForwardingStatus::from_str(&row.get::<_, String>(6).unwrap()),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7).unwrap())
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8).unwrap())
                .unwrap()
                .with_timezone(&chrono::Utc),
        }
    }

    fn row_to_account(row: &rusqlite::Row) -> Account {
        Account {
            id: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
            msisdn: row.get(2).unwrap(),
            imsi: row.get(3).unwrap(),
            balance: row.get(4).unwrap(),
            currency: row.get(5).unwrap(),
            status: AccountStatus::from_str(&row.get::<_, String>(6).unwrap()),
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7).unwrap())
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8).unwrap())
                .unwrap()
                .with_timezone(&chrono::Utc),
        }
    }
}
