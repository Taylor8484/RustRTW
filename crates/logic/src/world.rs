//! The running level: `StartLevel`, `UpdateGame` and the per-frame timer in
//! `MainGameLoop` (`RTWP/game.bb`).
//!
//! [`World::step`] runs one tick in `UpdateGame`'s order. The players are
//! in [`crate::player`] and the signs in [`crate::sign`]; the other object
//! families have their own modules.
//!
//! The camera and lights live here too: the original moves them by fixed
//! amounts each tick and draws random numbers for the lightning, so the
//! renderer only reads them. Particles are left to the renderer, but their
//! random draws are made here so the generator stays in step.

use wonderland_formats::Level;

use crate::block::{Block, Boxer, Transporter};
use crate::hazard::{Cannon, Electro, Fireball, MAX_FIREBALLS, Spike};
use crate::item::Item;
use crate::monster::{Monster, Warper};
use crate::player::Player;
use crate::scenery::{Fish, Pillar};
use crate::setup::{LoadOptions, MonsterDir, Setup, Spawn};
use crate::sign::{LEVEL_NAME, ShownSign};
use crate::switch::Button;
use crate::{BlitzRng, Input, Result, Ruleset};

/// Settings from `options.cfg` and the menus that change the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// `DetailLevel`, 1–5. It sets how many particles effects create, and
    /// each particle draws random numbers, so it changes the random sequence.
    pub detail_level: i32,
    /// `NoTimeLimit`: the level timer never runs out.
    pub no_time_limit: bool,
    /// Sound is on (`OptionsSoundLevel` above 0 and a sound device). Each
    /// positional sound then nudges the camera by float rounding, so it
    /// can change the free camera's limits.
    pub sound: bool,
    /// `Language`: 0 English, 1 German, 2 French. Only English draws a
    /// random number for the level-complete praise.
    pub language: i32,
    pub load: LoadOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            detail_level: 3,
            no_time_limit: false,
            sound: true,
            language: 0,
            load: LoadOptions {
                style: None,
                background: None,
            },
        }
    }
}

/// State that outlives a level: the next level or a restart picks it up.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    pub rng: BlitzRng,
    /// `ClassicDeathSound`: deaths so far. The first two always play the
    /// classic sound.
    pub classic_death_sound: i32,
}

/// `LevelCompleted`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Completion {
    /// 0: playing.
    #[default]
    Playing,
    /// 1: won, zooming out.
    Won,
    /// 2: done, back to the menu (and on to the next level).
    Done,
    /// 3: restart the level.
    Restart,
}

/// Sounds the simulation asks for. The game crate maps them to files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sound {
    /// `SFXTalk(n)`: a player's greeting at level start (0–7), or the
    /// player taking over on a switch (11–14).
    Talk(i32),
    /// `SFXDeath`: the first player of the level is hit.
    Death,
    /// `SFXTeleport`: also played when the focus moves to a dying player.
    Teleport,
    /// `SFXIce`: setting off on ice.
    Ice,
    /// `SFXSlIce(n)`, 0–4, played at pitch 19000: sliding onto ice.
    SlideIce(i32),
    /// `SFXPlayerConvey(player)`: a player rides onto a conveyor.
    Convey(usize),
    /// `SFXTransporter`.
    Transporter,
    /// `SFXTrampoline`.
    Trampoline,
    /// `SFXpBounce(player, n)`, n 1–4.
    Bounce(usize, i32),
    /// `SFXBreakAway`: a breakaway tile cracks.
    BreakAway,
    /// `SFXBoxPush`.
    BoxPush,
    /// `SFXBoulderPush`: a boulder starts rolling.
    BoulderPush,
    /// `SFXBoulderHit`: a rolling boulder or link sphere stops.
    BoulderHit,
    /// `SFXBoxSplosh`: a box falls into water.
    BoxSplosh,
    /// `SFXLava`: a steel box sinks into lava.
    Lava,
    /// `SFXSticky`: two sticky cubes set.
    Sticky,
    /// `SFXBoxExplo`: a box breaks.
    BoxExplode,
    /// `SFXSingle(n)`, 0–16: sounds that play at most once per tick, at
    /// the loudest volume asked for (1 cannon, 13 box factory; 12 is also
    /// a prism breaking).
    Single(i32),
    /// `SFXBounce` at this pitch: a beach ball hops.
    BallBounce(i32),
    /// `SFXCoilyDeath`: a beach ball pops.
    CoilyDeath,
    /// `SFXBombExplo`: a monster blows up.
    BombExplode,
    /// `SFXSaucerDie` at this pitch: 16000 for a saucer, 10000 for the
    /// mother ship.
    SaucerDie(i32),
    /// `SFXTime`: 30 seconds left.
    Time,
    /// `SFXCount(n)`: the last ten seconds.
    Count(i32),
    /// `SFXLightning(n)`.
    Lightning(i32),
    /// `SFXButton`: a button goes down.
    Button,
    /// `SFXTimerButton`: a timer button starts or restarts.
    TimerButton,
    /// `SFXHmmm`: a teleporter's exit is blocked.
    Hmmm,
    /// `SFXPlayerCoin(player)`: a player takes a coin.
    Coin(usize),
    /// `SFXLastCoin`: the last coin is gone and the final gate opens.
    LastCoin,
    /// `SFXPlayerClock(player)`.
    Clock(usize),
    /// `SFXPlayerBonus(player)`.
    Bonus(usize),
    /// `SFXPlayerEnd(player)`: a player leaves through the exit.
    Exit(usize),
    /// `SFXLevelLose`: the level is lost.
    Lose(LoseSound),
}

