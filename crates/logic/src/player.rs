//! The players: `CreatePlayer` to `EndTrampoline` in `RTWP/gameobject.bb`,
//! plus the keyboard control, walk, idle and blink animations and player
//! movement from `UpdateGame` in `RTWP/game.bb`.
//!
//! A player stands on tile `(x, y)` and, while moving, heads for
//! `(x2, y2)`. The entity position is simulated too, because flying off a
//! trampoline kills the player when the entity crosses the level edge and a
//! player switch swoops the camera by the entity offsets.

use crate::md2::{self, Anim};
use crate::world::{Completion, Event, Sound};
use crate::{Input, World};

/// Frames in each player's MD2 model.
pub const FRAMES: i32 = 147;

/// Which of a player's four textures the entity shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Skin {
    /// `PlayerTexture`.
    #[default]
    Normal,
    /// `PlayerTextureB`: dying.
    Dying,
    /// `PlayerTextureC`: an idle expression.
    C,
    /// `PlayerTextureD`: eyes shut, for blinking and some idles.
    D,
}

/// One player slot (`Player*` arrays).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Player {
    pub exists: bool,
    /// Current tile. 101 while flying off a trampoline, 0 once dying.
    pub x: i32,
    pub y: i32,
    /// The tile being moved to.
    pub x2: i32,
    pub y2: i32,
    pub moving: bool,
    pub moving_x: i32,
    pub moving_y: i32,
    /// Ticks left of the current move.
    pub moving_timer: i32,
    /// Ticks per tile of the current move.
    pub speed: i32,
    /// Facing in degrees, turning 5 per tick towards `rotation_goal`.
    pub rotation: i32,
    pub rotation_goal: i32,
    /// Spinning upwards after being hit.
    pub dying: bool,
    pub dying_timer: i32,
    /// Exploded; the level restarts when `dying_timer` runs out.
    pub dead: bool,
    /// `FocusOnPlayer`: the player under control.
    pub focus: bool,
    /// Ticks left of shrinking into a rainbow gate.
    pub shrink: i32,
    /// `PlayerAnimCurrent`: 0 still, 1 walk, 2 push, 3 jump or dying,
    /// 5–7 idles.
    pub anim_current: i32,
    /// `PlayerAnimNew`: the animation the next move should play.
    pub anim_new: i32,
    pub anim: Anim,
    pub idle_counter: i32,
    pub blink_counter: i32,
    pub skin: Skin,
    /// `PlayerBeforeBlinkTexture`: blinking only happens while it is
    /// [`Skin::Normal`].
    pub before_blink: Skin,
    /// In the air after a trampoline.
    pub jumping: bool,
    /// Ticks the jump takes.
    pub jumping_speed: i32,
    /// Tiles the next bounce goes; it grows with each bounce in a row.
    pub jumping_distance: i32,
    /// Where the jump lands.
    pub jumping_x: i32,
    pub jumping_y: i32,
    /// `PlayerView3dRotation`: this player's first-person heading, kept
    /// while another player has the focus.
    pub view3d_rotation: i32,
    /// The entity position in Blitz coordinates (x, up, -row).
    pub position: [f32; 3],
    /// `HideEntity`. Hidden entities also stop animating.
    pub hidden: bool,
    /// The round shadow, freed when the player is hit.
    pub shadow: bool,
}

/// The heading `MovePlayer` turns towards.
fn facing(dx: i32, dy: i32) -> i32 {
    match (dx, dy) {
        (0, 1) => 180,
        (0, _) => 0,
        (1, _) => 270,
        _ => 90,
    }
}

impl World {
    /// Whether player `j` is still in play: `BothCompleted` holds a bit for
    /// each player who has reached the exit.
    pub(crate) fn completed(&self, j: usize) -> bool {
        self.both_completed & (1 << (j - 1)) != 0
    }

    /// Whether `tile` is a conveyor that is running.
    pub(crate) fn conveying(&self, tile: i32) -> bool {
        tile / 100 == 5
            && ((tile % 100 < 8 && self.conveyor_active[0])
                || (tile % 100 >= 8 && self.conveyor_active[1]))
    }

    /// Ice or a running conveyor: things on it move at double speed.
    fn slippery(&self, tile: i32) -> bool {
        tile / 100 == 4 || self.conveying(tile)
    }

    fn player_tile(&self, k: usize) -> i32 {
        let p = &self.players[k];
        self.level.grid.get(p.x, p.y)
    }

    fn animate_player(&mut self, k: usize, mode: i32, speed: f32, first: i32, last: i32) {
        self.players[k].anim.start(mode, speed, first, last, FRAMES);
    }

    fn stop_player_anim(&mut self, k: usize) {
        self.animate_player(k, md2::STOP, 1.0, 0, 0);
    }

    /// `PlayMySound` at player `k`.
    fn play_at_player(&mut self, sound: Sound, k: usize) {
        let at = self.players[k].position;
        self.play_at(sound, at);
    }

    /// `CreatePlayer`.
    pub(crate) fn create_player(&mut self, i: usize, x: i32, y: i32) {
        let view3d_rotation = 2;
        self.players[i] = Player {
            exists: true,
            x,
            y,
            x2: x,
            y2: y,
            rotation: 180,
            rotation_goal: 180,
            jumping_distance: 1,
            blink_counter: 370,
            view3d_rotation,
            position: [x as f32 + 0.5, 0.0, -(y as f32 + 0.5)],
            shadow: true,
            ..Player::default()
        };
    }

