use soroban_sdk::testutils::Address as _;
use soroban_sdk::Address;
use stellar_tokens::confidential::compliance::ConfidentialComplianceClient;

use super::Harness;

#[test]
fn issuer_can_freeze_and_unfreeze() {
    let h = Harness::new();
    let investor = h.investor();
    let c = ConfidentialComplianceClient::new(&h.e, &h.token_addr);

    c.freeze(&investor, &h.issuer);
    assert!(c.is_frozen(&investor));

    c.unfreeze(&investor, &h.issuer);
    assert!(!c.is_frozen(&investor));
}

#[test]
#[should_panic]
fn stranger_cannot_freeze() {
    let h = Harness::new();
    let investor = h.investor();
    let stranger = Address::generate(&h.e);
    ConfidentialComplianceClient::new(&h.e, &h.token_addr).freeze(&investor, &stranger);
}
