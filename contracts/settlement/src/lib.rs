#![no_std]
use soroban_sdk::contractmeta;

contractmeta!(key = "name", val = "cell-seal-settlement");
contractmeta!(
    key = "repo",
    val = "github.com/SentinelLab-VN/cell-seal-mvp"
);

mod contract;
mod error;
mod event;
mod storage;
mod storage_types;

pub use contract::{Settlement, SettlementClient, PLATFORM_ROLE};
pub use error::SettlementError;
pub use event::{FundRegistered, LegSettled, WindowClosed};
pub use storage_types::{FundConfig, Leg, LegRole, WindowState};

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod test;
