//! The deterministic simulation of a Wonderland level.
//!
//! Plain Rust with no engine: [`World::start`] sets a level up from
//! [`wonderland_formats::Level`], [`World::step`] runs one fixed tick from an
//! [`Input`], and the game crate renders the resulting state. [`Clock`]
//! decides how many ticks each rendered frame runs.
//!
//! Matching the originals tick for tick takes their random number generator
//! ([`BlitzRng`]) drawn in the same order, including the draws made only for
//! particles and lighting effects.

pub mod block;
pub mod clock;
pub mod grid;
pub mod input;
pub mod md2;
mod pending;
pub mod player;
pub mod rng;
pub mod ruleset;
pub mod setup;
pub mod sign;
pub mod world;

pub use block::{Block, Boxer, Transporter};
pub use clock::{Clock, Frame};
pub use grid::Grid;
pub use input::Input;
pub use pending::{Item, Monster};
pub use player::{Player, Skin};
pub use rng::BlitzRng;
pub use ruleset::Ruleset;
pub use setup::{Setup, Spawn};
pub use world::{Completion, Event, Options, Session, Sound, World};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The level cannot be played.
    Level(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Level(msg) => write!(f, "level: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
