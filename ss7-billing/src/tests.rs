#[cfg(test)]
mod tests {
    use crate::account::{Account, AccountStatus};
    use crate::calea::{InterceptStatus, InterceptTarget, InterceptType};
    use crate::cdr::{CallDetailRecord, CallType};
    use crate::db::BillingDb;
    use crate::pbx::{ForwardingRule, ForwardingStatus, ForwardingType};
    use crate::rating::RatingEngine;

    fn test_db() -> BillingDb {
        BillingDb::open_in_memory().unwrap()
    }

    fn make_account(name: &str, msisdn: &str, imsi: &str) -> Account {
        Account::new(name.to_string(), msisdn.to_string(), imsi.to_string())
    }

    // --- Account Tests ---

    #[test]
    fn create_and_retrieve_account() {
        let db = test_db();
        let acct = make_account("Alice", "+14155550100", "310260000000001");
        db.create_account(&acct).unwrap();

        let fetched = db.get_account(&acct.id).unwrap();
        assert_eq!(fetched.name, "Alice");
        assert_eq!(fetched.msisdn, "+14155550100");
        assert_eq!(fetched.imsi, "310260000000001");
        assert_eq!(fetched.balance, 0.0);
        assert_eq!(fetched.status, AccountStatus::Active);
    }

    #[test]
    fn get_account_by_msisdn() {
        let db = test_db();
        let acct = make_account("Bob", "+14155550101", "310260000000002");
        db.create_account(&acct).unwrap();

        let fetched = db.get_account_by_msisdn("+14155550101").unwrap();
        assert_eq!(fetched.name, "Bob");
    }

    #[test]
    fn duplicate_msisdn_rejected() {
        let db = test_db();
        let a1 = make_account("Alice", "+14155550100", "310260000000001");
        let a2 = make_account("Bob", "+14155550100", "310260000000099");
        db.create_account(&a1).unwrap();
        let result = db.create_account(&a2);
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_imsi_rejected() {
        let db = test_db();
        let a1 = make_account("Alice", "+14155550100", "310260000000001");
        let a2 = make_account("Bob", "+14155550199", "310260000000001");
        db.create_account(&a1).unwrap();
        let result = db.create_account(&a2);
        assert!(result.is_err());
    }

    #[test]
    fn account_not_found() {
        let db = test_db();
        let result = db.get_account("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn credit_and_debit() {
        let db = test_db();
        let acct = make_account("Alice", "+14155550100", "310260000000001");
        db.create_account(&acct).unwrap();

        let balance = db.credit(&acct.id, 100.0).unwrap();
        assert_eq!(balance, 100.0);

        let balance = db.debit(&acct.id, 30.50).unwrap();
        assert!((balance - 69.50).abs() < 0.001);

        let fetched = db.get_account(&acct.id).unwrap();
        assert!((fetched.balance - 69.50).abs() < 0.001);
    }

    #[test]
    fn debit_insufficient_balance() {
        let db = test_db();
        let acct = make_account("Alice", "+14155550100", "310260000000001");
        db.create_account(&acct).unwrap();
        db.credit(&acct.id, 10.0).unwrap();

        let result = db.debit(&acct.id, 50.0);
        assert!(result.is_err());
    }

    #[test]
    fn debit_suspended_account_fails() {
        let db = test_db();
        let acct = make_account("Alice", "+14155550100", "310260000000001");
        db.create_account(&acct).unwrap();
        db.credit(&acct.id, 100.0).unwrap();
        db.update_status(&acct.id, AccountStatus::Suspended).unwrap();

        let result = db.debit(&acct.id, 10.0);
        assert!(result.is_err());
    }

    #[test]
    fn credit_closed_account_fails() {
        let db = test_db();
        let acct = make_account("Alice", "+14155550100", "310260000000001");
        db.create_account(&acct).unwrap();
        db.update_status(&acct.id, AccountStatus::Closed).unwrap();

        let result = db.credit(&acct.id, 10.0);
        assert!(result.is_err());
    }

    #[test]
    fn search_accounts_by_name() {
        let db = test_db();
        db.create_account(&make_account("Alice Smith", "+1001", "001")).unwrap();
        db.create_account(&make_account("Bob Jones", "+1002", "002")).unwrap();
        db.create_account(&make_account("Alice Johnson", "+1003", "003")).unwrap();

        let results = db.search_accounts_by_name("Alice").unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|a| a.name.contains("Alice")));
    }