    /// `CanPlayerMove`: whether player `k` can step by `(dx, dy)`. Pushes a
    /// box out of the way if it can, and sets the walk animation. Walking
    /// into water or lava off ice, or into an oncoming box, kills.
    pub(crate) fn can_player_move(&mut self, k: usize, dx: i32, dy: i32) -> bool {
        if self.players[k].dying {
            return true;
        }
        let (px, py) = (self.players[k].x, self.players[k].y);
        let (x, y) = (px + dx, py + dy);
        if !self.level.grid.in_level(x, y) {
            return false;
        }
        let tile = self.level.grid.get(x, y);
        let class = tile / 100;
        if (class == 3 || class == 9) && self.players[k].moving {
            self.kill_player(k);
            return false;
        }
        if class == 17 && tile > 1702 {
            // A moving transporter.
            return false;
        }
        if matches!(class, 0 | 2 | 3 | 8 | 9) || (class == 10 && tile % 100 < 50) {
            // Walls, water, cannons, lava and closed gates.
            return false;
        }
        for j in 1..self.players.len() {
            let q = &self.players[j];
            if q.exists && !self.completed(j) && ((q.x, q.y) == (x, y) || (q.x2, q.y2) == (x, y)) {
                return false;
            }
        }
        for i in 0..self.blocks.len() {
            let b = &self.blocks[i];
            if (b.x, b.y) != (x, y) && (b.x2, b.y2) != (x, y) {
                continue;
            }
            if b.moving {
                if (b.moving_x, b.moving_y) == (dx, dy) {
                    // Moving away: follow it.
                    self.players[k].anim_new = 1;
                    return true;
                }
                if (b.moving_x, b.moving_y) == (-dx, -dy) {
                    self.kill_player(k);
                }
                return false;
            }
            let (bx, by) = (b.x, b.y);
            if self.conveying(self.level.grid.get(bx, by)) || !self.can_box_move(i, dx, dy) {
                return false;
            }
            // Pushed from ice or a conveyor, the box slides faster.
            let speed = if self.slippery(self.player_tile(k)) {
                2
            } else {
                0
            };
            self.move_box(i, dx, dy, speed);
            self.players[k].anim_new = 2;
            let kind = self.blocks[i].kind;
            if (9..=12).contains(&kind) {
                // A link sphere: the others of its colour move too. The
                // original tests the pushed sphere's x, not each other's.
                for j in 0..self.blocks.len() {
                    let other = &self.blocks[j];
                    if i == j || other.kind != kind || self.blocks[i].x == 101 || other.moving {
                        continue;
                    }
                    let (ox, oy) = (other.x, other.y);
                    if !self.conveying(self.level.grid.get(ox, oy)) && self.can_box_move(j, dx, dy)
                    {
                        self.move_box(j, dx, dy, 0);
                    }
                }
            }
            return true;
        }
        let tile = self.player_tile(k);
        self.players[k].anim_new = if self.slippery(tile) { 0 } else { 1 };
        true
    }

    /// `MovePlayer`: starts a one-tile move.
    pub(crate) fn move_player(&mut self, k: usize, dx: i32, dy: i32) {
        let p = &mut self.players[k];
        p.idle_counter = 0;
        p.skin = Skin::Normal;
        if !p.anim.animating() || p.anim_current != p.anim_new {
            match p.anim_new {
                0 => self.stop_player_anim(k),
                1 => self.animate_player(k, md2::LOOP, 0.25, 2, 18),
                2 => self.animate_player(k, md2::LOOP, 0.25, 19, 35),
                _ => {}
            }
            let p = &mut self.players[k];
            p.anim_current = p.anim_new;
        }
        let tile = self.player_tile(k);
        if tile / 100 == 4 {
            self.play_at_player(Sound::Ice, k);
        }
        let speed = if self.slippery(tile) {
            self.ruleset.base_speed / 2
        } else {
            self.ruleset.base_speed
        };
        let p = &mut self.players[k];
        p.speed = speed;
        p.moving_timer = speed;
        p.x2 = p.x + dx;
        p.y2 = p.y + dy;
        p.moving = true;
        p.moving_x = dx;
        p.moving_y = dy;
        p.rotation_goal = facing(dx, dy);

        // A shadow stinker mirrors its player's moves.
        let k = k as i32;
        for i in 0..self.monsters.len() {
            let m = &self.monsters[i];
            if (m.kind == 12 + k || m.kind == 16 + k)
                && !m.moving
                && self.can_monster_move(i, -dx, -dy)
            {
                self.move_monster(i, -dx, -dy);
            }
        }
    }

    /// `EndMovePlayer`: player `k` has reached `(x2, y2)`. Handles the tile
    /// left behind and the tile arrived on, which may start the next move.
    pub(crate) fn end_move_player(&mut self, k: usize) {
        let (old_x, old_y) = (self.players[k].x, self.players[k].y);
        let old_tile = self.level.grid.get(old_x, old_y);
        let p = &mut self.players[k];
        p.x = p.x2;
        p.y = p.y2;

        if !self.end_move_trampoline {
            self.leave_tile(k, old_x, old_y, old_tile);
        }

        let tile = self.player_tile(k);
        if tile / 100 == 4 {
            self.slide_on_ice(k, tile, old_tile);
            return;
        }
        if self.conveying(tile) {
            if tile != old_tile {
                self.play_at_player(Sound::Convey(k), k);
            }
            let (dx, dy) = conveyor_dir(tile);
            // Anything a conveyor cannot carry on is crushed.
            if self.can_player_move(k, dx, dy) {
                self.move_player(k, dx, dy);
            } else {
                self.kill_player(k);
            }
            return;
        }
        let (x, y) = (self.players[k].x, self.players[k].y);
        if tile / 100 == 11 {
            self.step_on_button(x, y);
        }
        if tile / 100 == 12 {
            // Stop first, so the teleport cannot drown a moving player.
            self.players[k].moving = false;
            let (dx, dy) = (self.players[k].moving_x, self.players[k].moving_y);
            if self.player_step_on_teleporter(k, x, y, dx, dy) {
                return;
            }
        }
        self.start_signs_under(k);

        let (dx, dy) = (self.players[k].moving_x, self.players[k].moving_y);
        if tile / 100 == 17 && self.can_transporter_move(x, y, dx, dy) {
            self.move_player(k, dx, dy);
            self.move_transporter(x, y, dx, dy);
            self.players[k].anim_new = 0;
            self.stop_player_anim(k);
            self.play_at_player(Sound::Transporter, k);
            return;
        }
        if tile / 100 == 18 {
            self.bounce(k);
            return;
        }
        let p = &mut self.players[k];
        p.jumping_distance = 1;
        p.moving = false;
    }

