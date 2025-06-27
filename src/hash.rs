// Unfortunetely no significant difference was found between the two hashes.

#[cfg(feature = "ahash")]
pub type GlobalRandomState = ahash::RandomState;

#[cfg(not(feature = "ahash"))]
pub type GlobalRandomState = std::hash::RandomState;
