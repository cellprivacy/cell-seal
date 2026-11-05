use super::{order_id, shares, withdraw_data, Fund};
use crate::LegRole;

#[test]
fn full_demo_cycle_wrap_deliver_redeem_close() {
    let f = Fund::new();
    let investor = f.onboard_investor();

    // Window 1: fund has one subscription of 1,000 shares → net +1,000.
    f.wrap(1, 1_000);
    assert_eq!(f.share.total_supply(), shares(1_000));
    assert_eq!(
        f.share.balance(&f.wrapper_addr),
        shares(1_000),
        "all public supply sits in the wrapper"
    );

    // Deliver the subscription: sealed transfer settlement account → investor.
    f.submit(1, &f.share_leg(&f.settlement_account, &investor));
    assert!(f
        .settlement
        .leg_settled(&order_id(&f.e, 1), &LegRole::Share));
    f.wrapper.merge(&investor);

    // Investor redeems: sealed transfer investor → settlement account, tagged with the order.
    f.submit(2, &f.share_leg(&investor, &f.settlement_account));
    f.wrapper.merge(&f.settlement_account);

    // Public supply has not moved: the redemption is invisible on the ledger.
    assert_eq!(f.share.total_supply(), shares(1_000));

    // Window 2: net −400 → unwrap and burn. The only public number of the period.
    f.settlement
        .close_window(&f.fund_id, &2, &-shares(400), &Some(withdraw_data(&f.e)));
    assert_eq!(f.share.total_supply(), shares(600));
    assert_eq!(f.share.balance(&f.wrapper_addr), shares(600));
    assert_eq!(
        f.share.balance(&f.settlement_account),
        0,
        "nothing public lingers outside the wrapper"
    );
    assert_eq!(f.settlement.window(&f.fund_id).window_id, 3);
}