    /// The tile a player stepped off: switchers flip, breakaway tiles crumble,
    /// buttons pop up and the player's sign goes.
    fn leave_tile(&mut self, k: usize, x: i32, y: i32, old_tile: i32) {
        if old_tile / 100 == 5 && (old_tile - 500) % 8 >= 4 {
            self.flip_switcher(x, y);
        }
        if old_tile / 100 == 6 {
            // 600, 603 and 606 break into water, lava and nothing; the
            // others crack one step further.
            let t = match old_tile {
                600 => 300,
                603 => 900,
                606 => 0,
                _ => old_tile - 1,
            };
            self.play_at(Sound::BreakAway, [x as f32, 0.0, -y as f32]);
            self.level.grid.set(x, y, t);
            self.events.push(Event::TileChanged { x, y });
        }
        if old_tile / 100 == 11 {
            self.step_off_button(x, y);
        }
        for i in 0..self.level.signs.len() {
            let sign = &self.level.signs[i];
            if (sign.x, sign.y) == (x, y) && self.sign_active == k as i32 {
                self.stop_sign(crate::sign::FIRST_SIGN + i as i32);
            }
        }
    }

    /// A switcher stepped off turns to face the other way.
    pub(crate) fn flip_switcher(&mut self, x: i32, y: i32) {
        let grid = &mut self.level.grid;
        let mut t = grid.get(x, y) + 2;
        if (t - 100) % 8 <= 4 {
            t -= 4;
        }
        grid.set(x, y, t);
        self.events.push(Event::TileChanged { x, y });
    }

    /// Keeps a player on ice sliding: forward along the ice if possible,
    /// otherwise back.
    fn slide_on_ice(&mut self, k: usize, tile: i32, old_tile: i32) {
        if old_tile / 100 != 4 {
            let i = self.session.rng.rand(0, 4);
            self.play_at_player(Sound::SlideIce(i), k);
        }
        let (mx, my) = (self.players[k].moving_x, self.players[k].moving_y);
        let ((fx, fy), (bx, by)) = ice_turn(tile, mx, my);
        if self.can_player_move(k, fx, fy) {
            self.move_player(k, fx, fy);
        } else if self.can_player_move(k, bx, by) {
            self.move_player(k, bx, by);
        } else {
            self.kill_player(k);
        }
    }

    /// A trampoline: jump one tile further than last time, steered by the
    /// arrow keys from the third bounce on.
    fn bounce(&mut self, k: usize) {
        let base = self.ruleset.base_speed;
        let p = &mut self.players[k];
        p.jumping_distance += 1;
        p.jumping = true;
        p.moving = true;
        p.moving_timer = base * p.jumping_distance;
        p.speed = base;
        p.jumping_speed = p.moving_timer;
        let distance = p.jumping_distance;
        self.play_at_player(Sound::Trampoline, k);
        let n = match distance {
            6 | 9 => 3,
            12 => 4,
            _ => self.session.rng.rand(1, 2),
        };
        self.play_at_player(Sound::Bounce(k, n), k);
        self.animate_player(k, md2::ONE_SHOT, 0.2, 36, 42);
        let input = self.input;
        let p = &mut self.players[k];
        p.anim_new = 3;
        p.anim_current = 3;
        if p.focus && p.jumping_distance > 2 {
            let steer = [
                (input.up, (0, -1)),
                (input.down, (0, 1)),
                (input.left, (-1, 0)),
                (input.right, (1, 0)),
            ];
            for (held, (dx, dy)) in steer {
                if held {
                    (p.moving_x, p.moving_y) = (dx, dy);
                }
            }
        }
        p.rotation_goal = facing(p.moving_x, p.moving_y);
        p.jumping_x = p.x + p.jumping_distance * p.moving_x;
        p.jumping_y = p.y + p.jumping_distance * p.moving_y;
        // In the air the player is on no tile.
        (p.x, p.y, p.x2, p.y2) = (101, 101, 101, 101);
    }

    /// `ControlPlayer`: turns towards the walking direction, and kills a
    /// player who flies off the level.
    pub(crate) fn control_player(&mut self, k: usize) {
        let p = &mut self.players[k];
        if p.rotation != p.rotation_goal {
            let (r, g) = (p.rotation, p.rotation_goal);
            if (r < g && g - r < 180) || (r > g && r - g > 180) {
                p.rotation += 5;
            } else {
                p.rotation -= 5;
            }
            if p.rotation == 360 {
                p.rotation = 0;
            }
            if p.rotation == -5 {
                p.rotation = 355;
            }
        }
        if p.jumping {
            let [x, _, z] = p.position;
            let width = self.level.grid.width() as f32;
            let height = self.level.grid.height() as f32;
            if x - 0.5 < 0.0 || z + 0.5 > 0.0 || x + 0.5 > width || z - 0.5 < -height {
                self.kill_player(k);
                let p = &mut self.players[k];
                p.moving = false;
                p.jumping = false;
            }
        }
    }

