mod policy;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};
use stellar_tokens::confidential::compliance::PolicyClient;

use crate::{FundPolicy, FundPolicyClient, InvestorStatus};

pub const FUND_ID: u32 = 1;
pub const NOW: u64 = 1_000;

/// One policy contract with one fund registered. `check` is the `Policy`
/// trait client the wrapper would call; `policy` is the admin surface.
pub struct Harness<'a> {
    pub e: Env,
    pub policy: FundPolicyClient<'a>,
    pub check: PolicyClient<'a>,
    pub compliance: Address,
    pub wrapper: Address,
    pub settlement_account: Address,
}

impl<'a> Harness<'a> {
    pub fn new() -> Self {
        let e = Env::default();
        e.mock_all_auths();
        e.ledger().set_timestamp(NOW);
        let admin = Address::generate(&e);
        let compliance = Address::generate(&e);
        let addr = e.register(FundPolicy, (&admin, &compliance));
        let wrapper = Address::generate(&e);
        let settlement_account = Address::generate(&e);
        let policy = FundPolicyClient::new(&e, &addr);
        let check = PolicyClient::new(&e, &addr);
        policy.register_fund(&FUND_ID, &wrapper, &settlement_account, &compliance);
        Harness {
            e,
            policy,
            check,
            compliance,
            wrapper,
            settlement_account,
        }
    }

    pub fn set_investor(&self, who: &Address, status: InvestorStatus, kyc_expiry: u64) {
        self.policy
            .set_investor(&FUND_ID, who, &status, &kyc_expiry, &self.compliance);
    }

    pub fn authorized(&self, who: &Address) -> bool {
        self.check.is_authorized(who, &self.wrapper)
    }
}
