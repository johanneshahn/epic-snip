//! A research-only, deterministic in-memory state machine.
//!
//! It is not Epic Cash code, consensus compatible, or suitable for funds.

mod authorization;
mod descriptor;
mod errors;
mod ledger;
mod payment;
pub mod test_support;
mod types;

pub use authorization::SpendAuthorization;
pub use descriptor::PaymentDescriptor;
pub use errors::LedgerError;
pub use ledger::Ledger;
pub use payment::PendingPayment;
pub use types::{Amount, PaymentId, PaymentStatus, SpendPath, TransactionContext};
