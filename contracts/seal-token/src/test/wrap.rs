use super::{withdraw_data, Harness};

#[test]
fn settlement_account_wraps_and_unwraps() {
    let h = Harness::new();
    h.register(&h.settlement_account);
    h.sac.mint(&h.settlement_account, &1_000);

    h.token
        .deposit(&h.settlement_account, &h.settlement_account, &600);
    assert_eq!(h.sac.balance(&h.settlement_account), 400);
    assert_eq!(h.sac.balance(&h.token_addr), 600);

    h.token.merge(&h.settlement_account);
    h.token.withdraw(
        &h.settlement_account,
        &h.settlement_account,
        &250,
        &withdraw_data(&h.e),
    );
    assert_eq!(h.sac.balance(&h.settlement_account), 650);
    assert_eq!(h.sac.balance(&h.token_addr), 350);
}

#[test]
#[should_panic(expected = "Error(Contract, #4001)")]
fn investor_cannot_deposit() {
    let h = Harness::new();
    let investor = h.investor();
    h.sac.mint(&investor, &100);
    h.token.deposit(&investor, &investor, &100);
}

#[test]
#[should_panic(expected = "Error(Contract, #4001)")]
fn settlement_account_cannot_deposit_to_someone_else() {
    let h = Harness::new();
    h.register(&h.settlement_account);
    let investor = h.investor();
    h.sac.mint(&h.settlement_account, &100);
    h.token.deposit(&h.settlement_account, &investor, &100);
}

#[test]
#[should_panic(expected = "Error(Contract, #4002)")]
fn investor_cannot_withdraw() {
    let h = Harness::new();
    let investor = h.investor();
    h.token
        .withdraw(&investor, &investor, &1, &withdraw_data(&h.e));
}
