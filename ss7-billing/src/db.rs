use chrono::Utc;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

use crate::account::{Account, AccountStatus};
use crate::cdr::{CallDetailRecord, CallType};
use crate::error::{BillingError, Result};

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
            CREATE INDEX IF NOT EXISTS idx_cdrs_time ON cdrs(start_time);",
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