    #[test]
    fn search_accounts_by_balance_range() {
        let db = test_db();
        let a1 = make_account("Low", "+1001", "001");
        let a2 = make_account("Mid", "+1002", "002");
        let a3 = make_account("High", "+1003", "003");
        db.create_account(&a1).unwrap();
        db.create_account(&a2).unwrap();
        db.create_account(&a3).unwrap();

        db.credit(&a1.id, 10.0).unwrap();
        db.credit(&a2.id, 50.0).unwrap();
        db.credit(&a3.id, 100.0).unwrap();

        let results = db.search_accounts_by_balance(20.0, 80.0).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Mid");
    }

    #[test]
    fn list_accounts() {
        let db = test_db();
        db.create_account(&make_account("Charlie", "+1001", "001")).unwrap();
        db.create_account(&make_account("Alice", "+1002", "002")).unwrap();
        db.create_account(&make_account("Bob", "+1003", "003")).unwrap();

        let accounts = db.list_accounts().unwrap();
        assert_eq!(accounts.len(), 3);
        // Should be sorted by name
        assert_eq!(accounts[0].name, "Alice");
        assert_eq!(accounts[1].name, "Bob");
        assert_eq!(accounts[2].name, "Charlie");
    }

    #[test]
    fn delete_account() {
        let db = test_db();
        let acct = make_account("Alice", "+14155550100", "310260000000001");
        db.create_account(&acct).unwrap();
        db.delete_account(&acct.id).unwrap();

        let result = db.get_account(&acct.id);
        assert!(result.is_err());
    }

    #[test]
    fn update_account_status() {
        let db = test_db();
        let acct = make_account("Alice", "+14155550100", "310260000000001");
        db.create_account(&acct).unwrap();

        db.update_status(&acct.id, AccountStatus::Suspended).unwrap();
        let fetched = db.get_account(&acct.id).unwrap();
        assert_eq!(fetched.status, AccountStatus::Suspended);

        db.update_status(&acct.id, AccountStatus::Active).unwrap();
        let fetched = db.get_account(&acct.id).unwrap();
        assert_eq!(fetched.status, AccountStatus::Active);
    }

    #[test]
    fn account_count_and_total_balance() {
        let db = test_db();
        assert_eq!(db.get_account_count().unwrap(), 0);
        assert_eq!(db.get_total_balance().unwrap(), 0.0);

        let a1 = make_account("A", "+1", "1");
        let a2 = make_account("B", "+2", "2");
        db.create_account(&a1).unwrap();
        db.create_account(&a2).unwrap();
        db.credit(&a1.id, 100.0).unwrap();
        db.credit(&a2.id, 250.0).unwrap();

        assert_eq!(db.get_account_count().unwrap(), 2);
        assert!((db.get_total_balance().unwrap() - 350.0).abs() < 0.001);
    }

    // --- Rating Engine Tests ---

    #[test]
    fn rate_mo_call() {
        let engine = RatingEngine::default();
        let mut cdr = CallDetailRecord::new(
            "acct1".into(),
            CallType::MobileOriginating,
            "+1001".into(),
            "+1002".into(),
            90, // 1.5 minutes -> rounds to 2 minutes
            "MSC001".into(),
            "CELL01".into(),
        );
        engine.rate(&mut cdr);
        assert!((cdr.charge - 0.10).abs() < 0.001); // 2 * 0.05
    }

    #[test]
    fn rate_mt_call_free() {
        let engine = RatingEngine::default();
        let mut cdr = CallDetailRecord::new(
            "acct1".into(),
            CallType::MobileTerminating,
            "+1001".into(),
            "+1002".into(),
            300,
            "MSC001".into(),
            "CELL01".into(),
        );
        engine.rate(&mut cdr);
        assert_eq!(cdr.charge, 0.0);
    }

