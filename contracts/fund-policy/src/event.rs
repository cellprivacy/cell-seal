use soroban_sdk::{contractevent, Address, Env};

use crate::storage_types::InvestorStatus;

/// Emitted on every `set_investor`, including re-approvals and suspensions.
///
/// Topics: `("investor_updated", fund_id, account)`.
/// Data: `status`, `kyc_expiry`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvestorUpdated {
    #[topic]
    pub fund_id: u32,
    #[topic]
    pub account: Address,
    pub status: InvestorStatus,
    pub kyc_expiry: u64,
}

pub fn investor_updated(
    e: &Env,
    fund_id: u32,
    account: &Address,
    status: InvestorStatus,
    kyc_expiry: u64,
) {
    InvestorUpdated {
        fund_id,
        account: account.clone(),
        status,
        kyc_expiry,
    }
    .publish(e);
}
