#[cfg(not(feature = "local"))]
pub use eth2_testnet_config::*;
#[cfg(feature = "local")]
pub use eth2_testnet_config::*;
