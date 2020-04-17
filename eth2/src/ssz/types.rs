#[cfg(not(feature = "local"))]
pub use eth2_ssz_types::*;
#[cfg(feature = "local")]
pub use eth2_ssz_types::*;