    #[test]
    fn rate_sms() {
        let engine = RatingEngine::default();
        let mut cdr = CallDetailRecord::new(
            "acct1".into(),
            CallType::SMSOriginating,
            "+1001".into(),
            "+1002".into(),
            0,
            "MSC001".into(),
            "CELL01".into(),
        );
        engine.rate(&mut cdr);
        assert!((cdr.charge - 0.02).abs() < 0.001);
    }

    // --- CDR Storage Tests ---

    #[test]
    fn insert_and_retrieve_cdrs() {
        let db = test_db();
        let acct = make_account("Alice", "+1001", "001");
        db.create_account(&acct).unwrap();

        let cdr = CallDetailRecord::new(
            acct.id.clone(),
            CallType::MobileOriginating,
            "+1001".into(),
            "+1002".into(),
            120,
            "MSC001".into(),
            "CELL01".into(),
        );
        db.insert_cdr(&cdr).unwrap();

        let cdrs = db.get_cdrs_for_account(&acct.id).unwrap();
        assert_eq!(cdrs.len(), 1);
        assert_eq!(cdrs[0].calling_party, "+1001");
        assert_eq!(cdrs[0].duration_seconds, 120);
    }

    // --- CALEA Intercept Tests ---

    #[test]
    fn create_and_list_intercepts() {
        let db = test_db();
        let acct = make_account("Target", "+1001", "001");
        db.create_account(&acct).unwrap();

        let target = InterceptTarget::new(
            acct.id.clone(),
            "WARRANT-2026-001".into(),
            InterceptType::Full,
            "10.0.50.100".into(),
            9500,
        );
        db.create_intercept(&target).unwrap();

        let intercepts = db.list_intercepts().unwrap();
        assert_eq!(intercepts.len(), 1);
        assert_eq!(intercepts[0].warrant_id, "WARRANT-2026-001");
        assert_eq!(intercepts[0].dest_ip, "10.0.50.100");
        assert_eq!(intercepts[0].dest_port, 9500);
        assert_eq!(intercepts[0].intercept_type, InterceptType::Full);
        assert_eq!(intercepts[0].status, InterceptStatus::Active);
    }

    #[test]
    fn get_intercepts_for_account() {
        let db = test_db();
        let a1 = make_account("Target1", "+1001", "001");
        let a2 = make_account("Target2", "+1002", "002");
        db.create_account(&a1).unwrap();
        db.create_account(&a2).unwrap();

        db.create_intercept(&InterceptTarget::new(
            a1.id.clone(), "W001".into(), InterceptType::Full, "10.0.0.1".into(), 9500,
        )).unwrap();
        db.create_intercept(&InterceptTarget::new(
            a1.id.clone(), "W002".into(), InterceptType::Sms, "10.0.0.2".into(), 9501,
        )).unwrap();
        db.create_intercept(&InterceptTarget::new(
            a2.id.clone(), "W003".into(), InterceptType::SignalingOnly, "10.0.0.3".into(), 9502,
        )).unwrap();

        let a1_intercepts = db.get_intercepts_for_account(&a1.id).unwrap();
        assert_eq!(a1_intercepts.len(), 2);

        let a2_intercepts = db.get_intercepts_for_account(&a2.id).unwrap();
        assert_eq!(a2_intercepts.len(), 1);
    }

    #[test]
    fn toggle_intercept_status() {
        let db = test_db();
        let acct = make_account("Target", "+1001", "001");
        db.create_account(&acct).unwrap();

        let target = InterceptTarget::new(
            acct.id.clone(), "W001".into(), InterceptType::Full, "10.0.0.1".into(), 9500,
        );
        db.create_intercept(&target).unwrap();

        db.update_intercept_status(&target.id, InterceptStatus::Inactive).unwrap();
        let fetched = db.get_intercept(&target.id).unwrap();
        assert_eq!(fetched.status, InterceptStatus::Inactive);

        db.update_intercept_status(&target.id, InterceptStatus::Active).unwrap();
        let fetched = db.get_intercept(&target.id).unwrap();
        assert_eq!(fetched.status, InterceptStatus::Active);
    }

