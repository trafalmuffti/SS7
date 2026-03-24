use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterceptStatus {
    Active,
    Inactive,
}

impl InterceptStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InterceptStatus::Active => "active",
            InterceptStatus::Inactive => "inactive",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "inactive" => InterceptStatus::Inactive,
            _ => InterceptStatus::Active,
        }
    }
}

impl std::fmt::Display for InterceptStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterceptType {
    /// Full content intercept (voice + signaling)
    Full,
    /// Signaling/metadata only (call records, location updates)
    SignalingOnly,
    /// SMS content intercept
    Sms,
    /// Data session intercept
    Data,
}

impl InterceptType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InterceptType::Full => "full",
            InterceptType::SignalingOnly => "signaling",
            InterceptType::Sms => "sms",
            InterceptType::Data => "data",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "signaling" => InterceptType::SignalingOnly,
            "sms" => InterceptType::Sms,
            "data" => InterceptType::Data,
            _ => InterceptType::Full,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            InterceptType::Full => "Full (Voice+Signaling)",
            InterceptType::SignalingOnly => "Signaling Only",
            InterceptType::Sms => "SMS Content",
            InterceptType::Data => "Data Sessions",
        }
    }
}

impl std::fmt::Display for InterceptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// A CALEA (Communications Assistance for Law Enforcement Act) intercept target.
/// Configures the SS7 signaling layer to mirror/forward packets for a given
/// account to a designated mediation device at a specific IP address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterceptTarget {
    pub id: String,
    pub account_id: String,
    /// Warrant or authorization reference number
    pub warrant_id: String,
    /// Type of intercept
    pub intercept_type: InterceptType,
    /// Destination IP for the mediation/collection device
    pub dest_ip: String,
    /// Destination port on the mediation device
    pub dest_port: u16,
    pub status: InterceptStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InterceptTarget {
    pub fn new(
        account_id: String,
        warrant_id: String,
        intercept_type: InterceptType,
        dest_ip: String,
        dest_port: u16,
    ) -> Self {
        let now = Utc::now();
        InterceptTarget {
            id: uuid::Uuid::new_v4().to_string(),
            account_id,
            warrant_id,
            intercept_type,
            dest_ip,
            dest_port,
            status: InterceptStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }
}
