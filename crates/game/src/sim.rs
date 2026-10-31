//! Runs the simulation: each frame asks `logic::Clock` how many ticks have
//! passed on the real clock, as `MainGameLoop` does, reads the keyboard and
//! steps the world that many times.

use bevy::prelude::*;
use wonderland_logic::world::Lights;
use wonderland_logic::{Clock, Completion, Event, Input, Session, World};

use crate::flow::Phase;
use crate::game_menu::GameMenu;
use crate::level::{GameData, LevelData};
use crate::tow_ending::TowEnding;

/// The level in play.
#[derive(Resource)]
pub struct Sim {
    pub world: World,
    clock: Clock,
    /// Ticks run this frame.
    pub ticks: u32,
    /// How far into the next tick this frame is drawn (0–1). The original
    /// renders with `RenderWorld(tween)`; interpolating needs the previous
    /// tick's state, which the renderer does not keep yet.
    pub tween: f32,
    /// The events of every tick run this frame, in order.
    pub events: Vec<Event>,
    /// The in-game menu, while the level stands still behind it.
    pub menu: Option<GameMenu>,
    /// TOW's ending, when that is what the level is playing.
    pub ending: Option<TowEnding>,
}

impl Sim {
    /// `StartLevel` for `level`, carrying `session` over.
    pub fn start(
        data: &GameData,
        level: &LevelData,
        session: Session,
        now_ms: i64,
    ) -> Result<Self, String> {
        let world = World::start(&level.level, level.ruleset.clone(), data.options, session)
            .map_err(|e| format!("{}: {e}", level.name))?;
        let events = world.events().to_vec();
        Ok(Self {
            world,
            clock: Clock::new(&data.ruleset, now_ms),
            ticks: 0,
            tween: 0.0,
            events,
            menu: None,
            ending: None,
        })
    }

    /// The lights to draw by: `StartMenu(5)` dims the level until the next
    /// tick sets them back.
    pub fn lights(&self) -> Lights {
        let mut lights = self.world.lights;
        if self.menu.is_some() {
            lights.light = [0; 3];
            lights.ambient = [100; 3];
        }
        lights
    }
}

/// Texture scrolling `UpdateGame` does each tick, kept as the original's
/// float accumulators so the drift matches.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct TickAnim {
    /// `WaterTexU`, `WaterTexV`: the water and lava scroll, and the sky.
    pub water: [f32; 2],
    /// `ConveyorTex(1)`, `(2)`: how far each conveyor set has scrolled.
    pub conveyor: [f32; 2],
}

impl TickAnim {
    /// One tick's worth, as in `UpdateGame`.
    pub fn tick(&mut self, conveyor_active: [bool; 2], base_speed: i32) {
        self.water[0] += 0.001;
        self.water[1] += 0.0005;
        // `If WaterTexU = 1.0`: exact, so in practice it never wraps.
        for w in &mut self.water {
            if *w == 1.0 {
                *w = 0.0;
            }
        }
        for (c, active) in self.conveyor.iter_mut().zip(conveyor_active) {
            if active {
                *c += 1.0 / base_speed as f32 * 2.0;
                if *c >= 1.0 {
                    *c -= 1.0;
                }
            }
        }
    }
}

/// The keys `UpdateGame` reads: arrows (or the keypad), Shift for the free
/// camera, Return to switch player.
pub fn read_input(keys: &ButtonInput<KeyCode>) -> Input {
    let any = |k: &[KeyCode]| k.iter().any(|&k| keys.pressed(k));
    Input {
        up: any(&[KeyCode::ArrowUp, KeyCode::Numpad8]),
        down: any(&[KeyCode::ArrowDown, KeyCode::Numpad2]),
        left: any(&[KeyCode::ArrowLeft, KeyCode::Numpad4]),
        right: any(&[KeyCode::ArrowRight, KeyCode::Numpad6]),
        camera: any(&[KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        switch: any(&[KeyCode::Enter, KeyCode::NumpadEnter]),
    }
}

/// Runs in `Update` before anything that reads the world.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimStep;

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TickAnim>()
            .add_systems(Update, step.in_set(SimStep).run_if(in_state(Phase::Game)));
    }
}

fn now_ms(time: &Time<Real>) -> i64 {
    time.elapsed().as_millis() as i64
}

fn step(
    mut sim: ResMut<Sim>,
    mut anim: ResMut<TickAnim>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    data: Res<GameData>,
) {
    let frame = sim.clock.advance(now_ms(&time));
    let input = read_input(&keys);
    sim.events.clear();
    // `MenuLoop5` keeps the clock going but runs no ticks.
    sim.ticks = if sim.menu.is_some() { 0 } else { frame.ticks };
    sim.tween = frame.tween;
    for _ in 0..sim.ticks {
        if matches!(sim.world.completion, Completion::Done | Completion::Restart) {
            break;
        }
        let sim = &mut *sim;
        match &mut sim.ending {
            Some(ending) => ending.tick(&mut sim.world),
            None => sim.world.step(input),
        }
        let events = sim.world.events().to_vec();
        sim.events.extend(events);
        anim.tick(sim.world.conveyor_active, data.ruleset.base_speed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conveyors_scroll_a_tile_in_twelve_ticks() {
        let mut a = TickAnim::default();
        for _ in 0..12 {
            a.tick([true, false], 24);
        }
        assert!(a.conveyor[0] < 1e-5 || a.conveyor[0] > 0.99999);
        assert_eq!(a.conveyor[1], 0.0);
        assert!((a.water[0] - 0.012).abs() < 1e-6);
    }
}