    #[test]
    fn delete_intercept() {
        let db = test_db();
        let acct = make_account("Target", "+1001", "001");
        db.create_account(&acct).unwrap();

        let target = InterceptTarget::new(
            acct.id.clone(), "W001".into(), InterceptType::Full, "10.0.0.1".into(), 9500,
        );
        db.create_intercept(&target).unwrap();
        db.delete_intercept(&target.id).unwrap();

        assert_eq!(db.list_intercepts().unwrap().len(), 0);
    }

    #[test]
    fn intercept_count_only_active() {
        let db = test_db();
        let acct = make_account("Target", "+1001", "001");
        db.create_account(&acct).unwrap();

        let t1 = InterceptTarget::new(
            acct.id.clone(), "W001".into(), InterceptType::Full, "10.0.0.1".into(), 9500,
        );
        let t2 = InterceptTarget::new(
            acct.id.clone(), "W002".into(), InterceptType::Sms, "10.0.0.2".into(), 9501,
        );
        db.create_intercept(&t1).unwrap();
        db.create_intercept(&t2).unwrap();

        assert_eq!(db.get_intercept_count().unwrap(), 2);

        db.update_intercept_status(&t1.id, InterceptStatus::Inactive).unwrap();
        assert_eq!(db.get_intercept_count().unwrap(), 1);
    }

    // --- PBX Forwarding Tests ---

