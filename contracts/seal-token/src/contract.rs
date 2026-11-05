use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, Symbol, Vec};
use stellar_access::access_control::{self as access_control, AccessControl};
use stellar_macros::only_role;
use stellar_tokens::confidential::{
    compliance::{
        storage as compliance, ClawbackData, ComplianceConfig, ConfidentialClawback,
        ConfidentialCompliance,
    },
    storage as token, ConfidentialAccount, ConfidentialToken, SpenderDelegation,
};

use crate::hooks::SealHooks;
use crate::storage;
use crate::storage_types::SealConfig;

pub const ISSUER_ROLE: Symbol = symbol_short!("issuer");

#[contract]
pub struct SealToken;

#[contractimpl]
impl SealToken {
    #[allow(clippy::too_many_arguments)]
    pub fn __constructor(
        e: &Env,
        admin: Address,
        issuer: Address,
        token: Address,
        verifier: Address,
        auditor: Address,
        compliance: Option<ComplianceConfig>,
        seal: SealConfig,
    ) {
        token::set_underlying_asset(e, &token);
        token::set_verifier(e, &verifier);
        token::set_auditor(e, &auditor);
        token::set_address_as_field_element(e);
        if let Some(cfg) = compliance {
            compliance::set_compliance_config(e, &cfg);
        }
        storage::set_seal_config(e, &seal);
        access_control::set_admin(e, &admin);
        access_control::grant_role_no_auth(e, &issuer, &ISSUER_ROLE, &admin);
    }

    pub fn seal_config(e: &Env) -> SealConfig {
        storage::seal_config(e)
    }
}

// The primitive's full surface (register, deposit, merge, transfer, withdraw,
// spender delegation) with `SealHooks` in front of every call.
#[contractimpl(contracttrait)]
impl ConfidentialToken for SealToken {
    type Hooks = SealHooks;
}

#[contractimpl(contracttrait)]
impl ConfidentialCompliance for SealToken {
    #[only_role(operator, "issuer")]
    fn freeze(e: &Env, account: Address, operator: Address) {
        compliance::freeze(e, &account);
    }

    #[only_role(operator, "issuer")]
    fn unfreeze(e: &Env, account: Address, operator: Address) {
        compliance::unfreeze(e, &account);
    }

    #[only_role(operator, "issuer")]
    fn set_compliance_config(e: &Env, config: ComplianceConfig, operator: Address) {
        compliance::set_compliance_config(e, &config);
    }
}

#[contractimpl(contracttrait)]
impl ConfidentialClawback for SealToken {
    #[only_role(operator, "issuer")]
    fn clawback(
        e: &Env,
        account: Address,
        amount: i128,
        destination: Option<Address>,
        data: Bytes,
        operator: Address,
    ) {
        let d: ClawbackData = token::decode_data(e, &data);
        compliance::clawback(e, &account, amount, &destination, &d.proof);
    }

    #[only_role(operator, "issuer")]
    fn force_revoke_spender(e: &Env, account: Address, spender: Address, operator: Address) {
        compliance::force_revoke_spender(e, &account, &spender);
    }
}

#[contractimpl(contracttrait)]
impl AccessControl for SealToken {}
