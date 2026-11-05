use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, vec, Address, Bytes, BytesN, Env,
    IntoVal, Symbol, Vec,
};
use stellar_access::access_control::{self as access_control, AccessControl};
use stellar_macros::only_role;
use stellar_tokens::{
    confidential::ConfidentialTokenClient,
    fungible::{burnable::FungibleBurnableClient, FungibleTokenClient},
};

use crate::error::SettlementError;
use crate::storage_types::{FundConfig, Leg, LegRole, WindowState};
use crate::{event, storage};

pub const PLATFORM_ROLE: Symbol = symbol_short!("platform");

#[contract]
pub struct Settlement;

#[contractimpl]
impl Settlement {
    pub fn __constructor(e: &Env, admin: Address, platform: Address) {
        access_control::set_admin(e, &admin);
        access_control::grant_role_no_auth(e, &platform, &PLATFORM_ROLE, &admin);
        storage::set_next_fund_id(e, 1);
        storage::extend_instance(e);
    }

    // ---------- platform ----------

    /// Registers a fund and opens its first window. Ids start at 1.
    #[only_role(operator, "platform")]
    pub fn register_fund(e: &Env, cfg: FundConfig, operator: Address) -> u32 {
        let fund_id = storage::alloc_fund_id(e);
        storage::set_fund(e, fund_id, &cfg);
        storage::set_window(e, fund_id, &Self::open_window(e, 1));
        storage::extend_instance(e);
        event::fund_registered(e, fund_id, &cfg);
        fund_id
    }

    // ---------- reads ----------

    pub fn fund(e: &Env, fund_id: u32) -> FundConfig {
        Self::require_fund(e, fund_id)
    }

    pub fn window(e: &Env, fund_id: u32) -> WindowState {
        Self::require_window(e, fund_id)
    }

    pub fn leg_settled(e: &Env, order_id: BytesN<32>, role: LegRole) -> bool {
        storage::is_leg_settled(e, &order_id, role)
    }

    // ---------- settlement ----------

    /// One sealed transfer, tagged with the order it belongs to. One end of
    /// the leg must be the fund's settlement account: this is a subscription
    /// delivery or a redemption, never an arbitrary transfer. `leg.from` signs.
    ///
    /// The replay guard is written before the transfer runs, so a leg that
    /// fails inside the wrapper rolls the guard back with it.
    pub fn submit_leg(e: &Env, fund_id: u32, order_id: BytesN<32>, role: LegRole, leg: Leg) {
        leg.from.require_auth();
        let cfg = Self::require_fund(e, fund_id);
        if leg.token != cfg.share_wrapper {
            panic_with_error!(e, SettlementError::TokenNotInFund);
        }
        if leg.from != cfg.settlement_account && leg.to != cfg.settlement_account {
            panic_with_error!(e, SettlementError::LegShapeInvalid);
        }
        if storage::is_leg_settled(e, &order_id, role) {
            panic_with_error!(e, SettlementError::LegAlreadySettled);
        }
        storage::mark_leg_settled(e, &order_id, role);

        ConfidentialTokenClient::new(e, &leg.token)
            .confidential_transfer(&leg.from, &leg.to, &leg.data);

        storage::extend_instance(e);
        event::leg_settled(e, fund_id, order_id, role, leg);
    }

    /// Settles the period's net flow against the public share token. Only this
    /// call changes the public supply: `net > 0` mints and wraps the net into
    /// the settlement account; `net < 0` unwraps and burns it. Signed by the
    /// issuer admin (mint/burn authority) and the settlement account
    /// (wrapper owner).
    ///
    /// Unwrap and burn happen in the same call so no public share ever sits
    /// outside the wrapper between two transactions.
    pub fn close_window(
        e: &Env,
        fund_id: u32,
        window_id: u32,
        net: i128,
        withdraw_data: Option<Bytes>,
    ) {
        let cfg = Self::require_fund(e, fund_id);
        cfg.issuer_admin.require_auth();
        cfg.settlement_account.require_auth();

        let state = Self::require_window(e, fund_id);
        if state.window_id != window_id {
            panic_with_error!(e, SettlementError::WindowIdMismatch);
        }

        let share = FungibleTokenClient::new(e, &cfg.share_token);
        let wrapper = ConfidentialTokenClient::new(e, &cfg.share_wrapper);

        if net > 0 {
            Self::mint_shares(e, &cfg, net);
            wrapper.deposit(&cfg.settlement_account, &cfg.settlement_account, &net);
        } else if net < 0 {
            let amount = -net;
            let data = withdraw_data
                .unwrap_or_else(|| panic_with_error!(e, SettlementError::WithdrawDataMissing));
            wrapper.withdraw(
                &cfg.settlement_account,
                &cfg.settlement_account,
                &amount,
                &data,
            );
            FungibleBurnableClient::new(e, &cfg.share_token).burn(&cfg.settlement_account, &amount);
        }

        let supply_after = share.balance(&cfg.share_wrapper);
        storage::set_window(e, fund_id, &Self::open_window(e, window_id + 1));
        storage::extend_instance(e);
        event::window_closed(e, fund_id, window_id, net, supply_after);
    }
}

impl Settlement {
    fn require_fund(e: &Env, fund_id: u32) -> FundConfig {
        storage::get_fund(e, fund_id)
            .unwrap_or_else(|| panic_with_error!(e, SettlementError::UnknownFund))
    }

    fn require_window(e: &Env, fund_id: u32) -> WindowState {
        storage::get_window(e, fund_id)
            .unwrap_or_else(|| panic_with_error!(e, SettlementError::UnknownFund))
    }

    fn open_window(e: &Env, window_id: u32) -> WindowState {
        WindowState {
            window_id,
            opened_ledger: e.ledger().sequence(),
        }
    }

    // share-token exposes `mint(to, amount, operator)` gated by the issuer
    // role. Invoked by name so this crate does not depend on share-token.
    fn mint_shares(e: &Env, cfg: &FundConfig, amount: i128) {
        e.invoke_contract::<()>(
            &cfg.share_token,
            &Symbol::new(e, "mint"),
            vec![
                e,
                cfg.settlement_account.clone().into_val(e),
                amount.into_val(e),
                cfg.issuer_admin.clone().into_val(e),
            ],
        );
    }
}

#[contractimpl(contracttrait)]
impl AccessControl for Settlement {}
