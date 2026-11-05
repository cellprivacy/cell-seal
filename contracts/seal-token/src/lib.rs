#![no_std]
use soroban_sdk::contractmeta;

contractmeta!(key = "name", val = "cell-seal-token");
contractmeta!(
    key = "repo",
    val = "github.com/SentinelLab-VN/cell-seal-mvp"
);

mod contract;
mod error;
mod hooks;
mod storage;
mod storage_types;

pub use contract::{SealToken, SealTokenClient, ISSUER_ROLE};
pub use error::SealTokenError;
pub use hooks::SealHooks;
pub use storage_types::SealConfig;

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod test;
