use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::Address;

use super::{Harness, FUND_ID, NOW};
use crate::InvestorStatus;

#[test]
fn settlement_account_is_always_authorized() {
    let h = Harness::new();
    assert!(h.authorized(&h.settlement_account));
}

#[test]
fn approved_investor_with_live_kyc_is_authorized() {
    let h = Harness::new();
    let inv = Address::generate(&h.e);
    assert!(!h.authorized(&inv));

    h.set_investor(&inv, InvestorStatus::Approved, NOW + 4_000);
    assert!(h.authorized(&inv));

    h.e.ledger().set_timestamp(NOW + 5_000);
    assert!(!h.authorized(&inv), "expired KYC must fail");
}

#[test]
fn suspended_investor_is_not_authorized() {
    let h = Harness::new();
    let inv = Address::generate(&h.e);
    h.set_investor(&inv, InvestorStatus::Approved, NOW + 4_000);
    h.set_investor(&inv, InvestorStatus::Suspended, NOW + 4_000);
    assert!(!h.authorized(&inv));
    assert_eq!(
        h.policy.investor(&FUND_ID, &inv).unwrap().status,
        InvestorStatus::Suspended
    );
}

#[test]
fn unknown_token_is_not_authorized() {
    let h = Harness::new();
    let other = Address::generate(&h.e);
    assert!(!h.check.is_authorized(&h.settlement_account, &other));
    assert_eq!(h.policy.fund_of(&other), None);
    assert_eq!(h.policy.fund_of(&h.wrapper), Some(FUND_ID));
}

#[test]
#[should_panic]
fn stranger_cannot_set_investor() {
    let h = Harness::new();
    let inv = Address::generate(&h.e);
    let stranger = Address::generate(&h.e);
    h.policy
        .set_investor(&FUND_ID, &inv, &InvestorStatus::Approved, &9_999, &stranger);
}
