/// Test-only amount. It has no confidential-transaction semantics.
pub type Amount = u64;
pub type PaymentId = [u8; 32];
pub type TransactionContext = [u8; 32];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpendPath {
    Claim,
    Refund,
}

impl SpendPath {
    pub(crate) const fn tag(self) -> &'static [u8] {
        match self {
            Self::Claim => b"claim",
            Self::Refund => b"refund",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaymentStatus {
    Pending,
    Claimed,
    Refunded,
}