/// The level-lost sound, picked when the last player dies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoseSound {
    /// `sound\death2.wav`
    Classic,
    /// `newsounds\death-2b.wav` to `death-2d.wav`
    Variant(char),
    /// `newsounds\death2-spa.wav`
    Spanish,
    /// `newsounds\death2-ger.wav`
    German,
    /// `newsounds\death2-fra.wav`
    French,
}

/// Something the renderer or audio should react to this tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Sound(Sound),
    /// `PlayMySound` at an entity: the volume (0–1] fades with distance
    /// from the camera, before the sound level is applied.
    SoundAt(Sound, f32),
    /// A dying player finished spinning: burst into particles.
    PlayerExploded(usize),
    /// A tile changed under a player or object (switcher, breakaway).
    TileChanged {
        x: i32,
        y: i32,
    },
    /// A particle effect at a Blitz position. The simulation has drawn
    /// its random numbers; the renderer makes its own.
    Effect {
        effect: Effect,
        at: [f32; 3],
    },
    /// The last player left through the exit. The original stores `score`
    /// in the profile if it beats the best, marks the level perfect when
    /// no bonus coin is left, and saves the profile.
    LevelWon {
        score: i32,
        perfect: bool,
    },
}

/// The particle effects the original creates (`CreateParticle` calls).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// `KillBox`: splinters of a box of this type.
    Splinters(i32),
    /// `KillMonster` (and a saucer landing): an explosion for this type.
    Burst(i32),
    /// A star rising from the exit gate.
    ExitStar,
    /// A coin taken.
    CoinSparkle,
    /// A bonus coin taken.
    BonusSparkle,
}

/// The camera in Blitz coordinates (x right, y up, z = -row).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub position: [f32; 3],
    pub pitch: f32,
    pub yaw: f32,
    /// `CameraRange` near plane; the far plane is always 140.
    pub near: f32,
    /// `CameraZoom`.
    pub zoom: f32,
}

impl Camera {
    fn translate(&mut self, dx: f32, dy: f32, dz: f32) {
        self.position[0] += dx;
        self.position[1] += dy;
        self.position[2] += dz;
    }
}

/// The level light and ambient light, fading towards targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lights {
    pub light: [i32; 3],
    pub light_target: [i32; 3],
    pub light_speed: i32,
    pub ambient: [i32; 3],
    pub ambient_target: [i32; 3],
    pub ambient_speed: i32,
}

impl Lights {
    fn fade_out(&mut self) {
        self.light_target = [-255; 3];
        self.light_speed = 2;
        self.ambient_target = [0; 3];
        self.ambient_speed = 1;
    }

    fn approach(current: &mut i32, target: i32, speed: i32) {
        if *current < target - speed {
            *current += speed;
        } else if *current < target {
            *current = target;
        }
        if *current > target + speed {
            *current -= speed;
        } else if *current > target {
            *current = target;
        }
    }

    fn update(&mut self) {
        for c in 0..3 {
            Self::approach(&mut self.light[c], self.light_target[c], self.light_speed);
        }
        for c in 0..3 {
            Self::approach(
                &mut self.ambient[c],
                self.ambient_target[c],
                self.ambient_speed,
            );
        }
    }
}

