use soroban_sdk::{contractevent, Address, BytesN, Env};

use crate::storage_types::{FundConfig, Leg, LegRole};

/// Emitted once per fund, when the platform registers it.
///
/// Topics: `("fund_registered", fund_id)`.
/// Data: `share_token`, `share_wrapper`, `settlement_account`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundRegistered {
    #[topic]
    pub fund_id: u32,
    pub share_token: Address,
    pub share_wrapper: Address,
    pub settlement_account: Address,
}

/// Emitted after a sealed transfer ran under an order reference.
///
/// Topics: `("leg_settled", fund_id, order_id)`, so an indexer can follow one
/// fund or one order. Data: `role`, `token`, `from`, `to`. The amount is not
/// here: it is sealed, and lives only in the wrapper's own transfer event.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LegSettled {
    #[topic]
    pub fund_id: u32,
    #[topic]
    pub order_id: BytesN<32>,
    pub role: LegRole,
    pub token: Address,
    pub from: Address,
    pub to: Address,
}

/// Emitted when a window closes. The only place a period's public figure is
/// published.
///
/// Topics: `("window_closed", fund_id, window_id)`.
/// Data: `net` (signed, in share units), `supply_after` (public shares held
/// by the wrapper after the close).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowClosed {
    #[topic]
    pub fund_id: u32,
    #[topic]
    pub window_id: u32,
    pub net: i128,
    pub supply_after: i128,
}

pub fn fund_registered(e: &Env, fund_id: u32, cfg: &FundConfig) {
    FundRegistered {
        fund_id,
        share_token: cfg.share_token.clone(),
        share_wrapper: cfg.share_wrapper.clone(),
        settlement_account: cfg.settlement_account.clone(),
    }
    .publish(e);
}

pub fn leg_settled(e: &Env, fund_id: u32, order_id: BytesN<32>, role: LegRole, leg: Leg) {
    LegSettled {
        fund_id,
        order_id,
        role,
        token: leg.token,
        from: leg.from,
        to: leg.to,
    }
    .publish(e);
}

pub fn window_closed(e: &Env, fund_id: u32, window_id: u32, net: i128, supply_after: i128) {
    WindowClosed {
        fund_id,
        window_id,
        net,
        supply_after,
    }
    .publish(e);
}
