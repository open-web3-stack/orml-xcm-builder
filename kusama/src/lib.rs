#![cfg_attr(not(feature = "std"), no_std)]

// XCM configurations for enabling Tinkernet XCMultisigs in Kusama and parachains.
pub mod tinkernet_multisigs;
pub use tinkernet_multisigs::{
    TinkernetMultisigAsAccountId, TinkernetMultisigAsNativeOrigin, TinkernetMultisigMultiLocation,
};
