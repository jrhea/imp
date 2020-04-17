#[cfg(not(feature = "local"))]
pub use eth2_ssz_derive::*;
#[cfg(feature = "local")]
pub use eth2_ssz_local::*;