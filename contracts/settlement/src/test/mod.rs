mod cycle;
mod leg;
mod window;

use soroban_sdk::{
    contract, contractimpl,
    testutils::{Address as _, Ledger},
    xdr::ToXdr,
    Address, Bytes, BytesN, Env, String,
};
use stellar_tokens::confidential::{
    auditor::ConfidentialAuditorClient,
    compliance::ComplianceConfig,
    verifier::{CircuitType, ConfidentialVerifier},
    ConfidentialTokenClient, RegisterData, RegisterPayload, TransferData, TransferPayload,
    WithdrawData, WithdrawPayload,
};

use crate::{FundConfig, Leg, LegRole, Settlement, SettlementClient};

pub const AUDITOR_ID: u32 = 1;

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

pub fn shares(n: i128) -> i128 {
    n * 10_000_000
}

pub fn order_id(e: &Env, n: u8) -> BytesN<32> {
    BytesN::from_array(e, &[n; 32])
}

// ----- mocks -----

/// Accepts every proof, the way upstream's own unit tests do. The real
/// verifier is exercised in `seal-verifier`.
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

// ----- harness -----

/// Every real Cell Seal contract, wired the way `scripts/deploy_testnet.sh`
/// wires them, with one fund registered and its settlement account enrolled
/// on the wrapper.
///
/// All authorization is mocked, including the two-signer `close_window`.
pub struct Fund<'a> {
    pub e: Env,
    pub settlement: SettlementClient<'a>,
    pub wrapper: ConfidentialTokenClient<'a>,
    pub wrapper_addr: Address,
    pub share: share_token::ShareTokenClient<'a>,
    pub policy: fund_policy::FundPolicyClient<'a>,
    pub fund_id: u32,
    pub settlement_account: Address,
    pub compliance: Address,
}

impl<'a> Fund<'a> {
    pub fn new() -> Self {
        let e = Env::default();
        e.mock_all_auths_allowing_non_root_auth();
        e.ledger().set_timestamp(1_000);

        let admin = Address::generate(&e);
        let platform = Address::generate(&e);
        let issuer = Address::generate(&e);
        let compliance = Address::generate(&e);
        let settlement_account = Address::generate(&e);

        let verifier = e.register(MockVerifier, ());
        let auditor = e.register(seal_auditor::SealAuditor, (&admin, &platform));
        ConfidentialAuditorClient::new(&e, &auditor).register_key(
            &AUDITOR_ID,
            &point(&e),
            &platform,
        );

        let settlement_addr = e.register(Settlement, (&admin, &platform));
        let policy_addr = e.register(fund_policy::FundPolicy, (&admin, &compliance));

        let share_addr = e.register(
            share_token::ShareToken,
            (
                &admin,
                &issuer,
                String::from_str(&e, "Meridian T-Bill Fund"),
                String::from_str(&e, "MTF"),
            ),
        );
        let wrapper_addr = e.register(
            seal_token::SealToken,
            (
                &admin,
                &issuer,
                &share_addr,
                &verifier,
                &auditor,
                Some(ComplianceConfig {
                    policy: Some(policy_addr.clone()),
                    sac_passthrough: false,
                }),
                seal_token::SealConfig {
                    settlement: settlement_addr.clone(),
                    settlement_account: settlement_account.clone(),
                    auditor_id: AUDITOR_ID,
                },
            ),
        );

        let settlement = SettlementClient::new(&e, &settlement_addr);
        let fund_id = settlement.register_fund(
            &FundConfig {
                share_token: share_addr.clone(),
                share_wrapper: wrapper_addr.clone(),
                settlement_account: settlement_account.clone(),
                issuer_admin: issuer.clone(),
            },
            &platform,
        );
        let policy = fund_policy::FundPolicyClient::new(&e, &policy_addr);
        policy.register_fund(&fund_id, &wrapper_addr, &settlement_account, &compliance);

        let wrapper = ConfidentialTokenClient::new(&e, &wrapper_addr);
        wrapper.register(&settlement_account, &AUDITOR_ID, &register_data(&e));

        let share = share_token::ShareTokenClient::new(&e, &share_addr);
        Fund {
            e,
            settlement,
            wrapper,
            wrapper_addr,
            share,
            policy,
            fund_id,
            settlement_account,
            compliance,
        }
    }

    /// Whitelist a fresh investor and enrol them on the wrapper.
    pub fn onboard_investor(&self) -> Address {
        let inv = Address::generate(&self.e);
        self.policy.set_investor(
            &self.fund_id,
            &inv,
            &fund_policy::InvestorStatus::Approved,
            &9_999,
            &self.compliance,
        );
        self.wrapper
            .register(&inv, &AUDITOR_ID, &register_data(&self.e));
        inv
    }

    /// Mint and wrap `n` shares into the settlement account, then merge so
    /// they are spendable.
    pub fn wrap(&self, window_id: u32, n: i128) {
        self.settlement
            .close_window(&self.fund_id, &window_id, &shares(n), &None);
        self.wrapper.merge(&self.settlement_account);
    }

    pub fn share_leg(&self, from: &Address, to: &Address) -> Leg {
        Leg {
            token: self.wrapper_addr.clone(),
            from: from.clone(),
            to: to.clone(),
            data: transfer_data(&self.e),
        }
    }

    pub fn submit(&self, order: u8, leg: &Leg) {
        self.settlement.submit_leg(
            &self.fund_id,
            &order_id(&self.e, order),
            &LegRole::Share,
            leg,
        );
    }
}
