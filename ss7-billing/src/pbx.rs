use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ForwardingType {
    /// Always forward (CFU - Call Forward Unconditional)
    Unconditional,
    /// Forward when busy (CFB)
    Busy,
    /// Forward on no answer (CFNA)
    NoAnswer,
    /// Forward when not reachable (CFNRc)
    NotReachable,
}

impl ForwardingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ForwardingType::Unconditional => "unconditional",
            ForwardingType::Busy => "busy",
            ForwardingType::NoAnswer => "no_answer",
            ForwardingType::NotReachable => "not_reachable",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "busy" => ForwardingType::Busy,
            "no_answer" => ForwardingType::NoAnswer,
            "not_reachable" => ForwardingType::NotReachable,
            _ => ForwardingType::Unconditional,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ForwardingType::Unconditional => "Unconditional (CFU)",
            ForwardingType::Busy => "On Busy (CFB)",
            ForwardingType::NoAnswer => "No Answer (CFNA)",
            ForwardingType::NotReachable => "Not Reachable (CFNRc)",
        }
    }

    pub fn ss7_opcode(&self) -> &'static str {
        match self {
            ForwardingType::Unconditional => "RegisterSS(CFU)",
            ForwardingType::Busy => "RegisterSS(CFB)",
            ForwardingType::NoAnswer => "RegisterSS(CFNA)",
            ForwardingType::NotReachable => "RegisterSS(CFNRc)",
        }
    }
}

impl std::fmt::Display for ForwardingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ForwardingStatus {
    Active,
    Inactive,
}

impl ForwardingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ForwardingStatus::Active => "active",
            ForwardingStatus::Inactive => "inactive",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "inactive" => ForwardingStatus::Inactive,
            _ => ForwardingStatus::Active,
        }
    }
}

impl std::fmt::Display for ForwardingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A PBX call forwarding rule. Uses SS7 MAP operations (RegisterSS,
/// ActivateSS, DeactivateSS, EraseSS) to configure call forwarding
/// in the HLR/VLR for a given subscriber.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardingRule {
    pub id: String,
    /// Source account (subscriber whose calls are forwarded)
    pub source_account_id: String,
    /// Source MSISDN
    pub source_msisdn: String,
    /// Destination MSISDN or SIP URI for VoIP endpoint
    pub dest_address: String,
    /// Type of forwarding condition
    pub forwarding_type: ForwardingType,
    /// No-answer timeout in seconds (only relevant for CFNA)
    pub no_answer_timeout: u32,
    pub status: ForwardingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ForwardingRule {
    pub fn new(
        source_account_id: String,
        source_msisdn: String,
        dest_address: String,
        forwarding_type: ForwardingType,
        no_answer_timeout: u32,
    ) -> Self {
        let now = Utc::now();
        ForwardingRule {
            id: uuid::Uuid::new_v4().to_string(),
            source_account_id,
            source_msisdn,
            dest_address,
            forwarding_type,
            no_answer_timeout,
            status: ForwardingStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    /// Generate the SS7 MAP operation string that would be sent to the HLR
    pub fn ss7_map_operation(&self) -> String {
        format!(
            "{} ForwardedToNumber={} BasicService=allServices",
            self.forwarding_type.ss7_opcode(),
            self.dest_address,
        )
    }
}
