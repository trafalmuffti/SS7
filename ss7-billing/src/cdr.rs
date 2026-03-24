use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallType {
    MobileOriginating,
    MobileTerminating,
    SMSOriginating,
    SMSTerminating,
    Data,
    USSD,
}

impl CallType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CallType::MobileOriginating => "MO_CALL",
            CallType::MobileTerminating => "MT_CALL",
            CallType::SMSOriginating => "MO_SMS",
            CallType::SMSTerminating => "MT_SMS",
            CallType::Data => "DATA",
            CallType::USSD => "USSD",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "MT_CALL" => CallType::MobileTerminating,
            "MO_SMS" => CallType::SMSOriginating,
            "MT_SMS" => CallType::SMSTerminating,
            "DATA" => CallType::Data,
            "USSD" => CallType::USSD,
            _ => CallType::MobileOriginating,
        }
    }
}

impl std::fmt::Display for CallType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallDetailRecord {
    pub id: String,
    pub account_id: String,
    pub call_type: CallType,
    pub calling_party: String,
    pub called_party: String,
    pub start_time: DateTime<Utc>,
    pub duration_seconds: u32,
    pub charge: f64,
    pub msc_address: String,
    pub cell_id: String,
}

impl CallDetailRecord {
    pub fn new(
        account_id: String,
        call_type: CallType,
        calling_party: String,
        called_party: String,
        duration_seconds: u32,
        msc_address: String,
        cell_id: String,
    ) -> Self {
        CallDetailRecord {
            id: uuid::Uuid::new_v4().to_string(),
            account_id,
            call_type,
            calling_party,
            called_party,
            start_time: Utc::now(),
            duration_seconds,
            charge: 0.0,
            msc_address,
            cell_id,
        }
    }
}
