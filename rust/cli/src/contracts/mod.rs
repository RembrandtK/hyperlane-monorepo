//! This module contains autogenerated contract interfaces.
//!
//! Autogeneration is performed by build.rs in the project root.
//!
//! Autogeneration has a dependency on contracts having been compiled in the solidity directory of the monorepo.

#![allow(warnings)]
#![allow(clippy::all)]
#![allow(missing_docs)]

mod interchain_gas_paymaster;
pub use interchain_gas_paymaster::*;

mod mailbox;
pub use mailbox::*;

mod mock_hyperlane_environment;
pub use mock_hyperlane_environment::*;

pub mod mock_mailbox;

mod test_recipient;
pub use test_recipient::*;
