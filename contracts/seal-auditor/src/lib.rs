#![no_std]
use soroban_sdk::contractmeta;

contractmeta!(key = "name", val = "cell-seal-auditor");
contractmeta!(
    key = "repo",
    val = "github.com/SentinelLab-VN/cell-seal-mvp"
);

mod contract;

pub use contract::{SealAuditor, SealAuditorClient, MANAGER_ROLE};

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod test;