/// A level in play.
#[derive(Debug, Clone, PartialEq)]
pub struct World {
    pub ruleset: Ruleset,
    pub options: Options,
    pub session: Session,
    pub level: Setup,
    /// Indexed by player id 1–4, as in the original; slot 0 is unused.
    pub players: Vec<Player>,
    /// Boxes and other pushables, in creation order.
    pub blocks: Vec<Block>,
    /// Box factories.
    pub boxers: Vec<Boxer>,
    pub transporters: Vec<Transporter>,
    /// Monsters, in creation order.
    pub monsters: Vec<Monster>,
    pub warpers: Vec<Warper>,
    /// Coins, exit gates, clocks and bonus coins, in creation order.
    pub items: Vec<Item>,
    /// `NofCrystals`: coins left. The final gate opens when it reaches 0.
    pub coins: i32,
    /// `NofTotalCrystals`.
    pub total_coins: i32,
    /// `NofBonusCoins`: bonus coins left.
    pub bonus_coins: i32,
    /// `NofTotalBonusCoins`.
    pub total_bonus_coins: i32,
    pub buttons: Vec<Button>,
    /// The four electro tile types.
    pub electros: [Electro; 5],
    pub spikes: Vec<Spike>,
    pub cannons: Vec<Cannon>,
    /// The `Fireball*` slots, `MaxFireballs + 1` of them; the first
    /// `fireball_count` are in play ([`World::fireballs`]).
    pub(crate) fireball_slots: Vec<Fireball>,
    /// `NofFireballs`.
    pub(crate) fireball_count: usize,
    pub fish: Vec<Fish>,
    pub pillars: Vec<Pillar>,
    /// `HouseStyle`: the house texture, 1–4.
    pub house_style: i32,
    /// `LaserGateActive`: gates of each colour are closed.
    pub laser_gate_active: [bool; 10],
    /// The score the level-complete sign compares against, from the
    /// player's profile: the best score when above 0 for a campaign level,
    /// or the last score of a custom level played before. The original
    /// shows the comparison for custom levels in English only, so leave it
    /// `None` there in other languages. Set it after [`World::start`].
    pub record: Option<i32>,
    /// `ConveyorActive(1)` and `(2)`: conveyor tiles 5x0–5x7 and 5x8 up.
    pub conveyor_active: [bool; 2],
    /// `BothCompleted`: bit `k - 1` is set once player `k` has left through
    /// the exit.
    pub both_completed: i32,
    /// `CurrentLevelScore`: 250 per monster killed, before the bonuses.
    pub level_score: i32,
    /// `SoundPitch SFXSingle(8)`: a chomper sets it each time it picks a
    /// move.
    pub chomp_pitch: i32,
    /// `EndMovePlayerTrampoline`: the move ending is a jump landing.
    pub end_move_trampoline: bool,
    /// `View3dRotation`: the first-person heading in quarter turns.
    pub view3d_rotation: i32,
    /// `View3dRotation2`: the camera's heading in hundredths of a quarter
    /// turn, turning towards `view3d_rotation * 100`.
    pub view3d_rotation2: i32,
    pub(crate) signs_shown: Vec<ShownSign>,
    /// `SFXSingleVolume`: the loudest each single sound was asked for this
    /// tick.
    single_volume: [f32; 17],
    /// The keys held this tick; a bounce off a trampoline reads them.
    pub(crate) input: Input,
    pub camera: Camera,
    pub lights: Lights,
    /// `LevelTimer`: ticks since the level started.
    pub level_timer: i32,
    pub completion: Completion,
    pub completion_timer: i32,
    /// `MapTime`: seconds left.
    pub map_time: i32,
    /// Simulated `MilliSecs()`: `tick_ms` per tick since the level started.
    pub clock_ms: i64,
    /// `MapMilliTime`: when `map_time` last ticked down.
    pub map_ms: i64,
    pub focus_switch_timer: i32,
    pub lightning_timer: i32,
    /// `SignActive`: 0, or the sign being shown on a player.
    pub sign_active: i32,
    /// Ticks left of the final gate sinking.
    pub final_gate_anim: i32,
    /// `ShiftKeyPressed`: in free-camera mode last tick.
    pub free_camera_active: bool,
    /// `FreeCameraX`, `FreeCameraZ`: how far the free camera has moved.
    pub free_camera: [f32; 2],
    /// `CameraDX`, `CameraDZ`: a camera swoop still to do.
    pub camera_swoop: [f32; 2],
    pub(crate) events: Vec<Event>,
}

/// `CameraDStep`: the most a camera swoop moves per tick.
const CAMERA_STEP: f32 = 0.5;

impl World {
    /// `StartLevel`: loads `level` and creates its objects.
    pub fn start(
        level: &Level,
        ruleset: Ruleset,
        options: Options,
        session: Session,
    ) -> Result<Self> {
        let setup = Setup::load(level, &ruleset, options.load)?;
        let lights = Lights {
            light: [-255; 3],
            light_target: [0; 3],
            light_speed: 2,
            ambient: [0; 3],
            ambient_target: [0; 3],
            ambient_speed: 1,
        };
        let mut world = Self {
            players: vec![Player::default(); ruleset.players + 1],
            blocks: Vec::new(),
            boxers: Vec::new(),
            transporters: Vec::new(),
            single_volume: [0.0; 17],
            monsters: Vec::new(),
            warpers: Vec::new(),
            level_score: 0,
            chomp_pitch: 0,
            items: Vec::new(),
            coins: 0,
            total_coins: 0,
            bonus_coins: 0,
            total_bonus_coins: 0,
            laser_gate_active: [false; 10],
            buttons: Vec::new(),
            electros: [Electro::default(); 5],
            spikes: Vec::new(),
            cannons: Vec::new(),
            fireball_slots: vec![Fireball::default(); MAX_FIREBALLS + 1],
            fireball_count: 0,
            fish: Vec::new(),
            pillars: Vec::new(),
            house_style: 1,
            record: None,
            conveyor_active: [true; 2],
            both_completed: 0,
            end_move_trampoline: false,
            view3d_rotation: 0,
            view3d_rotation2: 0,
            signs_shown: Vec::new(),
            input: Input::default(),
            camera: Camera {
                position: [0.0; 3],
                pitch: 0.0,
                yaw: 0.0,
                near: 0.98,
                zoom: 2.0,
            },
            lights,
            level_timer: 0,
            completion: Completion::Playing,
            completion_timer: 0,
            map_time: setup.time,
            clock_ms: 0,
            // StartLevel sets TweenTime one period before MapMilliTime.
            map_ms: i64::from(ruleset.tick_ms),
            focus_switch_timer: 0,
            lightning_timer: 0,
            sign_active: 0,
            final_gate_anim: 0,
            free_camera_active: false,
            free_camera: [0.0; 2],
            camera_swoop: [0.0; 2],
            events: Vec::new(),
            level: setup,
            ruleset,
            options,
            session,
        };
        world.create_objects();
        world.create_electros();
        world.total_coins = world.coins;
        world.total_bonus_coins = world.bonus_coins;
        world.focus_first_player()?;

        // StartLevel, after LoadLevel.
        if world.level.view3d {
            world.camera.pitch = 38.0;
            world.camera.yaw = 180.0;
            world.camera.near = 0.18;
            world.camera.zoom = 1.0;
            world.view3d_rotation = 2;
            world.view3d_rotation2 = 200;
            for p in &mut world.players[1..] {
                p.view3d_rotation = 2;
            }
        } else {
            world.camera.pitch = 90.0;
        }
        // A sign under the first player keeps its dimmed lights.
        if world.sign_active == 0 {
            world.level_light_targets();
        }
        world.lights.light_speed = 2;
        world.lights.ambient_speed = 1;
        world.construct_level_tiles();
        world.check_buttons();
        world.box_already_in_water();
        let name = world.level.name.clone();
        world.start_sign(&name, 16.0, LEVEL_NAME, 0.0);
        Ok(world)
    }

