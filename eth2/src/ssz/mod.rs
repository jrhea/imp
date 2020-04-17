pub mod ssz {
    #[cfg(not(feature = "local"))]
    pub use eth2_ssz::*;
    #[cfg(feature = "local")]
    pub use eth2_ssz_local::*;
}

pub mod derive;
pub mod types;





