use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SealTokenError {
    /// `register` named an auditor other than the fund's.
    AuditorNotApproved = 4000,
    /// `deposit` by anyone but the settlement account, or to anyone else.
    DepositNotAllowed = 4001,
    /// `withdraw` by anyone but the settlement account.
    WithdrawNotAllowed = 4002,
    // 4003 is unassigned.
    /// `set_spender` naming anyone but the settlement account.
    SpenderNotAllowed = 4004,
    /// The instance has no `SealConfig`; only reachable on a broken deploy.
    ConfigNotSet = 4005,
}
