#[cfg(not(feature = "local"))]
pub use eth2_libp2p::rpc::*;
#[cfg(feature = "local")]
pub use eth2_libp2p_local::rpc::*;
