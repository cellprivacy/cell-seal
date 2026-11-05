#![no_std]
use soroban_sdk::contractmeta;

contractmeta!(key = "name", val = "cell-seal-share-token");
contractmeta!(
    key = "repo",
    val = "github.com/SentinelLab-VN/cell-seal-mvp"
);

mod contract;

pub use contract::{ShareToken, ShareTokenClient, DECIMALS, ISSUER_ROLE};

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod test;
