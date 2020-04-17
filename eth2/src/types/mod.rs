 /// Eth2 core domain types
#[cfg(not(feature = "local"))]
pub use eth2_types::*;
#[cfg(feature = "local")]
pub use eth2_types_local::*;






