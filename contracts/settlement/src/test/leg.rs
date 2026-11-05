use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;

use super::Fund;

#[test]
#[should_panic(expected = "Error(Contract, #4102)")]
fn same_order_leg_cannot_settle_twice() {
    let f = Fund::new();
    let investor = f.onboard_investor();
    f.wrap(1, 100);
    let leg = f.share_leg(&f.settlement_account, &investor);
    f.submit(1, &leg);
    f.submit(1, &leg);
}

#[test]
#[should_panic(expected = "Error(Contract, #4103)")]
fn leg_between_two_investors_is_rejected() {
    let f = Fund::new();
    let a = f.onboard_investor();
    let b = f.onboard_investor();
    f.submit(1, &f.share_leg(&a, &b));
}

#[test]
#[should_panic(expected = "Error(Contract, #4101)")]
fn leg_on_a_foreign_token_is_rejected() {
    let f = Fund::new();
    let investor = f.onboard_investor();
    let mut leg = f.share_leg(&f.settlement_account, &investor);
    leg.token = Address::generate(&f.e);
    f.submit(1, &leg);
}

#[test]
#[should_panic(expected = "Error(Contract, #3602)")]
fn delivery_to_unapproved_investor_is_blocked_by_policy() {
    let f = Fund::new();
    let stranger = Address::generate(&f.e);
    f.wrap(1, 100);
    f.submit(1, &f.share_leg(&f.settlement_account, &stranger));
}
