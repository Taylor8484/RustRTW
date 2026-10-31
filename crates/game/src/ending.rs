//! The two endings' scenes as `EndBkgUpdate` and `EndBkgUpdateDeluxe` run
//! them: where everything is after each tick. [`crate::ending_scene`] draws
//! them, [`crate::menu`] counts the ticks and shows the text.
//!
//! The classic ending (menu 12) sends the cast down a turning rainbow
//! tube, one every 600 ticks, among rings of sparkles. The cycle is 7800
//! ticks; for the last 1800 the tube runs empty. The deluxe ending (menu
//! 16) stands the four players on a green floor before a starry wall with
//! the sparkles circling behind them.
//!
//! The players' idle speeds in the deluxe ending are `Rnd(0.04,0.10)` in
//! the original; here they are fixed in that range, so the scene does not
//! draw from any generator.

use bevy::math::Quat;
use wonderland_logic::md2::{self, Anim};

use crate::model::{ModelId, blitz_quat};

/// More frames than any model has: `AnimateMD2` only clamps to them.
const FRAMES: i32 = 1000;

fn sin(deg: f32) -> f32 {
    deg.to_radians().sin()
}

fn cos(deg: f32) -> f32 {
    deg.to_radians().cos()
}

/// A model on stage (`MenuEntity(61)` on).
#[derive(Debug, Clone, PartialEq)]
pub struct Figure {
    pub model: ModelId,
    /// `EntityTexture` on the copy.
    pub texture: Option<&'static str>,
    pub at: [f32; 3],
    /// Turned from the template, in Bevy space.
    pub turn: Quat,
    pub alpha: f32,
    pub anim: Anim,
}

impl Figure {
    fn new(model: ModelId, texture: Option<&'static str>, at: [f32; 3], turn: [f32; 3]) -> Self {
        Self {
            model,
            texture,
            at,
            turn: blitz_quat(turn),
            alpha: 1.0,
            anim: Anim::default(),
        }
    }

    fn animate(mut self, mode: i32, speed: f32, first: i32, last: i32) -> Self {
        self.anim.start(mode, speed, first, last, FRAMES);
        self
    }
}

/// `MenuEntity(61)`'s entrance at `t2`, the time into the cycle.
fn cast_member(t2: i32) -> Option<Figure> {
    let at = [0.0, 0.0, 100.0];
    let f = |model, texture, turn| Figure::new(model, texture, at, turn);
    let stinky = "graphics\\stinky2.bmp";
    let loof = "graphics\\loof.bmp";
    Some(match t2 {
        5 => {
            f(ModelId::Player(1), Some(stinky), [20.0, 180.0, 0.0]).animate(md2::LOOP, 0.1, 44, 52)
        }
        605 => f(ModelId::Player(2), Some(loof), [0.0, 180.0, 0.0]).animate(
            md2::PING_PONG,
            0.1,
            38,
            44,
        ),
        1205 => f(ModelId::Cannon, None, [0.0, 0.0, 180.0]),
        1805 => f(ModelId::Coily, None, [0.0; 3]),
        2405 => f(ModelId::Bomb, None, [0.0; 3]),
        3005 => f(ModelId::ZBot, None, [-90.0, 0.0, 0.0]),
        3605 => f(ModelId::Ufo, None, [90.0, 0.0, 0.0]),
        4205 => f(ModelId::Chomper, None, [10.0, 180.0, 0.0]).animate(md2::PING_PONG, 0.3, 0, 6),
        4805 => f(
            ModelId::Chomper,
            Some("graphics\\chomper2b.bmp"),
            [10.0, 180.0, 0.0],
        )
        .animate(md2::PING_PONG, 0.3, 0, 6),
        5405 => f(ModelId::Ghost, None, [0.0; 3]),
        _ => return None,
    })
}

/// The `TurnEntity` of the figure on stage at `t2`.
fn cast_spin(t2: i32) -> Option<[f32; 3]> {
    match t2 {
        611..=1199 => Some([0.0, 0.0, -2.0]),
        1811..=2399 => Some([1.0, 2.0, -2.0]),
        3011..=3599 => Some([0.0, 0.0, -3.0]),
        3611..=4199 => Some([0.0, -3.0, 0.0]),
        5411..=5999 => Some([2.0, -3.0, 1.0]),
        _ => None,
    }
}

/// One of the endings in progress.
#[derive(Debug, Clone, PartialEq)]
pub struct EndScene {
    pub deluxe: bool,
    /// Degrees the tube has turned about its length.
    pub tube_turn: f32,
    /// `PositionTexture` of the rainbow texture.
    pub rainbow: [f32; 2],
    /// `PositionTexture` of the deluxe ending's wall.
    pub wall: [f32; 2],
    /// The 50 sparkles: position, `RotateSprite` angle and sprite scale.
    pub stars: Vec<([f32; 3], f32, f32)>,
    /// What is on stage: at most one in the classic ending.
    pub figures: Vec<Figure>,
}

impl EndScene {
    pub fn new(deluxe: bool) -> Self {
        Self {
            deluxe,
            tube_turn: 0.0,
            rainbow: [0.0; 2],
            wall: [0.0; 2],
            stars: vec![([0.0, 0.0, -50.0], 0.0, 0.6); 50],
            figures: Vec::new(),
        }
    }

