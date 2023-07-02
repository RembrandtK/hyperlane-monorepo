//! Provider AnvilWrapper for an AnvilInstance that implements Debug.

use ethers::utils::AnvilInstance;
use std::{
    fmt::{self, Debug, Formatter},
    ops::{Deref, DerefMut},
};

/// Wrapper for AnvilInstance that implements Debug.
pub struct AnvilInstanceWrapper {
    anvil: AnvilInstance,
}

impl AnvilInstanceWrapper {
    /// Create a new AnvilInstanceWrapper.
    pub fn new(anvil: AnvilInstance) -> Self {
        Self { anvil }
    }
}

impl Deref for AnvilInstanceWrapper {
    type Target = AnvilInstance;

    fn deref(&self) -> &Self::Target {
        &self.anvil
    }
}

impl DerefMut for AnvilInstanceWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.anvil
    }
}

impl Debug for AnvilInstanceWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnvilInstanceWrapper")
            .field("chain_id", &self.chain_id())
            .finish()
    }
}
