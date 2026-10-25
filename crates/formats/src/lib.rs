//! Readers for the Wonderland Classic data formats.
//!
//! The original games were written in Blitz3D, so their files are built from
//! Blitz's binary primitives (see [`blitz`]). Game data ships as a pack
//! (`res1.dat` index + `res2.dat` data, see [`pack`]) plus loose files in the
//! game folder; [`assets::GameAssets`] resolves paths across both the same way
//! the original games did.

pub mod assets;
pub mod blitz;
pub mod pack;

mod error;

pub use assets::GameAssets;
pub use error::{Error, Result};
pub use pack::{Pack, PackEntry, PackIndex};