    /// The tile rewrites at the end of `ConstructLevel`, which otherwise
    /// builds the level mesh (the renderer's job, from the loaded
    /// [`Setup`]): pillars, x-floors (104), box factories (16xx) and warp
    /// gates (22xx) become walls, and fake walls (15xx) become floor.
    fn construct_level_tiles(&mut self) {
        let grid = &mut self.level.grid;
        for p in &self.pillars {
            grid.set(p.x, p.y, 200);
        }
        let pass = |grid: &mut crate::Grid, f: &dyn Fn(i32) -> Option<i32>| {
            for (x, y) in grid.cells() {
                if let Some(t) = f(grid.get(x, y)) {
                    grid.set(x, y, t);
                }
            }
        };
        pass(grid, &|t| (t == 104).then_some(200));
        pass(grid, &|t| (t / 100 == 15).then_some(100));
        pass(grid, &|t| matches!(t / 100, 16 | 22).then_some(200));
    }

    /// The light targets for the level as it starts, outside lightning
    /// flashes and signs. Lightning levels are darker.
    pub(crate) fn level_light_targets(&mut self) {
        let lightning = i32::from(self.level.weather.lightning);
        let ch = self.level.channels;
        let channel = |on: bool, lit: i32| if on { lit } else { 100 };
        let targets = [
            channel(ch.red, 255 - 155 * lightning),
            channel(ch.green, 255 - 155 * lightning),
            channel(ch.blue, 255),
        ];
        self.lights.light_target = targets;
        self.lights.ambient_target = targets;
    }

    /// `PlayMySound` at a position: no sound beyond 9 units of the point
    /// the camera looks at, full volume within 3. Measuring moves the
    /// camera there and back, which can leave float rounding behind.
    pub(crate) fn play_at(&mut self, sound: Sound, at: [f32; 3]) {
        if let Some(volume) = self.hear(at) {
            self.events.push(Event::SoundAt(sound, volume));
        }
    }

    /// The volume of a sound at `at`, if it can be heard.
    fn hear(&mut self, at: [f32; 3]) -> Option<f32> {
        if !self.options.sound {
            return None;
        }
        self.camera.translate(0.0, -10.0, 9.0);
        let [x, y, z] = self.camera.position;
        let (dx, dy, dz) = (at[0] - x, at[1] - y, at[2] - z);
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        self.camera.translate(0.0, 10.0, -9.0);
        let (a, b) = (3.0, 9.0);
        let volume: f32 = (b - distance) / (b - a);
        (volume > 0.0).then(|| volume.min(1.0))
    }

    /// `PlaySingleSound`: asks for single sound `kind` from tile `(x, y)`
    /// (x = -999 for full volume). It plays at the end of the tick.
    pub(crate) fn play_single_sound(&mut self, x: i32, y: i32, kind: usize) {
        if !self.options.sound {
            return;
        }
        let volume = if x == -999 {
            1.0
        } else {
            match self.hear([x as f32 + 0.5, 0.0, -(y as f32) - 0.5]) {
                Some(v) => v.max(self.single_volume[kind]),
                None => return,
            }
        };
        self.single_volume[kind] = volume;
    }

