pub mod fixed_bases;
pub mod sinsemilla;
pub mod util;

pub use fixed_bases::{NullifierK, OrchardFixedBases, OrchardFixedBasesFull, ValueCommitV, H};

/// Domain prefix used for Schnorr signatures, with `hash_to_scalar`.
pub const DRK_SCHNORR_DOMAIN: &[u8] = b"DarkFi:Schnorr";

/// Domain prefix used for block hashes, with `hash_to_curve`.
pub const BLOCK_HASH_DOMAIN: &str = "DarkFi:Block";

pub const MERKLE_DEPTH_ORCHARD: usize = 32;

pub const MERKLE_DEPTH: u8 = MERKLE_DEPTH_ORCHARD as u8;

#[allow(dead_code)]
/// $\ell^\mathsf{Orchard}_\mathsf{base}$
pub(crate) const L_ORCHARD_BASE: usize = 255;

/// $\ell^\mathsf{Orchard}_\mathsf{scalar}$
pub(crate) const L_ORCHARD_SCALAR: usize = 255;

/// $\ell_\mathsf{value}$
pub(crate) const L_VALUE: usize = 64;