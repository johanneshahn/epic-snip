use crate::{PaymentId, SpendPath};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LedgerError {
    UnknownPayment(PaymentId),
    DuplicatePayment(PaymentId),
    NonceExhausted,
    AlreadySpent,
    WrongPath {
        expected: SpendPath,
        actual: SpendPath,
    },
    InvalidSignature,
    InvalidPaymentBinding,
    RefundTooEarly {
        current_height: u64,
        refund_height: u64,
    },
    HeightCannotDecrease {
        current_height: u64,
        requested_height: u64,
    },
}