    /// The `Create*` calls, in load order.
    fn create_objects(&mut self) {
        for i in 0..self.level.spawns.len() {
            match self.level.spawns[i] {
                Spawn::Player { id, x, y } => {
                    if let Ok(id @ 1..) = usize::try_from(id)
                        && id < self.players.len()
                    {
                        self.create_player(id, x, y);
                    }
                }
                Spawn::Box { x, y, kind } => self.create_box(x, y, kind),
                Spawn::Transporter { x, y } => self.create_transporter(x, y),
                Spawn::Boxer { x, y, kind } => self.create_boxer(x, y, kind),
                Spawn::Monster {
                    dir: MonsterDir::Random,
                    x,
                    y,
                    kind,
                } => {
                    // The loader draws the broken z-bot's direction as it
                    // scans; fix it now so later draws stay in order.
                    let d = self.session.rng.rand(0, 3);
                    self.level.spawns[i] = Spawn::Monster {
                        x,
                        y,
                        kind,
                        dir: MonsterDir::Fixed(d),
                    };
                    self.create_monster(x, y, kind, d);
                }
                Spawn::Monster {
                    dir: MonsterDir::Fixed(d),
                    x,
                    y,
                    kind,
                } => self.create_monster(x, y, kind, d),
                Spawn::Warper { x, y, kind } => self.create_warper(x, y, kind),
                Spawn::Item { x, y, kind } => self.create_item(x, y, kind),
                Spawn::LaserGate { col, .. } => self.create_laser_gate(col),
                Spawn::Button { x, y, kind } => self.create_button(x, y, kind),
                Spawn::Spike { x, y, speed } => self.create_spike(x, y, speed),
                Spawn::Cannon {
                    x,
                    y,
                    speed,
                    dx,
                    dy,
                } => self.create_cannon(x, y, speed, dx, dy),
                Spawn::Fireball { x, y, dx, dy } => self.create_fireball(x, y, dx, dy),
                Spawn::Fish { x, y } => self.create_fish(x, y),
                Spawn::Pillar { x, y, kind } => self.create_pillar(x, y, kind),
            }
        }
    }

    /// The end of `LoadLevel`: focus on the first player, greet and place
    /// the camera above them.
    fn focus_first_player(&mut self) -> Result<()> {
        let Some(k) = (1..self.players.len()).find(|&k| self.players[k].exists) else {
            return Err(crate::Error::Level("the level has no player".into()));
        };
        let talk = match k {
            3 => 6,
            4 => 7,
            _ => self.session.rng.rand(0, 4),
        };
        self.events.push(Event::Sound(Sound::Talk(talk)));
        let p = &mut self.players[k];
        p.focus = true;
        let (x, y) = (p.x as f32, p.y as f32);
        self.camera.position = if self.level.view3d {
            [x + 0.5, 10.6, -y - 0.5 + 10.0]
        } else {
            [x + 0.5, 10.4 + 10.0, -y - 9.5 + 10.0]
        };
        self.sign_active = 0;
        self.start_signs_under(k);
        Ok(())
    }

    /// Events from [`World::start`] and the last [`World::step`].
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Hands the session on to the next level or a restart.
    pub fn into_session(self) -> Session {
        self.session
    }

    fn any_dying(&self) -> bool {
        self.players.iter().any(|p| p.dying)
    }

    fn any_dead(&self) -> bool {
        self.players.iter().any(|p| p.dead)
    }

    /// Runs one tick: `UpdateGame`, then the timer check `MainGameLoop`
    /// makes after each frame's ticks.
    pub fn step(&mut self, input: Input) {
        self.events.clear();
        self.clock_ms += i64::from(self.ruleset.tick_ms);
        self.input = input;

        self.single_volume = [0.0; 17];
        self.control(input);
        if self.focus_switch_timer > 0 {
            self.focus_switch_timer -= 1;
        }
        for i in 1..self.players.len() {
            self.idle_and_blink(i);
            self.update_dying(i);
        }
        if self.level_timer < 160 {
            // Fade in.
            self.camera.translate(0.0, -10.0 / 160.0, -10.0 / 160.0);
            self.camera.pitch += -35.0 / 160.0;
            if self.level_timer > 150 && self.level.view3d {
                // First person: the camera is now inside the player.
                for p in &mut self.players[1..] {
                    if p.focus && p.exists {
                        p.hidden = true;
                    }
                }
            }
        }

        self.convey_players();
        self.convey_boxes();
        self.control_fireballs();
        for k in 1..self.players.len() {
            self.control_player(k);
        }
        self.control_boxes();
        self.control_electro_tiles();
        self.control_cannons();
        self.control_spikes();
        self.control_timer_buttons();
        self.control_monsters();
        self.control_items();
        self.control_fish();
        // ControlParticles: renderer.
        self.control_fireballs();
        self.control_boxers();
        self.control_warpers();

        self.move_players();
        self.move_transporters();
        self.move_boxes();
        self.move_monsters();
        // Transporters land first: a player may be riding one.
        self.end_transporter_moves();
        self.end_player_moves();
        self.end_box_moves();
        self.end_monster_moves();
        self.clean_up_boxes();
        self.clean_up_fireballs();
        self.clean_up_monsters();
        self.clean_up_items();

        self.swoop_camera();
        // Water, conveyor and laser gate animation: renderer (step 3.2).
        if self.completion == Completion::Won {
            self.completion_timer += 1;
            self.camera.translate(0.0, 10.0 / 512.0, 10.0 / 512.0);
            self.camera.pitch += 45.0 / 512.0;
            if self.completion_timer == 300 {
                self.completion = Completion::Done;
            }
            self.lights.light_target = [0; 3];
            self.lights.light_speed = 2;
            self.lights.ambient_target = [0; 3];
            self.lights.ambient_speed = 1;
        }
        if self.final_gate_anim > 0 {
            // The renderer sinks the gates (0.016 per tick) and hides them
            // at 0.
            self.final_gate_anim -= 1;
        }
        for p in &mut self.players[1..] {
            if p.shrink > 0 {
                // The renderer scales the player by shrink / 60.
                p.shrink -= 1;
                if p.shrink == 0 {
                    p.hidden = true;
                }
            }
        }
        for (i, &volume) in self.single_volume.iter().enumerate() {
            if volume > 0.0 {
                self.events
                    .push(Event::SoundAt(Sound::Single(i as i32), volume));
            }
        }
        self.control_signs();
        self.weather();
        self.lights.update();
        self.level_timer += 1;
        // UpdateWorld.
        self.animate_players();

        self.count_down();
    }

