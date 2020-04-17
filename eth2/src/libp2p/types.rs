#[cfg(not(feature = "local"))]
pub use eth2_libp2p::types::*;
#[cfg(feature = "local")]
pub use eth2_libp2p_local::types::*;