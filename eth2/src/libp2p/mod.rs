#[cfg(not(feature = "local"))]
pub use eth2_libp2p::*;
#[cfg(feature = "local")]
pub use eth2_libp2p_local::*;

pub mod discovery;
pub mod rpc;
pub mod types;