    /// One tick of TOW's ending: only the level's creatures run, with
    /// nobody playing; the ending sets the camera and lights itself.
    pub fn step_stage(&mut self) {
        self.events.clear();
        self.clock_ms += i64::from(self.ruleset.tick_ms);
        self.single_volume = [0.0; 17];
        self.control_monsters();
        self.control_items();
        self.control_fish();
        self.move_monsters();
        self.end_monster_moves();
        self.clean_up_monsters();
        self.clean_up_items();
        self.level_timer += 1;
        self.animate_players();
    }

    /// The input half of `UpdateGame`: free camera or player control.
    fn control(&mut self, input: Input) {
        if input.camera && !self.level.view3d {
            if !self.free_camera_active {
                self.stop_level_signs();
            }
            self.free_camera_active = true;
            let step = 0.1;
            let width = self.level.grid.width() as f32;
            let height = self.level.grid.height() as f32;
            // Each check sees the moves before it.
            if input.up && self.camera.position[2] < -1.0 - 9.5 {
                self.camera.translate(0.0, 0.0, step);
                self.free_camera[1] += step;
            }
            if input.down && self.camera.position[2] > -(height - 1.0) - 9.5 {
                self.camera.translate(0.0, 0.0, -step);
                self.free_camera[1] -= step;
            }
            if input.left && self.camera.position[0] > 1.0 {
                self.camera.translate(-step, 0.0, 0.0);
                self.free_camera[0] -= step;
            }
            if input.right && self.camera.position[0] < width - 1.0 {
                self.camera.translate(step, 0.0, 0.0);
                self.free_camera[0] += step;
            }
            return;
        }

        if self.free_camera_active {
            self.check_signs();
        }
        self.free_camera_active = false;
        let [fx, fz] = self.free_camera;
        if fx != 0.0 || fz != 0.0 {
            self.camera.translate(-fx, 0.0, -fz);
            self.free_camera = [0.0; 2];
        }
        if !self.any_dying()
            && !self.any_dead()
            && self.completion == Completion::Playing
            && self.focus_switch_timer < 20
        {
            if self.level.view3d {
                self.view3d_keyboard_control(input);
            } else {
                self.normal_keyboard_control(input);
            }
        }
        if input.switch && self.focus_switch_timer == 0 {
            let blocked = self.players.iter().any(|p| p.exists && (p.dying || p.dead));
            if !blocked {
                self.switch_player(true);
            }
        }
    }

    /// The dying and dead half of `UpdateGame`'s per-player loop.
    fn update_dying(&mut self, i: usize) {
        let p = &self.players[i];
        if p.dying && !p.dead && p.exists {
            // The renderer spins the player by (4, 6) degrees per tick.
            let lift = self.players[i].dying_timer as f32 / 2000.0;
            self.players[i].position[1] += lift;
            self.camera.translate(0.0, lift, 0.0);
            let p = &mut self.players[i];
            p.dying_timer -= 1;
            if p.dying_timer == 0 {
                p.dying_timer = 320;
                p.dead = true;
                let others_dead = self
                    .players
                    .iter()
                    .enumerate()
                    .any(|(j, q)| j != i && q.exists && q.dead);
                if !others_dead {
                    self.lose_sound();
                }
                self.players[i].hidden = true;
                self.events.push(Event::PlayerExploded(i));
                // 30*DetailLevel+1 particles, each drawing an angle, a speed,
                // a texture and a vertical speed.
                let particles = usize::try_from(30 * self.options.detail_level + 1).unwrap_or(0);
                self.session.rng.skip(4 * particles);
            }
        }
        if self.players[i].dead {
            self.lights.fade_out();
            let p = &mut self.players[i];
            p.dying_timer -= 1;
            if p.dying_timer <= 0 {
                self.completion = Completion::Restart;
            }
        }
    }

    /// Picks `SFXLevelLose`. The original mixes `MilliSecs()` into the pick;
    /// the simulated clock stands in for it.
    fn lose_sound(&mut self) {
        self.session.classic_death_sound += 1;
        let now = self.clock_ms as i32;
        let k2 = now % 100;
        let k3 = now.wrapping_mul(self.session.rng.rand(7, 37)) % 100;
        let sound = if self.session.classic_death_sound < 3 || k3 < 75 {
            LoseSound::Classic
        } else if k2 < 30 {
            LoseSound::Variant('b')
        } else if k2 < 60 {
            LoseSound::Variant('c')
        } else if k2 < 85 {
            LoseSound::Variant('d')
        } else if k2 < 90 {
            LoseSound::Spanish
        } else if k2 < 95 {
            LoseSound::German
        } else {
            LoseSound::French
        };
        self.events.push(Event::Sound(Sound::Lose(sound)));
    }

