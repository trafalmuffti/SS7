use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountStatus {
    Active,
    Suspended,
    Closed,
}

impl AccountStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountStatus::Active => "active",
            AccountStatus::Suspended => "suspended",
            AccountStatus::Closed => "closed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "suspended" => AccountStatus::Suspended,
            "closed" => AccountStatus::Closed,
            _ => AccountStatus::Active,
        }
    }
}

impl std::fmt::Display for AccountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub msisdn: String,
    pub imsi: String,
    pub balance: f64,
    pub currency: String,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn new(name: String, msisdn: String, imsi: String) -> Self {
        let now = Utc::now();
        Account {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            msisdn,
            imsi,
            balance: 0.0,
            currency: "USD".to_string(),
            status: AccountStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }
}
