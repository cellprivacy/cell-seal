use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol, Vec};
use stellar_access::access_control::{self as access_control, AccessControl};
use stellar_macros::only_role;
use stellar_tokens::confidential::compliance::Policy;

use crate::storage_types::{InvestorEntry, InvestorStatus};
use crate::{event, storage};

pub const COMPLIANCE_ROLE: Symbol = symbol_short!("complianc");

#[contract]
pub struct FundPolicy;

#[contractimpl]
impl FundPolicy {
    pub fn __constructor(e: &Env, admin: Address, compliance: Address) {
        access_control::set_admin(e, &admin);
        access_control::grant_role_no_auth(e, &compliance, &COMPLIANCE_ROLE, &admin);
        storage::extend_instance(e);
    }

    // ---------- compliance-gated writes ----------

    /// Binds a share wrapper to a fund and records the fund's settlement
    /// account, which is always authorized for that wrapper.
    #[only_role(operator, "complianc")]
    pub fn register_fund(
        e: &Env,
        fund_id: u32,
        share_wrapper: Address,
        settlement_account: Address,
        operator: Address,
    ) {
        storage::set_fund_id(e, &share_wrapper, fund_id);
        storage::set_settlement_account(e, fund_id, &settlement_account);
        storage::extend_instance(e);
    }

    #[only_role(operator, "complianc")]
    pub fn set_investor(
        e: &Env,
        fund_id: u32,
        account: Address,
        status: InvestorStatus,
        kyc_expiry: u64,
        operator: Address,
    ) {
        let entry = InvestorEntry {
            status,
            kyc_expiry,
            updated_ledger: e.ledger().sequence(),
        };
        storage::set_investor(e, fund_id, &account, &entry);
        storage::extend_instance(e);
        event::investor_updated(e, fund_id, &account, status, kyc_expiry);
    }

    // ---------- reads ----------

    pub fn investor(e: &Env, fund_id: u32, account: Address) -> Option<InvestorEntry> {
        storage::get_investor(e, fund_id, &account)
    }

    pub fn fund_of(e: &Env, share_wrapper: Address) -> Option<u32> {
        storage::get_fund_id(e, &share_wrapper)
    }
}

// Called by the wrapper's `ComplianceHooks` on both ends of every sealed
// transfer, and on register, deposit and withdraw.
#[contractimpl]
impl Policy for FundPolicy {
    fn is_authorized(e: Env, account: Address, token: Address) -> bool {
        let Some(fund_id) = storage::get_fund_id(&e, &token) else {
            return false;
        };
        if storage::get_settlement_account(&e, fund_id).as_ref() == Some(&account) {
            return true;
        }
        match storage::get_investor(&e, fund_id, &account) {
            Some(entry) => {
                entry.status == InvestorStatus::Approved
                    && entry.kyc_expiry > e.ledger().timestamp()
            }
            None => false,
        }
    }
}

#[contractimpl(contracttrait)]
impl AccessControl for FundPolicy {}
