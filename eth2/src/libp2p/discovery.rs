#[cfg(not(feature = "local"))]
pub use eth2_libp2p::discovery::*;
#[cfg(feature = "local")]
pub use eth2_libp2p_local::discovery::*;