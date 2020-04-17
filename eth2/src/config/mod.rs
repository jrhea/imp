#[cfg(not(feature = "local"))]
pub use eth2_config::*;
#[cfg(feature = "local")]
pub use eth2_config_local::*;
