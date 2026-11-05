use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};
use stellar_tokens::confidential::verifier::CircuitType;

use crate::contract::{SealVerifier, SealVerifierClient};

const TRANSFER_VK: &[u8] = include_bytes!("../../vks/transfer.vk.bin");

#[test]
fn registers_and_returns_a_real_verification_key() {
    let e = Env::default();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let manager = Address::generate(&e);
    let addr = e.register(SealVerifier, (&admin, &manager));
    let client = SealVerifierClient::new(&e, &addr);

    let vk = Bytes::from_slice(&e, TRANSFER_VK);
    client.register_verification_key(&CircuitType::Transfer, &vk, &manager);

    assert_eq!(client.get_verification_key(&CircuitType::Transfer), vk);
    assert_eq!(TRANSFER_VK.len(), 1760);
}

#[test]
#[should_panic]
fn rejects_registration_without_manager_role() {
    let e = Env::default();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let manager = Address::generate(&e);
    let stranger = Address::generate(&e);
    let addr = e.register(SealVerifier, (&admin, &manager));
    let client = SealVerifierClient::new(&e, &addr);

    client.register_verification_key(
        &CircuitType::Transfer,
        &Bytes::from_slice(&e, TRANSFER_VK),
        &stranger,
    );
}

// ---- real proof, real verifier: the exact path a testnet `register` call takes.

const REGISTER_PROOF: &[u8] = include_bytes!("../testdata/register/proof");
const REGISTER_PUBLIC_INPUTS: &[u8] = include_bytes!("../testdata/register/public_inputs");
const REGISTER_VK: &[u8] = include_bytes!("../../vks/register.vk.bin");

fn verifier_with_register_vk(e: &Env) -> SealVerifierClient<'_> {
    e.mock_all_auths();
    let admin = Address::generate(e);
    let manager = Address::generate(e);
    let addr = e.register(SealVerifier, (&admin, &manager));
    let client = SealVerifierClient::new(e, &addr);
    client.register_verification_key(
        &CircuitType::Register,
        &Bytes::from_slice(e, REGISTER_VK),
        &manager,
    );
    client
}

#[test]
fn accepts_a_real_register_proof_from_the_pinned_toolchain() {
    let e = Env::default();
    e.cost_estimate().budget().reset_unlimited();
    let client = verifier_with_register_vk(&e);
    let ok = client.verify_proof(
        &CircuitType::Register,
        &Bytes::from_slice(&e, REGISTER_PUBLIC_INPUTS),
        &Bytes::from_slice(&e, REGISTER_PROOF),
    );
    assert!(ok, "proof produced by nargo 1.0.0-beta.11 + bb 0.87.0 (keccak transcript) must verify on-chain");
    assert_eq!(REGISTER_PUBLIC_INPUTS.len(), 6 * 32);
}

#[test]
fn rejects_the_same_proof_with_a_tampered_public_input() {
    let e = Env::default();
    e.cost_estimate().budget().reset_unlimited();
    let client = verifier_with_register_vk(&e);
    let mut pi = REGISTER_PUBLIC_INPUTS.to_vec();
    pi[5 * 32 + 31] ^= 0x01; // flip one bit of acct_f: a proof for another account
    let ok = client.verify_proof(
        &CircuitType::Register,
        &Bytes::from_slice(&e, &pi),
        &Bytes::from_slice(&e, REGISTER_PROOF),
    );
    assert!(!ok, "a proof bound to another account must be rejected");
}
