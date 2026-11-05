use super::{shares, Fund};

#[test]
fn closing_with_zero_net_only_advances_the_window() {
    let f = Fund::new();
    f.settlement.close_window(&f.fund_id, &1, &0, &None);
    assert_eq!(f.share.total_supply(), 0);
    assert_eq!(f.settlement.window(&f.fund_id).window_id, 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #4105)")]
fn closing_the_wrong_window_is_rejected() {
    let f = Fund::new();
    f.settlement.close_window(&f.fund_id, &2, &0, &None);
}

#[test]
#[should_panic(expected = "Error(Contract, #4106)")]
fn negative_net_needs_withdraw_proof() {
    let f = Fund::new();
    f.settlement.close_window(&f.fund_id, &1, &-1, &None);
}

#[test]
#[should_panic(expected = "Error(Contract, #4100)")]
fn unknown_fund_is_rejected() {
    let f = Fund::new();
    f.settlement.close_window(&99, &1, &shares(1), &None);
}
