use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

use super::{point, register_data, Harness, OTHER_AUDITOR};

#[test]
fn register_binds_the_account_to_the_fund_auditor() {
    let h = Harness::new();
    let investor = h.investor();
    let acct = h.token.confidential_balance(&investor);
    assert_eq!(acct.auditor_id, super::FUND_AUDITOR);
    assert_eq!(acct.spending_public_key, point(&h.e));
}

#[test]
#[should_panic(expected = "Error(Contract, #4000)")]
fn register_with_foreign_auditor_is_rejected() {
    let h = Harness::new();
    let investor = Address::generate(&h.e);
    h.token
        .register(&investor, &OTHER_AUDITOR, &register_data(&h.e));
}