    #[test]
    fn create_and_list_forwarding_rules() {
        let db = test_db();
        let acct = make_account("Alice", "+1001", "001");
        db.create_account(&acct).unwrap();

        let rule = ForwardingRule::new(
            acct.id.clone(),
            "+1001".into(),
            "+1099".into(),
            ForwardingType::Unconditional,
            20,
        );
        db.create_forwarding_rule(&rule).unwrap();

        let rules = db.list_forwarding_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].source_msisdn, "+1001");
        assert_eq!(rules[0].dest_address, "+1099");
        assert_eq!(rules[0].forwarding_type, ForwardingType::Unconditional);
        assert_eq!(rules[0].status, ForwardingStatus::Active);
    }

    #[test]
    fn forwarding_rule_ss7_map_operation() {
        let rule = ForwardingRule::new(
            "acct1".into(),
            "+1001".into(),
            "sip:user@voip.example.com".into(),
            ForwardingType::Busy,
            20,
        );
        let op = rule.ss7_map_operation();
        assert!(op.contains("RegisterSS(CFB)"));
        assert!(op.contains("sip:user@voip.example.com"));
    }

    #[test]
    fn forwarding_types_have_correct_opcodes() {
        assert_eq!(ForwardingType::Unconditional.ss7_opcode(), "RegisterSS(CFU)");
        assert_eq!(ForwardingType::Busy.ss7_opcode(), "RegisterSS(CFB)");
        assert_eq!(ForwardingType::NoAnswer.ss7_opcode(), "RegisterSS(CFNA)");
        assert_eq!(ForwardingType::NotReachable.ss7_opcode(), "RegisterSS(CFNRc)");
    }

    #[test]
    fn get_forwarding_rules_for_account() {
        let db = test_db();
        let a1 = make_account("Alice", "+1001", "001");
        let a2 = make_account("Bob", "+1002", "002");
        db.create_account(&a1).unwrap();
        db.create_account(&a2).unwrap();

        db.create_forwarding_rule(&ForwardingRule::new(
            a1.id.clone(), "+1001".into(), "+1099".into(), ForwardingType::Unconditional, 20,
        )).unwrap();
        db.create_forwarding_rule(&ForwardingRule::new(
            a1.id.clone(), "+1001".into(), "+1098".into(), ForwardingType::Busy, 20,
        )).unwrap();
        db.create_forwarding_rule(&ForwardingRule::new(
            a2.id.clone(), "+1002".into(), "+1097".into(), ForwardingType::NoAnswer, 30,
        )).unwrap();

        assert_eq!(db.get_forwarding_rules_for_account(&a1.id).unwrap().len(), 2);
        assert_eq!(db.get_forwarding_rules_for_account(&a2.id).unwrap().len(), 1);
    }

    #[test]
    fn toggle_forwarding_status() {
        let db = test_db();
        let acct = make_account("Alice", "+1001", "001");
        db.create_account(&acct).unwrap();

        let rule = ForwardingRule::new(
            acct.id.clone(), "+1001".into(), "+1099".into(), ForwardingType::Unconditional, 20,
        );
        db.create_forwarding_rule(&rule).unwrap();

        db.update_forwarding_status(&rule.id, ForwardingStatus::Inactive).unwrap();
        let fetched = db.get_forwarding_rule(&rule.id).unwrap();
        assert_eq!(fetched.status, ForwardingStatus::Inactive);

        db.update_forwarding_status(&rule.id, ForwardingStatus::Active).unwrap();
        let fetched = db.get_forwarding_rule(&rule.id).unwrap();
        assert_eq!(fetched.status, ForwardingStatus::Active);
    }

    #[test]
    fn delete_forwarding_rule() {
        let db = test_db();
        let acct = make_account("Alice", "+1001", "001");
        db.create_account(&acct).unwrap();

        let rule = ForwardingRule::new(
            acct.id.clone(), "+1001".into(), "+1099".into(), ForwardingType::Unconditional, 20,
        );
        db.create_forwarding_rule(&rule).unwrap();
        db.delete_forwarding_rule(&rule.id).unwrap();

        assert_eq!(db.list_forwarding_rules().unwrap().len(), 0);
    }

    #[test]
    fn forwarding_count_only_active() {
        let db = test_db();
        let acct = make_account("Alice", "+1001", "001");
        db.create_account(&acct).unwrap();

        let r1 = ForwardingRule::new(
            acct.id.clone(), "+1001".into(), "+1099".into(), ForwardingType::Unconditional, 20,
        );
        let r2 = ForwardingRule::new(
            acct.id.clone(), "+1001".into(), "+1098".into(), ForwardingType::Busy, 20,
        );
        db.create_forwarding_rule(&r1).unwrap();
        db.create_forwarding_rule(&r2).unwrap();

        assert_eq!(db.get_forwarding_count().unwrap(), 2);

        db.update_forwarding_status(&r1.id, ForwardingStatus::Inactive).unwrap();
        assert_eq!(db.get_forwarding_count().unwrap(), 1);
    }

    // --- Intercept Type Tests ---

    #[test]
    fn intercept_type_roundtrip() {
        for t in &[InterceptType::Full, InterceptType::SignalingOnly, InterceptType::Sms, InterceptType::Data] {
            let s = t.as_str();
            let parsed = InterceptType::from_str(s);
            assert_eq!(*t, parsed);
        }
    }

    // --- Forwarding Type Tests ---

    #[test]
    fn forwarding_type_roundtrip() {
        for t in &[
            ForwardingType::Unconditional,
            ForwardingType::Busy,
            ForwardingType::NoAnswer,
            ForwardingType::NotReachable,
        ] {
            let s = t.as_str();
            let parsed = ForwardingType::from_str(s);
            assert_eq!(*t, parsed);
        }
    }

    // --- Delete account cascades to CDRs ---

    #[test]
    fn delete_account_cascades_cdrs() {
        let db = test_db();
        let acct = make_account("Alice", "+1001", "001");
        db.create_account(&acct).unwrap();

        let cdr = CallDetailRecord::new(
            acct.id.clone(),
            CallType::MobileOriginating,
            "+1001".into(),
            "+1002".into(),
            60,
            "MSC".into(),
            "CELL".into(),
        );
        db.insert_cdr(&cdr).unwrap();
        assert_eq!(db.get_cdrs_for_account(&acct.id).unwrap().len(), 1);

        db.delete_account(&acct.id).unwrap();
        assert_eq!(db.get_cdrs_for_account(&acct.id).unwrap().len(), 0);
    }
}
