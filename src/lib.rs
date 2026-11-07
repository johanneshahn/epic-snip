//! A research-only, deterministic in-memory state machine.
//!
//! It is not Epic Cash code, consensus compatible, or suitable for funds.

pub mod crypto;
pub mod one_sided;
mod types;

pub use crypto::{CryptoError, EpicCrypto, SpendTranscript};
pub use one_sided::{
    ClaimCapability, ClaimedOutput, OneSidedError, OneSidedLedger, OneSidedPayment, OneSidedStatus,
    ReceiverKey,
};
pub use types::{Amount, PaymentId};
