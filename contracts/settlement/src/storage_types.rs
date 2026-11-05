use soroban_sdk::{contracttype, Address, Bytes, BytesN};

// ledger, ~5s/ledger
pub(crate) const DAY_IN_LEDGERS: u32 = 17_280;

pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const PERSISTENT_BUMP_AMOUNT: u32 = 90 * DAY_IN_LEDGERS;
pub(crate) const PERSISTENT_LIFETIME_THRESHOLD: u32 = PERSISTENT_BUMP_AMOUNT - DAY_IN_LEDGERS;

/// Everything the settlement contract needs to know about one fund.
///
/// `issuer_admin` holds the share token's mint authority and signs every
/// `close_window`; `settlement_account` owns the fund's sealed balance.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundConfig {
    pub share_token: Address,
    pub share_wrapper: Address,
    pub settlement_account: Address,
    pub issuer_admin: Address,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LegRole {
    Share = 0,
    Cash = 1,
}

/// One sealed transfer. `data` is the primitive's `TransferData` XDR.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Leg {
    pub token: Address,
    pub from: Address,
    pub to: Address,
    pub data: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WindowState {
    pub window_id: u32,
    pub opened_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    NextFundId,               // instance: u32, next id to hand out
    Fund(u32),                // persistent: FundConfig
    Window(u32),              // persistent: WindowState, per fund
    Leg(BytesN<32>, LegRole), // persistent: bool; presence = settled
}
