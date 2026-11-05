use soroban_sdk::{panic_with_error, Env};

use crate::error::SealTokenError;
use crate::storage_types::{SealConfig, SEAL_CONFIG};

// Instance TTL is extended by the primitive on every call, so only the
// config accessors live here.

pub fn set_seal_config(e: &Env, cfg: &SealConfig) {
    e.storage().instance().set(&SEAL_CONFIG, cfg);
}

/// The config is written by the constructor, so a missing entry is a broken
/// instance, not a default.
pub fn seal_config(e: &Env) -> SealConfig {
    e.storage()
        .instance()
        .get(&SEAL_CONFIG)
        .unwrap_or_else(|| panic_with_error!(e, SealTokenError::ConfigNotSet))
}
