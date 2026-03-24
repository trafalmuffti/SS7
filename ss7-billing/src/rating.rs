use crate::cdr::{CallDetailRecord, CallType};

pub struct RatingEngine {
    pub mo_call_per_minute: f64,
    pub mt_call_per_minute: f64,
    pub mo_sms_flat: f64,
    pub mt_sms_flat: f64,
    pub data_per_mb: f64,
    pub ussd_flat: f64,
}

impl Default for RatingEngine {
    fn default() -> Self {
        RatingEngine {
            mo_call_per_minute: 0.05,
            mt_call_per_minute: 0.00,
            mo_sms_flat: 0.02,
            mt_sms_flat: 0.00,
            data_per_mb: 0.01,
            ussd_flat: 0.005,
        }
    }
}

impl RatingEngine {
    pub fn rate(&self, cdr: &mut CallDetailRecord) {
        let charge = match cdr.call_type {
            CallType::MobileOriginating => {
                let minutes = (cdr.duration_seconds as f64 / 60.0).ceil();
                minutes * self.mo_call_per_minute
            }
            CallType::MobileTerminating => {
                let minutes = (cdr.duration_seconds as f64 / 60.0).ceil();
                minutes * self.mt_call_per_minute
            }
            CallType::SMSOriginating => self.mo_sms_flat,
            CallType::SMSTerminating => self.mt_sms_flat,
            CallType::Data => {
                let mb = cdr.duration_seconds as f64 / 1024.0;
                mb * self.data_per_mb
            }
            CallType::USSD => self.ussd_flat,
        };
        cdr.charge = (charge * 10000.0).round() / 10000.0;
    }
}