    /// `KillPlayer`: player `k` is hit and starts spinning upwards. The
    /// camera follows the dying player.
    pub(crate) fn kill_player(&mut self, k: usize) {
        let p = &self.players[k];
        if !p.exists || p.dying || self.completion != Completion::Playing || self.completed(k) {
            return;
        }
        if !self.players.iter().any(|p| p.dying || p.dead) {
            self.events.push(Event::Sound(Sound::Death));
        }
        self.animate_player(k, md2::ONE_SHOT, 0.2, 36, 42);
        let p = &mut self.players[k];
        p.skin = Skin::Dying;
        p.anim_current = 3;
        p.x = 0;
        p.y = 0;
        p.dying_timer = 100;
        p.dying = true;
        // Each switch moves the focus on by one, so this ends on `k`.
        for _ in 0..self.players.len() {
            if self.players[k].focus {
                break;
            }
            self.switch_player(false);
        }
        self.players[k].shadow = false;
    }

    /// Kills every player, as when time runs out or the player restarts
    /// from the in-game menu.
    pub fn kill_players(&mut self) {
        for k in 1..self.players.len() {
            if self.players[k].exists {
                self.kill_player(k);
            }
        }
    }

    /// `SwitchPlayer`: hands the focus to the next player still in play.
    /// `talk` is the Return key (the new player speaks); otherwise the
    /// focus moved because someone was hit or left (a teleport sound).
    pub(crate) fn switch_player(&mut self, talk: bool) {
        let mut current = 0;
        let mut old = 0;
        let mut any = false;
        for i in 1..self.players.len() {
            let p = &self.players[i];
            if p.focus {
                current = i;
                old = i;
            }
            if !p.focus && p.exists && !self.completed(i) {
                any = true;
            }
        }
        if !any || (self.level_timer < 160 && talk) {
            return;
        }
        let slots = self.players.len() - 1;
        for _ in 0..slots {
            current = current % slots + 1;
            let p = &self.players[current];
            if !p.focus && p.exists && !self.completed(current) {
                break;
            }
        }
        let sound = if talk {
            Sound::Talk(10 + current as i32)
        } else {
            Sound::Teleport
        };
        self.events.push(Event::Sound(sound));
        self.players[current].focus = true;
        self.players[old].focus = false;
        if self.focus_switch_timer < 60 {
            self.focus_switch_timer = 20;
        }
        let to = self.players[current].position;
        self.camera_swoop = if self.level.view3d {
            let from = self.players[old].position;
            [to[0] - from[0], to[2] - from[2]]
        } else {
            let cam = self.camera.position;
            [to[0] - cam[0], (to[2] - 9.0) - cam[2]]
        };
        self.stop_level_signs();
        self.check_signs();
        if self.level.view3d {
            self.players[current].hidden = true;
            self.players[old].hidden = false;
            self.players[old].view3d_rotation = self.view3d_rotation;
            self.view3d_rotation = self.players[current].view3d_rotation;
        }
    }

    /// `EndTrampoline`: a player (`box_landing` false) or box landed at the
    /// end of a jump. Landing on anything solid kills or destroys both.
    pub(crate) fn end_trampoline(&mut self, box_landing: bool, who: usize) {
        let (x, y) = if box_landing {
            (self.blocks[who].x, self.blocks[who].y)
        } else {
            (self.players[who].x, self.players[who].y)
        };
        let tile = self.level.grid.get(x, y);
        let class = tile / 100;
        let at = |ax: i32, ay: i32, bx: i32, by: i32| (x, y) == (ax, ay) || (x, y) == (bx, by);
        let in_play = |w: &World, j: usize| w.players[j].exists && !w.completed(j);

        if !box_landing {
            if matches!(class, 0 | 2 | 3 | 8 | 9 | 10 | 16) || (class == 17 && tile > 1702) {
                self.players[who].position[1] += 0.5;
                self.kill_player(who);
                return;
            }
            for i in 0..self.blocks.len() {
                let b = &self.blocks[i];
                if at(b.x, b.y, b.x2, b.y2) {
                    self.players[who].position[1] += 0.5;
                    self.kill_player(who);
                    self.kill_box(i);
                    return;
                }
            }
            for i in 0..self.monsters.len() {
                let m = &self.monsters[i];
                if at(m.x, m.y, m.x2, m.y2) {
                    self.players[who].position[1] += 0.5;
                    self.kill_player(who);
                    self.kill_monster(i);
                    return;
                }
            }
            for j in 1..self.players.len() {
                let q = &self.players[j];
                if j != who && in_play(self, j) && at(q.x, q.y, q.x2, q.y2) {
                    self.players[who].position[1] += 0.5;
                    self.kill_player(who);
                    self.kill_player(j);
                    return;
                }
            }
            for i in 0..self.items.len() {
                if (self.items[i].x, self.items[i].y) == (x, y) {
                    self.pick_up_item(i, who);
                }
            }
            return;
        }

        if matches!(class, 0 | 2 | 8 | 10 | 16) || (class == 17 && tile > 1702) {
            self.kill_box(who);
            return;
        }
        for i in 0..self.blocks.len() {
            let b = &self.blocks[i];
            if who != i && at(b.x, b.y, b.x2, b.y2) {
                // Sticky cubes (8) survive the crash.
                if b.kind != 8 && self.blocks[who].kind != 8 {
                    self.kill_box(who);
                }
                self.kill_box(i);
                return;
            }
        }
        for i in 0..self.monsters.len() {
            let m = &self.monsters[i];
            if at(m.x, m.y, m.x2, m.y2) {
                if m.kind != 9 {
                    self.kill_monster(i);
                    self.kill_box(who);
                }
                return;
            }
        }
        for j in 1..self.players.len() {
            let q = &self.players[j];
            if in_play(self, j) && at(q.x, q.y, q.x2, q.y2) {
                self.kill_box(who);
                self.kill_player(j);
                return;
            }
        }
        for i in 0..self.items.len() {
            if (self.items[i].x, self.items[i].y) == (x, y) {
                self.kill_box(who);
            }
        }
    }

