use soroban_sdk::{BytesN, Env};

use crate::storage_types::{
    DataKey, FundConfig, LegRole, WindowState, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
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

// ----- NextFundId (instance) -----
pub fn set_next_fund_id(e: &Env, id: u32) {
    e.storage().instance().set(&DataKey::NextFundId, &id);
}

/// Hand out the next fund id and advance the counter.
pub fn alloc_fund_id(e: &Env) -> u32 {
    let id: u32 = e
        .storage()
        .instance()
        .get(&DataKey::NextFundId)
        .unwrap_or(1);
    set_next_fund_id(e, id + 1);
    id
}

// ----- Fund (persistent) -----
pub fn get_fund(e: &Env, fund_id: u32) -> Option<FundConfig> {
    get_persistent(e, &DataKey::Fund(fund_id))
}

pub fn set_fund(e: &Env, fund_id: u32, cfg: &FundConfig) {
    set_persistent(e, &DataKey::Fund(fund_id), cfg);
}

// ----- Window (persistent, per fund) -----
pub fn get_window(e: &Env, fund_id: u32) -> Option<WindowState> {
    get_persistent(e, &DataKey::Window(fund_id))
}

pub fn set_window(e: &Env, fund_id: u32, state: &WindowState) {
    set_persistent(e, &DataKey::Window(fund_id), state);
}

// ----- Leg (persistent) -----
//
// The entry's presence is the replay guard. Nothing about the leg is stored,
// only that this `(order_id, role)` has run once.
pub fn is_leg_settled(e: &Env, order_id: &BytesN<32>, role: LegRole) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::Leg(order_id.clone(), role))
}

pub fn mark_leg_settled(e: &Env, order_id: &BytesN<32>, role: LegRole) {
    set_persistent(e, &DataKey::Leg(order_id.clone(), role), &true);
}
