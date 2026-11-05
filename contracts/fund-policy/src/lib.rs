#![no_std]
use soroban_sdk::contractmeta;

contractmeta!(key = "name", val = "cell-seal-fund-policy");
contractmeta!(
    key = "repo",
    val = "github.com/SentinelLab-VN/cell-seal-mvp"
);

mod contract;
mod event;
mod storage;
mod storage_types;

pub use contract::{FundPolicy, FundPolicyClient, COMPLIANCE_ROLE};
pub use event::InvestorUpdated;
pub use storage_types::{InvestorEntry, InvestorStatus};

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod test;
