use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::contract::{ShareToken, ShareTokenClient};

fn shares(n: i128) -> i128 {
    n * 10_000_000
}

fn setup(e: &Env) -> (ShareTokenClient<'_>, Address) {
    e.mock_all_auths();
    let admin = Address::generate(e);
    let issuer = Address::generate(e);
    let addr = e.register(
        ShareToken,
        (
            &admin,
            &issuer,
            String::from_str(e, "Meridian T-Bill Fund"),
            String::from_str(e, "MTF"),
        ),
    );
    (ShareTokenClient::new(e, &addr), issuer)
}

#[test]
fn issuer_mints_and_holder_burns() {
    let e = Env::default();
    let (client, issuer) = setup(&e);
    let holder = Address::generate(&e);

    client.mint(&holder, &shares(1_000), &issuer);
    assert_eq!(client.balance(&holder), shares(1_000));
    assert_eq!(client.total_supply(), shares(1_000));
    assert_eq!(client.decimals(), 7);

    client.burn(&holder, &shares(400));
    assert_eq!(client.balance(&holder), shares(600));
    assert_eq!(client.total_supply(), shares(600));
}

#[test]
#[should_panic]
fn non_issuer_cannot_mint() {
    let e = Env::default();
    let (client, _issuer) = setup(&e);
    let stranger = Address::generate(&e);
    client.mint(&stranger, &1, &stranger);
}
