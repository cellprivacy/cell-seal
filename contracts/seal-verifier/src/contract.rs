use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, Symbol, Vec};
use stellar_access::access_control::{self as access_control, AccessControl};
use stellar_macros::only_role;
use stellar_tokens::confidential::verifier::{
    storage as verifier, CircuitType, ConfidentialVerifier,
};

pub const MANAGER_ROLE: Symbol = symbol_short!("manager");

#[contract]
pub struct SealVerifier;

#[contractimpl]
impl SealVerifier {
    pub fn __constructor(e: &Env, admin: Address, manager: Address) {
        access_control::set_admin(e, &admin);
        access_control::grant_role_no_auth(e, &manager, &MANAGER_ROLE, &admin);
    }
}

#[contractimpl(contracttrait)]
impl ConfidentialVerifier for SealVerifier {
    #[only_role(operator, "manager")]
    fn register_verification_key(
        e: &Env,
        circuit_type: CircuitType,
        verification_key: Bytes,
        operator: Address,
    ) {
        verifier::register_verification_key(e, circuit_type, &verification_key);
    }

    #[only_role(operator, "manager")]
    fn update_verification_key(
        e: &Env,
        circuit_type: CircuitType,
        new_verification_key: Bytes,
        operator: Address,
    ) {
        verifier::update_verification_key(e, circuit_type, &new_verification_key);
    }
}

#[contractimpl(contracttrait)]
impl AccessControl for SealVerifier {}
