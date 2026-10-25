//! Blitz3D's random number generator (`Rnd`, `Rand`, `SeedRnd`, `RndSeed`).
//!
//! Ported from `bbruntime/bbmath.cpp` in github.com/blitz-research/blitz3d:
//! a Park–Miller generator (multiplier 48271) computed with Schrage's method.
//! Each `Rnd` or `Rand` call draws once, whatever its range.
//!
//! None of the games call `SeedRnd`, so every run starts from Blitz's default
//! seed of `0x1234`. The menus draw from the same generator, so the state at
//! level start depends on the menu path; recordings must log `RndSeed()` when
//! a level starts.
//!
//! The runtime was built for x87, where the float products below stay in
//! extended precision until they are stored. They are computed in `f64` here,
//! which holds them exactly, and rounded once to `f32`.

const A: i32 = 48271;
const M: i32 = 2147483647;
const Q: i32 = 44488;
const R: i32 = 3399;

/// The default seed the Blitz runtime starts with.
pub const DEFAULT_SEED: i32 = 0x1234;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlitzRng {
    state: i32,
}

impl Default for BlitzRng {
    fn default() -> Self {
        Self::new(DEFAULT_SEED)
    }
}

impl BlitzRng {
    /// `SeedRnd seed`. A seed of 0 becomes 1.
    pub fn new(seed: i32) -> Self {
        let seed = seed & 0x7fff_ffff;
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    /// `RndSeed()`: the current state, which [`BlitzRng::new`] restores.
    pub fn seed(&self) -> i32 {
        self.state
    }

    /// The runtime's `rnd()`: a float in (0, 1) with 16 bits of randomness.
    fn next(&mut self) -> f64 {
        self.state = A * (self.state % Q) - R * (self.state / Q);
        if self.state < 0 {
            self.state += M;
        }
        f64::from(self.state & 65535) / 65536.0 + 0.5 / 65536.0
    }

    /// `Rand(from, to)`: an int in `from..=to` (the bounds may be swapped).
    pub fn rand(&mut self, from: i32, to: i32) -> i32 {
        let (from, to) = if to < from { (to, from) } else { (from, to) };
        (self.next() * f64::from(to - from + 1)) as i32 + from
    }

    /// `Rnd(from, to)`: a float in `from..to`.
    pub fn rnd(&mut self, from: f32, to: f32) -> f32 {
        let (from, to) = (f64::from(from), f64::from(to));
        (self.next() * (to - from) + from) as f32
    }

    /// Draws `n` times and ignores the results, for calls whose values only
    /// feed visuals (particles) that the renderer generates itself.
    pub fn skip(&mut self, n: usize) {
        for _ in 0..n {
            self.next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_park_miller() {
        let mut rng = BlitzRng::default();
        let mut s: u64 = 0x1234;
        for _ in 0..10_000 {
            rng.next();
            s = s * 48271 % 2147483647;
            assert_eq!(rng.seed() as u64, s);
        }
    }

    #[test]
    fn seeding() {
        assert_eq!(BlitzRng::new(0).seed(), 1);
        assert_eq!(BlitzRng::new(-1).seed(), 0x7fff_ffff);
        let mut a = BlitzRng::new(99);
        a.rand(0, 10);
        let mut b = BlitzRng::new(a.seed());
        assert_eq!(a.rand(0, 1000), b.rand(0, 1000));
    }

    #[test]
    fn ranges() {
        let mut rng = BlitzRng::default();
        let mut seen = [false; 5];
        for _ in 0..1000 {
            let v = rng.rand(4, 0);
            seen[v as usize] = true;
            let f = rng.rnd(-2.0, 3.0);
            assert!((-2.0..3.0).contains(&f));
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn first_values_from_default_seed() {
        // 0x1234 * 48271 = 224_942_860 (below M); its low 16 bits are 0x5B0C.
        let mut rng = BlitzRng::default();
        assert_eq!(rng.rand(0, 65535), 0x5B0C);
    }
}
