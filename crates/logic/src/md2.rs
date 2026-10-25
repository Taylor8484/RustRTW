//! Blitz3D's MD2 animation state (`AnimateMD2`, `MD2Animating`).
//!
//! Ported from `MD2Model::startMD2Anim` and `MD2Model::animate` in
//! github.com/blitz-research/blitz3d (`blitz3d/md2model.cpp`). The logic
//! needs it because `MovePlayer` restarts a walk only when the current
//! animation has stopped, and a one-shot animation stops by itself. The
//! renderer reads the frame from [`Anim::time`]. Transitions (the `trans`
//! argument) are never used by the games.
//!
//! `UpdateWorld` advances every shown entity by one step per tick; hidden
//! entities are disabled and do not animate.

/// `AnimateMD2` modes.
pub const STOP: i32 = 0;
pub const LOOP: i32 = 1;
pub const PING_PONG: i32 = 2;
pub const ONE_SHOT: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anim {
    /// 0 once stopped.
    pub mode: i32,
    pub speed: f32,
    pub first: i32,
    pub last: i32,
    /// The current frame, fractional.
    pub time: f32,
}

impl Default for Anim {
    /// A copied entity: not animating, on frame 0.
    fn default() -> Self {
        Self {
            mode: STOP,
            speed: 0.0,
            first: 0,
            last: 0,
            time: 0.0,
        }
    }
}

impl Anim {
    /// `AnimateMD2 entity, mode, speed, first, last` on a model with
    /// `frames` frames.
    pub fn start(&mut self, mode: i32, speed: f32, first: i32, last: i32, frames: i32) {
        let (first, last) = if last < first {
            (last, first)
        } else {
            (first, last)
        };
        let clamp = |f: i32| f.clamp(0, frames - 1);
        let (first, last) = (clamp(first), clamp(last));
        self.first = first;
        self.last = last;
        self.speed = speed;
        self.time = if mode == LOOP || speed >= 0.0 {
            first as f32
        } else {
            last as f32
        };
        self.mode = mode;
        if speed == 0.0 || last == first {
            self.mode = STOP;
        }
    }

    /// `MD2Animating`.
    pub fn animating(&self) -> bool {
        self.mode != STOP
    }

    /// One `UpdateWorld` step.
    pub fn animate(&mut self) {
        if self.mode == STOP {
            return;
        }
        let (first, last) = (self.first as f32, self.last as f32);
        let len = (self.last - self.first) as f32;
        self.time += self.speed;
        if self.time < first {
            match self.mode {
                LOOP => self.time += len,
                PING_PONG => {
                    self.time = first + (first - self.time);
                    self.speed = -self.speed;
                }
                _ => {
                    self.time = first;
                    self.mode = STOP;
                }
            }
        } else if self.time >= last {
            match self.mode {
                LOOP => self.time -= len,
                PING_PONG => {
                    self.time = last - (self.time - last);
                    self.speed = -self.speed;
                }
                _ => {
                    self.time = last;
                    self.mode = STOP;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shot_stops_on_its_last_frame() {
        let mut a = Anim::default();
        assert!(!a.animating());
        a.start(ONE_SHOT, 0.2, 36, 42, 147);
        let mut steps = 0;
        while a.animating() {
            a.animate();
            steps += 1;
        }
        // 0.2 as an f32 is a little over, so 30 steps reach 42.
        assert_eq!(steps, 30);
        assert_eq!(a.time, 42.0);
    }

    #[test]
    fn stop_and_clamping() {
        let mut a = Anim::default();
        a.start(LOOP, 0.1, 105, 147, 147);
        assert_eq!((a.first, a.last), (105, 146));
        for _ in 0..1000 {
            a.animate();
            assert!((105.0..146.0).contains(&a.time));
        }
        a.start(STOP, 1.0, 0, 0, 147);
        assert!(!a.animating());
        // A single-frame range never animates.
        a.start(LOOP, 1.0, 3, 3, 147);
        assert!(!a.animating());
    }
}
