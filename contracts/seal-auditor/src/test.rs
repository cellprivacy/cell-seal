use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
use stellar_contract_utils::crypto::grumpkin::Grumpkin;

use crate::contract::{SealAuditor, SealAuditorClient};

const GRUMPKIN_G_BYTES: [u8; 64] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xcf, 0x13, 0x5e, 0x75, 0x06, 0xa4, 0x5d, 0x63,
    0x2d, 0x27, 0x0d, 0x45, 0xf1, 0x18, 0x12, 0x94, 0x83, 0x3f, 0xc4, 0x8d, 0x82, 0x3f, 0x27, 0x2c,
];

fn setup(e: &Env) -> (SealAuditorClient<'_>, Address) {
    e.mock_all_auths();
    let admin = Address::generate(e);
    let manager = Address::generate(e);
    let addr = e.register(SealAuditor, (&admin, &manager));
    (SealAuditorClient::new(e, &addr), manager)
}

#[test]
fn registers_and_rotates_a_fund_key() {
    let e = Env::default();
    let (client, manager) = setup(&e);
    let g = BytesN::from_array(&e, &GRUMPKIN_G_BYTES);
    let two_g = Grumpkin::add(&e, &g, &g);

    client.register_key(&1, &g, &manager);
    assert_eq!(client.get_key(&1), g);

    client.rotate_key(&1, &two_g, &manager);
    assert_eq!(client.get_key(&1), two_g);
}

#[test]
#[should_panic]
fn rejects_an_off_curve_point() {
    let e = Env::default();
    let (client, manager) = setup(&e);
    client.register_key(&1, &BytesN::from_array(&e, &[7u8; 64]), &manager);
}