    /// `EndBkgUpdate` at `LevelTimer = timer`, then `UpdateWorld`.
    pub fn tick(&mut self, timer: i32) {
        self.tube_turn += 0.1;
        self.rainbow = [timer as f32 / 1000.0, timer as f32 / 2000.0];
        if self.deluxe {
            self.tick_deluxe(timer);
        } else {
            self.tick_classic(timer);
        }
        for f in &mut self.figures {
            f.anim.animate();
        }
    }

    fn tick_classic(&mut self, timer: i32) {
        let radius = 3.0 + sin(timer as f32);
        for (i, s) in self.stars.iter_mut().enumerate() {
            let k = i as i32;
            let z = k * 5 - (timer - k * 5) % 250;
            let turn = (k * 36) as f32;
            s.0 = [radius * cos(turn), radius * sin(turn), z as f32];
            s.1 = (timer * 3 + k * 47) as f32;
        }
        let t2 = timer % 7800;
        let t = timer % 600;
        if let Some(f) = cast_member(t2) {
            self.figures = vec![f];
        }
        let Some(f) = self.figures.first_mut() else {
            return;
        };
        if let Some(spin) = cast_spin(t2) {
            f.turn *= blitz_quat(spin);
        }
        if t >= 10 && t2 < 6000 {
            f.at = [0.0, -0.6, 98.0 - t as f32 / 6.0];
            if t == 10 {
                f.alpha = 1.0;
            }
            if t > 550 {
                f.alpha = (600.0 - t as f32) / 50.0;
            }
        }
    }

    fn tick_deluxe(&mut self, timer: i32) {
        for (i, s) in self.stars.iter_mut().enumerate() {
            let k = i as i32;
            let fi = i as f32;
            let a = (timer as f32 - fi * 16.0) / 16.0;
            s.0 = [
                24.0 * cos(a),
                8.0 * sin(a) + fi / 40.0 * sin((k * 57 + k * k * 15) as f32),
                25.0 + fi / 100.0,
            ];
            s.1 = (timer + k * 47) as f32;
            s.2 = if i > 0 { 0.4 } else { 1.0 };
        }
        if timer == 0 {
            self.figures = (1..=4)
                .map(|i| {
                    let fi = i as f32;
                    Figure::new(
                        ModelId::Player(i),
                        None,
                        [-3.75 + fi * 1.5, 0.0, 0.0],
                        [20.0, 180.0 + 25.0 - fi * 10.0, 0.0],
                    )
                    .animate(md2::LOOP, 0.03 + 0.015 * fi, 44, 52)
                })
                .collect();
        }
        self.wall = [
            (timer % 10000) as f32 / 10000.0,
            (timer % 20000) as f32 / 20000.0,
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(s: &mut EndScene, from: i32, to: i32) {
        for t in from..to {
            s.tick(t);
        }
    }

    #[test]
    fn the_cast_comes_down_the_tube_in_turn() {
        let mut s = EndScene::new(false);
        run(&mut s, 1, 5);
        assert!(s.figures.is_empty());
        s.tick(5);
        assert_eq!(s.figures[0].model, ModelId::Player(1));
        assert_eq!(s.figures[0].at[2], 100.0);
        run(&mut s, 6, 11);
        assert!((s.figures[0].at[2] - (98.0 - 10.0 / 6.0)).abs() < 1e-5);
        run(&mut s, 11, 576);
        assert_eq!(s.figures[0].alpha, 0.5);
        run(&mut s, 576, 606);
        assert_eq!(s.figures[0].model, ModelId::Player(2));
        assert_eq!(s.figures[0].alpha, 1.0);
        run(&mut s, 606, 5406);
        assert_eq!(s.figures[0].model, ModelId::Ghost);
    }

    #[test]
    fn the_stage_stays_empty_after_the_ghost_and_the_cycle_repeats() {
        let mut s = EndScene::new(false);
        run(&mut s, 1, 7000);
        let at = s.figures[0].at;
        run(&mut s, 7000, 7805);
        assert_eq!(s.figures[0].model, ModelId::Ghost);
        assert_eq!(s.figures[0].at, at);
        s.tick(7805);
        assert_eq!(s.figures[0].model, ModelId::Player(1));
    }

    #[test]
    fn sparkles_run_towards_the_camera_and_wrap() {
        let mut s = EndScene::new(false);
        s.tick(10);
        assert_eq!(s.stars[0].0[2], -10.0);
        assert_eq!(s.stars[3].0[2], 15.0 + 5.0);
        s.tick(250);
        assert_eq!(s.stars[0].0[2], 0.0);
    }

    #[test]
    fn the_deluxe_ending_lines_up_the_four_players() {
        let mut s = EndScene::new(true);
        s.tick(0);
        assert_eq!(s.figures.len(), 4);
        assert_eq!(s.figures[0].at, [-2.25, 0.0, 0.0]);
        assert_eq!(s.figures[3].at, [2.25, 0.0, 0.0]);
        for f in &s.figures {
            assert!((0.04..=0.10).contains(&f.anim.speed));
        }
        assert_eq!(s.stars[0].2, 1.0);
        assert_eq!(s.stars[1].2, 0.4);
    }
}
