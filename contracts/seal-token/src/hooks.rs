use soroban_sdk::{panic_with_error, Address, Env};
use stellar_tokens::confidential::{
    compliance::ComplianceHooks, Hooks, RegisterPayload, SetSpenderPayload, SpenderTransferPayload,
    TransferPayload, WithdrawPayload,
};

use crate::error::SealTokenError;
use crate::storage;

/// Cell Seal rules on top of the primitive's compliance gates.
///
/// Wrap and unwrap carry a public amount, so only the fund's settlement
/// account may do either, and only to itself. Every sealed transfer must be
/// driven by the settlement contract so it carries an order reference and
/// cannot pull an allowance without paying the other leg. Every account
/// must bind to the fund's auditor key, or the fund cannot read it.
///
/// Each hook checks the Cell rule first, then hands over to
/// `ComplianceHooks` (freeze, policy).
pub struct SealHooks;

impl Hooks for SealHooks {
    fn on_register(e: &Env, account: &Address, auditor_id: u32, payload: &RegisterPayload) {
        if auditor_id != storage::seal_config(e).auditor_id {
            panic_with_error!(e, SealTokenError::AuditorNotApproved);
        }
        ComplianceHooks::on_register(e, account, auditor_id, payload);
    }

    fn on_deposit(e: &Env, from: &Address, to: &Address, amount: i128) {
        let cfg = storage::seal_config(e);
        if from != &cfg.settlement_account || to != &cfg.settlement_account {
            panic_with_error!(e, SealTokenError::DepositNotAllowed);
        }
        ComplianceHooks::on_deposit(e, from, to, amount);
    }

    fn on_merge(e: &Env, account: &Address) {
        ComplianceHooks::on_merge(e, account);
    }

    fn on_withdraw(e: &Env, from: &Address, to: &Address, amount: i128, payload: &WithdrawPayload) {
        if from != &storage::seal_config(e).settlement_account {
            panic_with_error!(e, SealTokenError::WithdrawNotAllowed);
        }
        ComplianceHooks::on_withdraw(e, from, to, amount, payload);
    }

    fn on_transfer(e: &Env, from: &Address, to: &Address, payload: &TransferPayload) {
        storage::seal_config(e).settlement.require_auth();
        ComplianceHooks::on_transfer(e, from, to, payload);
    }

    fn on_spender_transfer(
        e: &Env,
        spender: &Address,
        from: &Address,
        to: &Address,
        payload: &SpenderTransferPayload,
    ) {
        storage::seal_config(e).settlement.require_auth();
        ComplianceHooks::on_spender_transfer(e, spender, from, to, payload);
    }

    fn on_set_spender(
        e: &Env,
        account: &Address,
        spender: &Address,
        live_until_ledger: u32,
        payload: &SetSpenderPayload,
    ) {
        if spender != &storage::seal_config(e).settlement_account {
            panic_with_error!(e, SealTokenError::SpenderNotAllowed);
        }
        ComplianceHooks::on_set_spender(e, account, spender, live_until_ledger, payload);
    }

    fn on_revoke_spender(e: &Env, account: &Address, spender: &Address) {
        ComplianceHooks::on_revoke_spender(e, account, spender);
    }
}
