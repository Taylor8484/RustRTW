//! How the games sequence their levels and keep the players' progress.
//! Plain Rust with no engine types, so it is tested headless.
//!
//! - [`profile`]: RTWP's player profiles (`CreateNewPlayer`, `LoadPlayer`,
//!   `SavePlayer`) and the `players` folder the player select screen lists.

pub mod profile;
