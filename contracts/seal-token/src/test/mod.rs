mod compliance;
mod register;
mod transfer;
mod wrap;

use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, token::StellarAssetClient, xdr::ToXdr,
    Address, Bytes, BytesN, Env,
};
use stellar_tokens::confidential::{
    auditor::{ConfidentialAuditor, ConfidentialAuditorClient},
    compliance::{ComplianceConfig, Policy},
    verifier::{CircuitType, ConfidentialVerifier},
    ConfidentialTokenClient, RegisterData, RegisterPayload, TransferData, TransferPayload,
    WithdrawData, WithdrawPayload,
};

use crate::{SealConfig, SealToken};

pub const FUND_AUDITOR: u32 = 7;
pub const OTHER_AUDITOR: u32 = 99;

// ----- fixtures: payloads the mock verifier accepts -----

const GRUMPKIN_G_BYTES: [u8; 64] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xcf, 0x13, 0x5e, 0x75, 0x06, 0xa4, 0x5d, 0x63,
    0x2d, 0x27, 0x0d, 0x45, 0xf1, 0x18, 0x12, 0x94, 0x83, 0x3f, 0xc4, 0x8d, 0x82, 0x3f, 0x27, 0x2c,
];

pub fn point(e: &Env) -> BytesN<64> {
    BytesN::from_array(e, &GRUMPKIN_G_BYTES)
}

fn field(e: &Env, byte: u8) -> BytesN<32> {
    let mut b = [byte; 32];
    b[0] = 0;
    BytesN::from_array(e, &b)
}

pub fn register_data(e: &Env) -> Bytes {
    RegisterData {
        payload: RegisterPayload {
            y: point(e),
            pvk: point(e),
        },
        proof: Bytes::new(e),
    }
    .to_xdr(e)
}

pub fn transfer_data(e: &Env) -> Bytes {
    TransferData {
        payload: TransferPayload {
            c_spend_new: point(e),
            c_transfer: point(e),
            r_e_point: point(e),
            v_tilde: field(e, 0x11),
            b_tilde: field(e, 0x12),
            sigma: field(e, 0x13),
            v_tilde_aud_r: field(e, 0x14),
            r_tilde_aud_r: field(e, 0x15),
            v_tilde_aud_s: field(e, 0x16),
            b_tilde_aud_s: field(e, 0x17),
            r_tilde_aud_s: field(e, 0x18),
        },
        proof: Bytes::new(e),
    }
    .to_xdr(e)
}

pub fn withdraw_data(e: &Env) -> Bytes {
    WithdrawData {
        payload: WithdrawPayload {
            c_spend_new: point(e),
            b_tilde: field(e, 0xaa),
            r_e_point: point(e),
            sigma: field(e, 0xbb),
            b_tilde_aud_s: field(e, 0xcc),
            r_tilde_aud_s: field(e, 0xdd),
        },
        proof: Bytes::new(e),
    }
    .to_xdr(e)
}

// ----- mocks: the same shape upstream uses in its own unit tests -----

#[contract]
pub struct MockVerifier;

#[contractimpl(contracttrait)]
impl ConfidentialVerifier for MockVerifier {
    fn register_verification_key(_e: &Env, _c: CircuitType, _vk: Bytes, _op: Address) {}
    fn update_verification_key(_e: &Env, _c: CircuitType, _vk: Bytes, _op: Address) {}
    fn verify_proof(_e: &Env, _c: CircuitType, _pi: Bytes, _proof: Bytes) -> bool {
        true
    }
}

#[contract]
pub struct MockAuditor;

#[contractimpl(contracttrait)]
impl ConfidentialAuditor for MockAuditor {
    fn register_key(e: &Env, auditor_id: u32, point: BytesN<64>, _op: Address) {
        stellar_tokens::confidential::auditor::storage::register_key(e, auditor_id, &point);
    }
    fn rotate_key(e: &Env, auditor_id: u32, point: BytesN<64>, _op: Address) {
        stellar_tokens::confidential::auditor::storage::rotate_key(e, auditor_id, &point);
    }
}

/// Lets every account through, so these tests see only the Cell rules. The
/// real policy is tested in `fund-policy` and end to end in `settlement`.
#[contract]
pub struct AllowPolicy;

#[contractimpl]
impl Policy for AllowPolicy {
    fn is_authorized(_e: Env, _a: Address, _t: Address) -> bool {
        true
    }
}

/// Stand-in for the settlement contract: the only path a sealed transfer may take.
#[contract]
pub struct FakeSettlement;

#[contractimpl]
impl FakeSettlement {
    pub fn relay_transfer(e: &Env, token: Address, from: Address, to: Address, data: Bytes) {
        e.current_contract_address().require_auth();
        ConfidentialTokenClient::new(e, &token).confidential_transfer(&from, &to, &data);
    }
}

// ----- harness -----

/// One wrapper over a Stellar asset, with mock verifier, mock auditor
/// (keys `FUND_AUDITOR` and `OTHER_AUDITOR`), an allow-all policy and a fake
/// settlement contract.
pub struct Harness<'a> {
    pub e: Env,
    pub token: ConfidentialTokenClient<'a>,
    pub token_addr: Address,
    pub sac: StellarAssetClient<'a>,
    pub settlement: Address,
    pub settlement_account: Address,
    pub issuer: Address,
}

impl<'a> Harness<'a> {
    pub fn new() -> Self {
        let e = Env::default();
        e.mock_all_auths_allowing_non_root_auth();

        let token_admin = Address::generate(&e);
        let sac = e.register_stellar_asset_contract_v2(token_admin.clone());
        let sac = StellarAssetClient::new(&e, &sac.address());

        let verifier = e.register(MockVerifier, ());
        let auditor = e.register(MockAuditor, ());
        let auditor_client = ConfidentialAuditorClient::new(&e, &auditor);
        auditor_client.register_key(&FUND_AUDITOR, &point(&e), &Address::generate(&e));
        auditor_client.register_key(&OTHER_AUDITOR, &point(&e), &Address::generate(&e));
        let policy = e.register(AllowPolicy, ());
        let settlement = e.register(FakeSettlement, ());

        let admin = Address::generate(&e);
        let issuer = Address::generate(&e);
        let settlement_account = Address::generate(&e);

        let token_addr = e.register(
            SealToken,
            (
                &admin,
                &issuer,
                &sac.address,
                &verifier,
                &auditor,
                Some(ComplianceConfig {
                    policy: Some(policy),
                    sac_passthrough: false,
                }),
                SealConfig {
                    settlement: settlement.clone(),
                    settlement_account: settlement_account.clone(),
                    auditor_id: FUND_AUDITOR,
                },
            ),
        );
        let token = ConfidentialTokenClient::new(&e, &token_addr);
        Harness {
            e,
            token,
            token_addr,
            sac,
            settlement,
            settlement_account,
            issuer,
        }
    }

    pub fn register(&self, who: &Address) {
        self.token
            .register(who, &FUND_AUDITOR, &register_data(&self.e));
    }

    /// A registered account that is not the settlement account.
    pub fn investor(&self) -> Address {
        let inv = Address::generate(&self.e);
        self.register(&inv);
        inv
    }
}