    /// `NormalKeyboardControl`: each held arrow walks the focused player,
    /// checked up, down, left, right, each seeing the moves before it.
    pub(crate) fn normal_keyboard_control(&mut self, input: Input) {
        let dirs = [
            (input.up, 0, -1),
            (input.down, 0, 1),
            (input.left, -1, 0),
            (input.right, 1, 0),
        ];
        for (held, dx, dy) in dirs {
            if held {
                self.walk(dx, dy);
            }
        }
    }

    /// `View3dKeyboardControl`: up and down walk relative to the view, left
    /// and right turn it a quarter at a time.
    pub(crate) fn view3d_keyboard_control(&mut self, input: Input) {
        let mut walk_dir = -1;
        if input.up {
            walk_dir = self.view3d_rotation % 4;
        }
        if input.down {
            walk_dir = (2 + self.view3d_rotation) % 4;
        }
        for (dir, dx, dy) in [(0, 0, -1), (2, 0, 1), (3, -1, 0), (1, 1, 0)] {
            if walk_dir == dir {
                self.walk(dx, dy);
            }
        }

        if input.left && self.view3d_rotation * 100 == self.view3d_rotation2 {
            self.view3d_rotation -= 1;
            if self.view3d_rotation == -1 {
                self.view3d_rotation = 3;
            }
        }
        if input.right && self.view3d_rotation * 100 == self.view3d_rotation2 {
            self.view3d_rotation += 1;
            if self.view3d_rotation == 4 {
                self.view3d_rotation = 0;
            }
        }

        // Turn the camera 1.8 degrees per tick towards the new heading.
        let goal = self.view3d_rotation * 100;
        let r2 = self.view3d_rotation2;
        if goal != r2 {
            if (r2 < goal && goal - r2 < 200) || (r2 > goal && r2 - goal > 200) {
                self.view3d_rotation2 += 2;
                self.camera.yaw -= 1.8;
            } else {
                self.view3d_rotation2 -= 2;
                self.camera.yaw += 1.8;
            }
            if self.view3d_rotation2 == 400 {
                self.view3d_rotation2 = 0;
            }
            if self.view3d_rotation2 == -2 {
                self.view3d_rotation2 = 398;
            }
        }
    }

    /// One arrow key in the keyboard control: the focused player walks, or
    /// rides a transporter, or pushes a box standing on the next
    /// transporter. A player on a running conveyor cannot walk.
    fn walk(&mut self, dx: i32, dy: i32) {
        for i in 1..self.players.len() {
            if !self.players[i].focus {
                continue;
            }
            let tile = self.player_tile(i);
            if self.players[i].moving || self.conveying(tile) {
                continue;
            }
            if self.can_player_move(i, dx, dy) {
                self.move_player(i, dx, dy);
                continue;
            }
            // Read again: being hit on the way moves the player to (0, 0).
            let (x, y) = (self.players[i].x, self.players[i].y);
            let tile = self.player_tile(i);
            let next_in_level = self.level.grid.in_level(x + dx, y + dy);
            if tile / 100 == 17 && tile < 1703 {
                if self.can_transporter_move(x, y, dx, dy) {
                    self.move_player(i, dx, dy);
                    self.move_transporter(x, y, dx, dy);
                    self.players[i].anim_new = 0;
                } else if next_in_level {
                    self.push_box_on_transporter(x + dx, y + dy, dx, dy);
                }
            } else if next_in_level {
                self.push_box_on_transporter(x + dx, y + dy, dx, dy);
            }
        }
    }

    /// A box standing on the transporter at `(x, y)` rides it away.
    fn push_box_on_transporter(&mut self, x: i32, y: i32, dx: i32, dy: i32) {
        let tile = self.level.grid.get(x, y);
        if tile / 100 != 17 || tile >= 1703 {
            return;
        }
        for m in 0..self.blocks.len() {
            let b = &self.blocks[m];
            if !b.moving && (b.x, b.y) == (x, y) && self.can_transporter_move(x, y, dx, dy) {
                self.move_box(m, dx, dy, 0);
                self.move_transporter(x, y, dx, dy);
            }
        }
    }

    /// The start of `UpdateGame`'s per-player loop: the walk animation
    /// stops when the move ends, eyes blink, and a player left alone long
    /// enough plays an idle animation. Blinks and idles draw random numbers.
    pub(crate) fn idle_and_blink(&mut self, i: usize) {
        let p = &self.players[i];
        if p.exists && !p.moving && p.anim_current < 3 {
            self.stop_player_anim(i);
            let p = &mut self.players[i];
            p.anim_new = 0;
            p.anim_current = 0;
        }

        let p = &self.players[i];
        if p.before_blink == Skin::Normal && p.exists {
            let p = &mut self.players[i];
            p.blink_counter -= 1;
            let c = p.blink_counter;
            if c > 20 && c < 30 {
                p.skin = Skin::D;
                p.idle_counter -= 1;
            }
            if (10..20).contains(&c) {
                p.skin = Skin::Normal;
                p.idle_counter -= 1;
            }
            if c > 0 && c < 10 {
                p.skin = Skin::D;
                p.idle_counter -= 1;
            }
            if c <= 0 {
                p.skin = Skin::Normal;
                p.blink_counter = 500 + self.session.rng.rand(0, 500);
            }
        }

        // Counted for every slot, but only an existing player idles.
        self.players[i].idle_counter += 1;
        if self.players[i].idle_counter != 900 || !self.players[i].exists {
            return;
        }
        let pick = self.session.rng.rand(0, 100);
        let face = self.session.rng.rand(0, 100);
        // (animation, mode, speed, frames, idle counter, skin thresholds)
        let (anim, mode, speed, first, last, counter, normal_below, c_below) = if pick < 50 {
            (0, md2::STOP, 1.0, 0, 0, 600, 60, 95)
        } else if pick < 60 {
            (7, md2::LOOP, 0.1, 105, 147, -300, 50, 95)
        } else if pick < 65 {
            (6, md2::LOOP, 0.1, 62, 104, -300, 50, 101)
        } else {
            (5, md2::LOOP, 0.2, 53, 61, -300, 45, 95)
        };
        self.animate_player(i, mode, speed, first, last);
        let p = &mut self.players[i];
        p.anim_current = anim;
        // The last idle sets the current animation twice and never the new
        // one.
        if anim != 5 {
            p.anim_new = anim;
        }
        p.idle_counter = counter;
        let skin = if face < normal_below {
            Skin::Normal
        } else if face < c_below {
            Skin::C
        } else {
            Skin::D
        };
        p.skin = skin;
        p.before_blink = skin;
    }

