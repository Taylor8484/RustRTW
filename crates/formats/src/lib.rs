//! Readers for the Wonderland Classic data formats.
//!
//! The original games were written in Blitz3D, so their files are built from
//! Blitz's binary primitives (see [`blitz`]). Game data ships as a pack
//! (`res1.dat` index + `res2.dat` data, see [`pack`]) plus loose files in the
//! game folder; [`assets::GameAssets`] resolves paths across both the same way
//! the original games did.

pub mod assets;
pub mod blitz;
pub mod config;
pub mod level;
pub mod pack;
pub mod save;

mod error;

pub use assets::GameAssets;
pub use config::{ExtraCfg, Options, StoryDat};
pub use error::{Error, Result};
pub use level::Level;
pub use pack::{Pack, PackEntry, PackIndex};
pub use save::PlayerFile;

/// One of the three games. Files shared between them are not always read the
/// same way, so parsers that differ take the game that loads the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Game {
    /// The Original Wonderland.
    Tow,
    /// Return to Wonderland Platinum.
    Rtwp,
    /// Wonderland Secret Worlds.
    Wsw,
}
