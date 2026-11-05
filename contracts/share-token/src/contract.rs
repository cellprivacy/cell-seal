use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Env, MuxedAddress, String, Symbol, Vec,
};
use stellar_access::access_control::{self as access_control, AccessControl};
use stellar_macros::only_role;
use stellar_tokens::fungible::{
    burnable::FungibleBurnable,
    total_supply::{FungibleTotalSupply, TotalSupply},
    Base, Compose, FungibleToken,
};

pub const ISSUER_ROLE: Symbol = symbol_short!("issuer");
pub const DECIMALS: u32 = 7;

#[contract]
pub struct ShareToken;

#[contractimpl]
impl ShareToken {
    pub fn __constructor(e: &Env, admin: Address, issuer: Address, name: String, symbol: String) {
        Base::set_metadata(e, DECIMALS, name, symbol);
        access_control::set_admin(e, &admin);
        access_control::grant_role_no_auth(e, &issuer, &ISSUER_ROLE, &admin);
    }

    #[only_role(operator, "issuer")]
    pub fn mint(e: &Env, to: Address, amount: i128, operator: Address) {
        <Self as FungibleToken>::ContractType::mint(e, &to, amount);
    }
}

#[contractimpl(contracttrait)]
impl FungibleToken for ShareToken {
    type ContractType = Compose<(TotalSupply,)>;
}

#[contractimpl(contracttrait)]
impl FungibleTotalSupply for ShareToken {}

#[contractimpl(contracttrait)]
impl FungibleBurnable for ShareToken {}

#[contractimpl(contracttrait)]
impl AccessControl for ShareToken {}
