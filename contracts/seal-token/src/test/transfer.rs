//! V-2: a sealed transfer must be driven by the settlement contract.

use soroban_sdk::testutils::{MockAuth, MockAuthInvoke};
use soroban_sdk::IntoVal;

use super::{point, transfer_data, FakeSettlementClient, Harness};

#[test]
fn transfer_through_settlement_succeeds() {
    let h = Harness::new();
    h.register(&h.settlement_account);
    let investor = h.investor();

    FakeSettlementClient::new(&h.e, &h.settlement).relay_transfer(
        &h.token_addr,
        &h.settlement_account,
        &investor,
        &transfer_data(&h.e),
    );

    let acct = h.token.confidential_balance(&investor);
    assert_eq!(acct.receiving_commitment, point(&h.e));
}

#[test]
fn direct_transfer_bypassing_settlement_is_rejected() {
    let h = Harness::new();
    h.register(&h.settlement_account);
    let investor = h.investor();

    // Authorize only the sender. The settlement contract has not signed off,
    // so the hook's `settlement.require_auth()` must fail.
    let data = transfer_data(&h.e);
    h.e.mock_auths(&[MockAuth {
        address: &h.settlement_account,
        invoke: &MockAuthInvoke {
            contract: &h.token_addr,
            fn_name: "confidential_transfer",
            args: (&h.settlement_account, &investor, &data).into_val(&h.e),
            sub_invokes: &[],
        },
    }]);
    let res = h
        .token
        .try_confidential_transfer(&h.settlement_account, &investor, &data);
    assert!(
        res.is_err(),
        "transfer that bypasses settlement must be rejected"
    );
}