    /// The camera swoop after switching player or teleporting.
    fn swoop_camera(&mut self) {
        let step = |d: &mut f32| {
            if *d >= CAMERA_STEP {
                *d -= CAMERA_STEP;
                CAMERA_STEP
            } else if *d <= -CAMERA_STEP {
                *d += CAMERA_STEP;
                -CAMERA_STEP
            } else {
                std::mem::take(d)
            }
        };
        let dx = step(&mut self.camera_swoop[0]);
        self.camera.translate(dx, 0.0, 0.0);
        let dz = step(&mut self.camera_swoop[1]);
        self.camera.translate(0.0, 0.0, dz);
    }

    /// Snow, rain, lightning and psyche. Every tick draws for lightning and
    /// psyche, even in levels without them: Blitz's `And` does not
    /// short-circuit.
    fn weather(&mut self) {
        let detail = 5 - self.options.detail_level;
        let rng = &mut self.session.rng;
        let spawn_tick = self.level_timer % (1 + detail * detail) == 0;
        if self.level.weather.snow && spawn_tick {
            // A snowflake: position, fall speed and spin.
            rng.skip(5);
        }
        if self.level.weather.rain && spawn_tick {
            // A raindrop: position.
            rng.skip(3);
        }

        let calm = !self.any_dying() && self.sign_active == 0;
        let roll = self.session.rng.rand(0, 150);
        if self.level.weather.lightning && self.lightning_timer == 0 && calm && roll == 1 {
            self.lightning_timer = 1;
            if self.session.rng.rand(0, 3) == 1 {
                let n = self.session.rng.rand(0, 3);
                self.events.push(Event::Sound(Sound::Lightning(n)));
            }
        }
        if self.lightning_timer > 0 && calm {
            self.lightning();
        }

        let roll = self.session.rng.rand(0, 100);
        if self.level.weather.psyche && calm && roll == 1 {
            let ch = self.level.channels;
            let rng = &mut self.session.rng;
            for (c, on) in [ch.red, ch.green, ch.blue].into_iter().enumerate() {
                if on {
                    self.lights.light_target[c] = rng.rand(0, 255);
                    self.lights.ambient_target[c] = rng.rand(100, 255);
                } else {
                    self.lights.light_target[c] = 100;
                    self.lights.ambient_target[c] = 100;
                }
            }
            self.lights.light_speed = 1;
            self.lights.ambient_speed = 1;
        }
    }

    /// Three flashes over 100 ticks.
    fn lightning(&mut self) {
        let ch = self.level.channels;
        let rng = &mut self.session.rng;
        match self.lightning_timer {
            1 | 30 | 60 => {
                let lit = |on: bool| if on { 255 } else { 100 };
                self.lights.light_target = [lit(ch.red), lit(ch.green), lit(ch.blue)];
                self.lights.ambient_target = self.lights.light_target;
                self.lights.light_speed = rng.rand(20, 40);
                self.lights.ambient_speed = rng.rand(20, 30);
            }
            10 | 35 | 65 => {
                let blue = if ch.blue { 255 } else { 100 };
                self.lights.light_target = [100, 100, blue];
                self.lights.ambient_target = [100, 100, blue];
                self.lights.light_speed = rng.rand(5, 10);
                self.lights.ambient_speed = rng.rand(2, 5);
            }
            _ => {}
        }
        self.lightning_timer += 1;
        if self.lightning_timer == 100 {
            self.lightning_timer = 0;
        }
    }