    /// Players standing on a running conveyor move on once the way is clear.
    pub(crate) fn convey_players(&mut self) {
        for i in 1..self.players.len() {
            let p = &self.players[i];
            if !p.exists || p.moving {
                continue;
            }
            let tile = self.player_tile(i);
            if !self.conveying(tile) {
                continue;
            }
            // Each check sees the tile after the one before it moved.
            for (dir, dx, dy) in [(0, 0, -1), (2, 0, 1), (1, 1, 0), (3, -1, 0)] {
                if self.player_tile(i) % 4 == dir && self.can_player_move(i, dx, dy) {
                    self.move_player(i, dx, dy);
                }
            }
        }
    }

    /// Object movement for the players: one tick along the current move,
    /// with the camera following the focused player.
    pub(crate) fn move_players(&mut self) {
        for i in 1..self.players.len() {
            let p = &mut self.players[i];
            if !p.exists || !p.moving || p.dead || p.dying {
                continue;
            }
            let dx = p.moving_x as f32 / p.speed as f32;
            let dz = -p.moving_y as f32 / p.speed as f32;
            p.position[0] += dx;
            p.position[2] += dz;
            let focus = p.focus;
            if p.jumping {
                let angle = 180.0 / p.jumping_speed as f32 * p.moving_timer as f32;
                p.position[1] += -0.2 * cos_deg(angle);
            }
            p.moving_timer -= 1;
            if focus {
                self.camera.position[0] += dx;
                self.camera.position[2] += dz;
            }
        }
    }

    /// The end of the players' moves, landing any jumps first.
    pub(crate) fn end_player_moves(&mut self) {
        for i in 1..self.players.len() {
            let p = &self.players[i];
            if !(p.moving && p.moving_timer == 0 && p.exists) {
                continue;
            }
            let jumping = p.jumping;
            self.end_move_trampoline = false;
            if jumping {
                let p = &mut self.players[i];
                p.jumping = false;
                p.x2 = p.jumping_x;
                p.y2 = p.jumping_y;
                p.x = p.x2;
                p.y = p.y2;
                p.anim_new = 3;
                p.position[1] -= p.position[1];
                self.stop_player_anim(i);
                self.end_move_trampoline = true;
                self.end_trampoline(false, i);
            }
            self.end_move_player(i);
        }
    }

    /// `UpdateWorld` for the player models.
    pub(crate) fn animate_players(&mut self) {
        for p in &mut self.players[1..] {
            if p.exists && !p.hidden {
                p.anim.animate();
            }
        }
    }
}

/// The forward and backward directions on ice tile `tile` when arriving
/// along `(mx, my)`. Curved ice (401–404) turns; any other ice tile leaves
/// both at (0, 0), as the original's unset locals.
pub(crate) fn ice_turn(tile: i32, mx: i32, my: i32) -> ((i32, i32), (i32, i32)) {
    match tile {
        400 => ((mx, my), (-mx, -my)),
        401 if mx == -1 => ((0, 1), (1, 0)),
        401 => ((1, 0), (0, 1)),
        402 if mx == 1 => ((0, 1), (-1, 0)),
        402 => ((-1, 0), (0, 1)),
        403 if mx == 1 => ((0, -1), (-1, 0)),
        403 => ((-1, 0), (0, -1)),
        404 if mx == -1 => ((0, -1), (1, 0)),
        404 => ((1, 0), (0, -1)),
        _ => ((0, 0), (0, 0)),
    }
}

/// Where a conveyor tile carries things.
pub(crate) fn conveyor_dir(tile: i32) -> (i32, i32) {
    match tile % 4 {
        0 => (0, -1),
        1 => (1, 0),
        2 => (0, 1),
        _ => (-1, 0),
    }
}

