//! How the games sequence their levels and keep the players' progress.
//! Plain Rust with no engine types, so it is tested headless.
//!
//! - [`profile`]: RTWP's player profiles (`CreateNewPlayer`, `LoadPlayer`,
//!   `SavePlayer`) and the `players` folder the player select screen lists.
//! - [`custom`]: RTWP's custom levels, the `CustomLevels` folder as the
//!   custom level menu lists it.
//! - [`worlds`]: the worlds and level lists (`data/rtwp.ron`, and TOW's
//!   `data/tow.ron` as world pack 3) and the level selector's rules.

pub mod custom;
pub mod pack;
pub mod profile;
pub mod worlds;
