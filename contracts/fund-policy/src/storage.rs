use soroban_sdk::{Address, Env};

use crate::storage_types::{
    DataKey, InvestorEntry, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
    PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD,
};

pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

/// Read a persistent entry, refreshing its TTL when it exists.
fn get_persistent<V: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>>(
    e: &Env,
    key: &DataKey,
) -> Option<V> {
    let value = e.storage().persistent().get::<_, V>(key)?;
    e.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
    Some(value)
}

/// Write a persistent entry and refresh its TTL.
fn set_persistent<V: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(
    e: &Env,
    key: &DataKey,
    value: &V,
) {
    e.storage().persistent().set(key, value);
    e.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

// ----- Fund (persistent): share wrapper -> fund id -----
pub fn get_fund_id(e: &Env, share_wrapper: &Address) -> Option<u32> {
    get_persistent(e, &DataKey::Fund(share_wrapper.clone()))
}

pub fn set_fund_id(e: &Env, share_wrapper: &Address, fund_id: u32) {
    set_persistent(e, &DataKey::Fund(share_wrapper.clone()), &fund_id);
}

// ----- Settlement (persistent): fund id -> settlement account -----
pub fn get_settlement_account(e: &Env, fund_id: u32) -> Option<Address> {
    get_persistent(e, &DataKey::Settlement(fund_id))
}

pub fn set_settlement_account(e: &Env, fund_id: u32, account: &Address) {
    set_persistent(e, &DataKey::Settlement(fund_id), account);
}

// ----- Investor (persistent) -----
pub fn get_investor(e: &Env, fund_id: u32, account: &Address) -> Option<InvestorEntry> {
    get_persistent(e, &DataKey::Investor(fund_id, account.clone()))
}

pub fn set_investor(e: &Env, fund_id: u32, account: &Address, entry: &InvestorEntry) {
    set_persistent(e, &DataKey::Investor(fund_id, account.clone()), entry);
}
