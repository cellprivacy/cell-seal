use soroban_sdk::{contracttype, symbol_short, Address, Symbol};

/// Instance key for `SealConfig`. The primitive keeps its own keys
/// (`ConfidentialTokenStorageKey`, `ComplianceStorageKey`); this is the one
/// entry Cell Seal adds.
pub(crate) const SEAL_CONFIG: Symbol = symbol_short!("SEAL");

/// What the hooks check against. Written once by the constructor.
///
/// `settlement` is the contract every sealed transfer must be driven by;
/// `settlement_account` is the fund's own account, the only one allowed to
/// wrap and unwrap; `auditor_id` is the key every account must bind to.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SealConfig {
    pub settlement: Address,
    pub settlement_account: Address,
    pub auditor_id: u32,
}
