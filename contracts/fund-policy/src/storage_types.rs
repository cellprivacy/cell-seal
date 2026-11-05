use soroban_sdk::{contracttype, Address};

// ledger, ~5s/ledger
pub(crate) const DAY_IN_LEDGERS: u32 = 17_280;

pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const PERSISTENT_BUMP_AMOUNT: u32 = 90 * DAY_IN_LEDGERS;
pub(crate) const PERSISTENT_LIFETIME_THRESHOLD: u32 = PERSISTENT_BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvestorStatus {
    Approved = 0,
    Suspended = 1,
    Removed = 2,
}

/// What compliance last decided about one account in one fund. Only
/// `Approved` with a live `kyc_expiry` (unix seconds) passes the policy.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvestorEntry {
    pub status: InvestorStatus,
    pub kyc_expiry: u64,
    pub updated_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Fund(Address),          // persistent: u32, share wrapper -> fund id
    Settlement(u32),        // persistent: Address, fund id -> settlement account
    Investor(u32, Address), // persistent: InvestorEntry, (fund id, account)
}
