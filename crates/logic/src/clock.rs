//! Turns wall-clock time into simulation ticks, as `MainGameLoop` does.
//!
//! The original busy-waits until more than one tick period has passed, runs
//! `elapsed / TweenPeriod` ticks and renders with the remainder as the tween
//! fraction. A frame runs at most 20 ticks; beyond that the backlog is
//! dropped. The game crate calls [`Clock::advance`] once per frame instead of
//! busy-waiting.

use crate::Ruleset;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clock {
    period: i64,
    max_elapsed: i64,
    /// `TweenTime`: the wall-clock time the simulation has reached.
    tween_time: i64,
}

/// What one frame should do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    /// Ticks to run before rendering.
    pub ticks: u32,
    /// How far into the next tick to interpolate the render, `0.0..1.0`.
    pub tween: f32,
}

impl Clock {
    /// Starts the clock as `StartLevel` does: one period in the past, so the
    /// first frame runs a tick straight away.
    pub fn new(ruleset: &Ruleset, now_ms: i64) -> Self {
        let period = i64::from(ruleset.tick_ms);
        Self {
            period,
            max_elapsed: period * i64::from(ruleset.max_catch_up_ticks),
            tween_time: now_ms - period,
        }
    }

    /// Advances to `now_ms`. Returns no ticks until more than one period has
    /// passed, where the original would still be waiting.
    pub fn advance(&mut self, now_ms: i64) -> Frame {
        let mut elapsed = now_ms - self.tween_time;
        if elapsed <= self.period {
            return Frame {
                ticks: 0,
                tween: 0.0,
            };
        }
        if elapsed > self.max_elapsed {
            elapsed = self.max_elapsed;
            self.tween_time = now_ms - elapsed;
        }
        let ticks = elapsed / self.period;
        self.tween_time += ticks * self.period;
        Frame {
            ticks: ticks as u32,
            tween: (elapsed % self.period) as f32 / self.period as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_one_tick_per_period_and_keeps_the_remainder() {
        let mut c = Clock::new(&Ruleset::rtwp(), 1000);
        assert_eq!(c.advance(1000).ticks, 0);
        let f = c.advance(1001);
        assert_eq!((f.ticks, f.tween), (1, 1.0 / 11.0));
        // 12 ms frames: the spare millisecond adds up to an extra tick.
        let ticks: u32 = (1..=11).map(|i| c.advance(1001 + 12 * i).ticks).sum();
        assert_eq!(ticks, 12);
    }

    #[test]
    fn drops_backlog_beyond_twenty_ticks() {
        let mut c = Clock::new(&Ruleset::rtwp(), 0);
        assert_eq!(c.advance(5000).ticks, 20);
        assert_eq!(c.advance(5011).ticks, 0);
        assert_eq!(c.advance(5012).ticks, 1);
    }
}