/// Blitz's `Cos`, in degrees.
pub(crate) fn cos_deg(deg: f32) -> f32 {
    (f64::from(deg).to_radians()).cos() as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::tests::level;
    use crate::world::{Options, Session};
    use crate::{BlitzRng, Ruleset};

    fn world(width: usize, tiles: &[i32], objects: &[i32]) -> World {
        let l = level(5, width, tiles, objects);
        World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap()
    }

    fn row(tiles: &[i32]) -> World {
        let mut objects = vec![0; tiles.len()];
        objects[0] = 1;
        world(tiles.len(), tiles, &objects)
    }

    const RIGHT: Input = Input {
        right: true,
        up: false,
        down: false,
        left: false,
        camera: false,
        switch: false,
    };

    fn run(w: &mut World, ticks: usize, input: Input) -> Vec<Event> {
        let mut events = vec![];
        for _ in 0..ticks {
            w.step(input);
            events.extend_from_slice(w.events());
        }
        events
    }

    /// Waits out the fade-in: until the camera has come down, positional
    /// sounds are too far away to hear.
    fn settle(w: &mut World) {
        run(w, 160, Input::default());
    }

    fn sounds(events: &[Event]) -> Vec<Sound> {
        events
            .iter()
            .filter_map(|e| match *e {
                Event::Sound(s) | Event::SoundAt(s, _) => Some(s),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn walks_one_tile_in_base_speed_ticks() {
        let mut w = row(&[100, 100, 100]);
        let camera = w.camera.position[0];
        run(&mut w, 1, RIGHT);
        let p = &w.players[1];
        assert!(p.moving && p.x == 0 && p.x2 == 1 && p.rotation_goal == 270);
        assert_eq!(p.anim_current, 1);
        run(&mut w, 23, Input::default());
        let p = &w.players[1];
        assert!(!p.moving && p.x == 1);
        assert!((p.position[0] - 1.5).abs() < 1e-5);
        assert!((w.camera.position[0] - camera - 1.0).abs() < 1e-5);
        // Turning 5 degrees a tick from 180 to 270.
        assert_eq!(p.rotation, 270);
        // The walk stops once the move is over.
        run(&mut w, 1, Input::default());
        assert_eq!(w.players[1].anim_current, 0);
    }

    #[test]
    fn walls_and_other_players_block() {
        let mut w = row(&[100, 0]);
        run(&mut w, 30, RIGHT);
        assert_eq!(w.players[1].x, 0);
        let mut w = world(2, &[100, 100], &[1, 2]);
        run(&mut w, 30, RIGHT);
        assert_eq!(w.players[1].x, 0);
    }

    #[test]
    fn ice_slides_at_double_speed_and_draws_once() {
        let mut w = row(&[100, 400, 400, 100]);
        settle(&mut w);
        let mut rng = w.session.rng.clone();
        let events = run(&mut w, 1, RIGHT);
        assert_eq!(sounds(&events), []);
        let events = run(&mut w, 23 + 12 + 12, Input::default());
        let p = &w.players[1];
        assert!(!p.moving && p.x == 3);
        // Two draws a tick for the weather, and the slide sound's as the
        // first move ends on tick 24, before that tick's weather.
        rng.skip(2 * 23);
        let slide = rng.rand(0, 4);
        rng.skip(2 * 25);
        assert_eq!(w.session.rng, rng);
        assert_eq!(
            sounds(&events),
            [Sound::SlideIce(slide), Sound::Ice, Sound::Ice]
        );
    }

    #[test]
    fn sliding_into_water_kills() {
        let mut w = row(&[100, 400, 300]);
        let events = run(&mut w, 24, RIGHT);
        let p = &w.players[1];
        assert!(p.dying && p.moving);
        assert_eq!((p.x, p.y, p.x2), (0, 0, -1));
        assert!(sounds(&events).contains(&Sound::Death));
        // As in the original, the dying player is then sent back the way
        // it came, which swaps the dying look for the walk.
        assert_eq!((p.skin, p.anim_current), (Skin::Normal, 1));
    }

    #[test]
    fn conveyors_carry_and_crush() {
        let mut w = row(&[501, 100]);
        run(&mut w, 12, Input::default());
        assert_eq!(w.players[1].x, 1);

        let mut w = row(&[501, 100]);
        w.conveyor_active = [false, true];
        run(&mut w, 12, Input::default());
        assert_eq!(w.players[1].x, 0);

        let mut w = row(&[100, 501, 0]);
        settle(&mut w);
        let events = run(&mut w, 24, RIGHT);
        assert!(w.players[1].dying);
        assert!(sounds(&events).contains(&Sound::Convey(1)));
    }

    #[test]
    fn trampolines_throw_one_tile_further_each_bounce() {
        let mut w = row(&[100, 1800, 100, 100, 100]);
        run(&mut w, 24, RIGHT);
        let p = &w.players[1];
        assert!(p.jumping && p.jumping_distance == 2 && p.jumping_x == 3);
        assert_eq!((p.x, p.y), (101, 101));
        run(&mut w, 47, Input::default());
        assert!(w.players[1].jumping);
        run(&mut w, 1, Input::default());
        let p = &w.players[1];
        assert!(!p.jumping && !p.moving && !p.dying && p.x == 3);
        assert_eq!(p.position[1], 0.0);
        assert_eq!(p.jumping_distance, 1);

        // Landing on a wall, or flying off the level, kills.
        let mut w = row(&[100, 1800, 100, 0]);
        run(&mut w, 24 + 48, RIGHT);
        assert!(w.players[1].dying);
        let mut w = row(&[100, 1800, 100]);
        run(&mut w, 24 + 48, RIGHT);
        assert!(w.players[1].dying);
    }

    #[test]
    fn return_switches_player_after_the_fade_in() {
        let mut w = world(3, &[100, 100, 100], &[1, 0, 2]);
        let switch = Input {
            switch: true,
            ..Input::default()
        };
        run(&mut w, 1, switch);
        assert!(w.players[1].focus);
        run(&mut w, 159, Input::default());
        let events = run(&mut w, 1, switch);
        assert!(w.players[2].focus && !w.players[1].focus);
        assert_eq!(sounds(&events), [Sound::Talk(12)]);
        assert_eq!(w.focus_switch_timer, 19);
        // The camera swoops two tiles right, half a unit per tick.
        assert!((w.camera_swoop[0] - 1.5).abs() < 1e-5);
    }

    #[test]
    fn the_camera_follows_a_hit_player() {
        let mut w = world(2, &[100, 100], &[1, 2]);
        w.kill_player(2);
        assert!(w.players[2].focus && !w.players[1].focus);
        assert_eq!(
            sounds(w.events()),
            [
                Sound::Talk(BlitzRng::default().rand(0, 4)),
                Sound::Death,
                Sound::Teleport
            ]
        );
        // Nobody can be hit twice, and a level being won protects everyone.
        w.kill_player(2);
        w.completion = Completion::Won;
        w.kill_player(1);
        assert!(!w.players[1].dying);
    }

    #[test]
    fn blinks_and_idles_draw_random_numbers() {
        let mut w = row(&[100]);
        let mut rng = w.session.rng.clone();
        w.players[1].blink_counter = 1;
        run(&mut w, 1, Input::default());
        // The blink draws before the weather's two.
        let blink = 500 + rng.rand(0, 500);
        rng.skip(2);
        assert_eq!(w.players[1].blink_counter, blink);
        assert_eq!(w.session.rng, rng);

        w.players[1].idle_counter = 899;
        run(&mut w, 1, Input::default());
        let pick = rng.rand(0, 100);
        let face = rng.rand(0, 100);
        rng.skip(2);
        assert_eq!(w.session.rng, rng);
        let p = &w.players[1];
        assert_eq!(p.idle_counter, if pick < 50 { 600 } else { -300 });
        assert_eq!(p.skin, p.before_blink);
        if face >= 95 {
            assert_eq!(p.skin, Skin::D);
        }
    }

    #[test]
    fn signs_dim_the_lights_while_stood_on() {
        let mut w = row(&[100, 1301, 100]);
        assert_eq!(w.signs_shown()[0].tag, crate::sign::LEVEL_NAME);
        run(&mut w, 24, RIGHT);
        assert_eq!(w.sign_active, 1);
        assert_eq!(w.signs_shown()[1].text, "sign 1");
        assert_eq!(w.lights.light_target, [0; 3]);
        run(&mut w, 24, RIGHT);
        assert_eq!(w.sign_active, 0);
        assert_eq!(w.signs_shown().len(), 1);
        assert_eq!(w.lights.light_target, [255; 3]);
        // The level name goes on the 801st tick.
        run(&mut w, 800 - 48, Input::default());
        assert_eq!(w.signs_shown().len(), 1);
        run(&mut w, 1, Input::default());
        assert!(w.signs_shown().is_empty());

        // A sign under the first player stays lit dimly from the start.
        let mut objects = [0; 2];
        objects[0] = 1;
        let w = world(2, &[1300, 100], &objects);
        assert_eq!(w.sign_active, 1);
        assert_eq!(w.lights.light_target, [0; 3]);
        assert_eq!((w.lights.light_speed, w.lights.ambient_speed), (2, 1));
    }

    #[test]
    fn breakaway_tiles_crumble_behind_the_player() {
        let mut w = row(&[602, 100]);
        settle(&mut w);
        let events = run(&mut w, 24, RIGHT);
        assert_eq!(w.level.grid.get(0, 0), 601);
        assert!(events.contains(&Event::TileChanged { x: 0, y: 0 }));
        assert!(sounds(&events).contains(&Sound::BreakAway));
        let mut w = row(&[600, 100]);
        run(&mut w, 24, RIGHT);
        assert_eq!(w.level.grid.get(0, 0), 300);
    }

    #[test]
    fn switchers_carry_and_then_flip() {
        // A switcher is a conveyor; this one points up.
        let w = &mut world(1, &[100, 504], &[0, 1]);
        run(w, 12, Input::default());
        assert_eq!(w.players[1].y, 0);
        assert_eq!(w.level.grid.get(0, 1), 506);
    }

    #[test]
    fn time_up_kills_everyone() {
        let mut w = world(2, &[100, 100], &[1, 2]);
        w.map_time = 0;
        run(&mut w, 92, Input::default());
        assert!(w.players[1].dying && w.players[2].dying);
    }

    #[test]
    fn first_person_walks_and_turns_relative_to_the_view() {
        let w = &mut world(1, &[2300, 100, 100], &[1, 0, 0]);
        assert_eq!((w.view3d_rotation, w.view3d_rotation2), (2, 200));
        // Heading 2 faces down the level.
        let up = Input {
            up: true,
            ..Input::default()
        };
        run(w, 24, up);
        assert_eq!(w.players[1].y, 1);
        let right = Input {
            right: true,
            ..Input::default()
        };
        let yaw = w.camera.yaw;
        run(w, 1, right);
        assert_eq!((w.view3d_rotation, w.view3d_rotation2), (3, 202));
        run(w, 49, right);
        assert_eq!(w.view3d_rotation2, 300);
        assert!((w.camera.yaw - (yaw - 90.0)).abs() < 1e-3);
        run(w, 110, Input::default());
        assert!(w.players[1].hidden);
    }

    #[test]
    fn positional_sounds_fade_with_distance() {
        let mut w = row(&[100; 30]);
        let camera = w.camera.position;
        w.play_at(Sound::Ice, [camera[0], camera[1] - 10.0, camera[2] + 9.0]);
        w.play_at(
            Sound::Ice,
            [camera[0] + 6.0, camera[1] - 10.0, camera[2] + 9.0],
        );
        w.play_at(
            Sound::Ice,
            [camera[0] + 9.0, camera[1] - 10.0, camera[2] + 9.0],
        );
        let volumes: Vec<f32> = w.events()[1..]
            .iter()
            .map(|e| match e {
                Event::SoundAt(_, v) => *v,
                _ => panic!("{e:?}"),
            })
            .collect();
        assert_eq!(volumes.len(), 2);
        assert_eq!(volumes[0], 1.0);
        assert!((volumes[1] - 0.5).abs() < 1e-5);
    }
}
