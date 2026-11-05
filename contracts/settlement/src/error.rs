use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SettlementError {
    UnknownFund = 4100,
    /// The leg's token is not the fund's share wrapper.
    TokenNotInFund = 4101,
    /// A leg with this `(order_id, role)` has already been settled.
    LegAlreadySettled = 4102,
    /// Neither end of the leg is the fund's settlement account.
    LegShapeInvalid = 4103,
    // 4104 is unassigned.
    WindowIdMismatch = 4105,
    /// `net < 0` needs the settlement account's withdraw proof.
    WithdrawDataMissing = 4106,
}