    /// `MainGameLoop`'s time limit. The original checks it once per frame
    /// against the wall clock; here it runs after every tick against the
    /// simulated clock, so a second lasts 91 ticks (1001 ms).
    fn count_down(&mut self) {
        if self.completion != Completion::Playing || self.options.no_time_limit {
            return;
        }
        if self.clock_ms - self.map_ms >= 1000 {
            self.map_time -= 1;
            if self.map_time == 30 {
                self.events.push(Event::Sound(Sound::Time));
            }
            if (0..=10).contains(&self.map_time) {
                self.events.push(Event::Sound(Sound::Count(self.map_time)));
            }
            self.map_ms = self.clock_ms;
        }
        if self.map_time < 0 {
            self.kill_players();
            self.map_time = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::tests::level;

    fn start(tiles: &[i32], objects: &[i32]) -> World {
        let l = level(5, tiles.len(), tiles, objects);
        World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap()
    }

    #[test]
    fn greets_and_frames_the_first_player() {
        let w = start(&[100, 100, 100], &[0, 2, 1]);
        assert!(w.players[1].focus && !w.players[2].focus);
        assert_eq!(
            w.events(),
            [Event::Sound(Sound::Talk(BlitzRng::default().rand(0, 4)))]
        );
        assert_eq!(w.camera.position, [2.5, 20.4, 0.5]);
        assert_eq!(w.camera.pitch, 90.0);

        let w = start(&[100, 100], &[3, 0]);
        assert_eq!(w.events(), [Event::Sound(Sound::Talk(6))]);
        assert_eq!(w.session.rng, BlitzRng::default());

        let l = level(5, 2, &[100, 100], &[0, 0]);
        let err = World::start(&l, Ruleset::rtwp(), Options::default(), Session::default());
        assert!(err.is_err());
    }

    #[test]
    fn construct_level_rewrites_tiles() {
        // x-floor, fake wall, box factory, warp gate, and a pillar (v5
        // object 50) on floor.
        let w = start(&[100, 104, 1500, 1600, 2200, 100], &[1, 0, 0, 0, 0, 50]);
        let row: Vec<i32> = (0..6).map(|x| w.level.grid.get(x, 0)).collect();
        assert_eq!(row, [100, 200, 100, 200, 200, 200]);
    }

    #[test]
    fn broken_zbot_draws_its_direction_before_the_greeting() {
        let w = start(&[100, 100], &[26, 1]);
        let mut rng = BlitzRng::default();
        let dir = rng.rand(0, 3);
        let talk = rng.rand(0, 4);
        assert!(matches!(
            w.level.spawns[0],
            Spawn::Monster { dir: MonsterDir::Fixed(d), .. } if d == dir
        ));
        assert_eq!(w.events(), [Event::Sound(Sound::Talk(talk))]);
    }

    #[test]
    fn every_tick_draws_for_lightning_and_psyche() {
        let mut w = start(&[100], &[1]);
        let mut rng = w.session.rng.clone();
        for _ in 0..50 {
            w.step(Input::default());
            rng.skip(2);
        }
        assert_eq!(w.session.rng, rng);
    }

    #[test]
    fn snow_draws_on_its_detail_cadence() {
        let mut w = start(&[100, 100], &[1, 84]);
        let mut rng = w.session.rng.clone();
        // Detail 3: a flake every 1 + 2*2 = 5 ticks, from tick 0.
        for _ in 0..10 {
            w.step(Input::default());
        }
        rng.skip(10 * 2 + 2 * 5);
        assert_eq!(w.session.rng, rng);
    }

    #[test]
    fn time_limit_counts_down_in_simulated_seconds() {
        let mut w = start(&[100], &[1]);
        w.map_time = 11;
        let mut sounds = vec![];
        for tick in 1..=183 {
            w.step(Input::default());
            if tick == 91 {
                assert_eq!(w.map_time, 11);
            }
            if tick == 92 {
                assert_eq!(w.map_time, 10);
                sounds.extend_from_slice(w.events());
            }
        }
        assert_eq!(w.map_time, 9);
        assert_eq!(sounds, [Event::Sound(Sound::Count(10))]);

        w.options.no_time_limit = true;
        for _ in 0..200 {
            w.step(Input::default());
        }
        assert_eq!(w.map_time, 9);
    }

    #[test]
    fn camera_fades_in_over_160_ticks() {
        let mut w = start(&[100], &[1]);
        let [x, y, z] = w.camera.position;
        for _ in 0..200 {
            w.step(Input::default());
        }
        let [x2, y2, z2] = w.camera.position;
        assert_eq!(x2, x);
        assert!((y - y2 - 10.0).abs() < 1e-4 && (z - z2 - 10.0).abs() < 1e-4);
        assert!((w.camera.pitch - 55.0).abs() < 1e-4);
        // Both lights take 255 ticks to come up.
        assert_eq!(w.lights.light, [145; 3]);
        for _ in 0..55 {
            w.step(Input::default());
        }
        assert_eq!(w.lights.light, [255, 255, 255]);
        assert_eq!(w.lights.ambient, [255, 255, 255]);
    }

    #[test]
    fn free_camera_moves_and_snaps_back() {
        let mut objects = [0; 30];
        objects[0] = 1;
        let mut w = start(&[100; 30], &objects);
        let start = w.camera.position;
        let held = Input {
            camera: true,
            right: true,
            ..Input::default()
        };
        for _ in 0..10 {
            w.step(held);
        }
        assert!(w.free_camera_active);
        assert!((w.free_camera[0] - 1.0).abs() < 1e-5);
        w.step(Input::default());
        assert!(!w.free_camera_active);
        assert!((w.camera.position[0] - start[0]).abs() < 1e-5);
    }

    #[test]
    fn a_dead_player_restarts_the_level() {
        let mut w = start(&[100], &[1]);
        w.players[1].dying = true;
        w.players[1].dying_timer = 50;
        let mut exploded_at = None;
        let mut tick = 0;
        while w.completion == Completion::Playing {
            w.step(Input::default());
            tick += 1;
            if w.events().contains(&Event::PlayerExploded(1)) {
                exploded_at = Some(tick);
                assert!(
                    w.events()
                        .contains(&Event::Sound(Sound::Lose(LoseSound::Classic)))
                );
            }
            assert!(tick < 1000);
        }
        assert_eq!(exploded_at, Some(50));
        assert_eq!(tick, 50 + 319);
        assert_eq!(w.completion, Completion::Restart);
        assert_eq!(w.session.classic_death_sound, 1);
        assert_eq!(w.lights.light_target, [-255; 3]);
    }

    #[test]
    fn same_inputs_give_the_same_world() {
        let run = || {
            let mut w = start(&[100, 100, 100, 100], &[1, 86, 87, 85]);
            for t in 0..2000u32 {
                w.step(Input::from_bits((t / 7 % 64) as u8));
            }
            w
        };
        assert_eq!(run(), run());
    }
}
