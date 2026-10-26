//! Laser gates and the final gate: `ToggleLaserGate` and the `LaserGateActive`
//! flags `CreateLaserGate` sets (`RTWP/gameobject.bb`). The last coin opens
//! the final gate (colour 0); buttons open and close the others.
//!
//! Gate tiles are 10xx: `(tile - 1000) % 50 / 2` is the colour (0 is the
//! final gate) and 50 is added while a gate is open. The renderer shows the
//! gates from the tiles and sinks the final gate over `final_gate_anim`.

use crate::World;
use crate::setup::Spawn;
use crate::world::Event;

impl World {
    /// `CreateLaserGate`, as far as the simulation goes. The original's
    /// array has 10 flags; a colour past it (an open gate in the level
    /// file) would write out of bounds there and is ignored here.
    pub(crate) fn create_laser_gate(&mut self, col: i32) {
        if let Some(active) = usize::try_from(col)
            .ok()
            .and_then(|c| self.laser_gate_active.get_mut(c))
        {
            *active = true;
        }
    }

    /// `ToggleLaserGate`: opens or closes every gate of colour `n`.
    /// `always_off` only opens them. Opening the final gate starts its
    /// sinking animation.
    pub(crate) fn toggle_laser_gate(&mut self, n: i32, always_off: bool) {
        let (width, height) = (
            self.level.grid.width() as i32,
            self.level.grid.height() as i32,
        );
        let set_active = |w: &mut World, on: bool| {
            if let Some(a) = usize::try_from(n)
                .ok()
                .and_then(|c| w.laser_gate_active.get_mut(c))
            {
                *a = on;
            }
        };
        for x in 0..width {
            for y in 0..height {
                let tile = self.level.grid.get(x, y);
                if tile / 100 != 10 || (tile - 1000) % 50 / 2 != n {
                    continue;
                }
                if tile % 100 < 50 {
                    self.level.grid.set(x, y, tile + 50);
                    set_active(self, false);
                    let has_gate = self
                        .level
                        .spawns
                        .iter()
                        .any(|s| matches!(*s, Spawn::LaserGate { x: gx, y: gy, .. } if (gx, gy) == (x, y)));
                    if has_gate && tile % 50 < 2 {
                        self.final_gate_anim = 60;
                    }
                } else if !always_off {
                    self.level.grid.set(x, y, tile - 50);
                    set_active(self, true);
                    self.close_gate_on(x, y);
                } else {
                    continue;
                }
                self.events.push(Event::TileChanged { x, y });
            }
        }
    }

    /// A gate closing on `(x, y)` destroys whatever is on, or moving to or
    /// from, that tile. Ghosts (9) pass through.
    fn close_gate_on(&mut self, x: i32, y: i32) {
        let hit = |ax: i32, bx: i32, ay: i32, by: i32| (ax == x || bx == x) && (ay == y || by == y);
        for k in 1..self.players.len() {
            let p = &self.players[k];
            if p.exists && hit(p.x, p.x2, p.y, p.y2) {
                self.kill_player(k);
            }
        }
        for k in 0..self.blocks.len() {
            let b = &self.blocks[k];
            if hit(b.x, b.x2, b.y, b.y2) {
                self.kill_box(k);
            }
        }
        self.kill_fireballs_in_gate(x, y);
        for k in 0..self.monsters.len() {
            let m = &self.monsters[k];
            if hit(m.x, m.x2, m.y, m.y2) && m.kind != 9 {
                self.kill_monster(k);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::setup::tests::level;
    use crate::{Options, Ruleset, Session, World};

    #[test]
    fn toggles_gates_of_one_colour() {
        // Final gate, a closed colour-1 gate and an open one.
        let l = level(5, 4, &[100, 1000, 1002, 1052], &[1, 0, 0, 0]);
        let mut w =
            World::start(&l, Ruleset::rtwp(), Options::default(), Session::default()).unwrap();
        assert!(w.laser_gate_active[0] && w.laser_gate_active[1]);

        w.toggle_laser_gate(1, false);
        assert_eq!(
            (w.level.grid.get(2, 0), w.level.grid.get(3, 0)),
            (1052, 1002)
        );
        assert!(w.laser_gate_active[1]);
        w.toggle_laser_gate(1, true);
        assert_eq!(
            (w.level.grid.get(2, 0), w.level.grid.get(3, 0)),
            (1052, 1052)
        );
        assert!(!w.laser_gate_active[1]);
        assert_eq!(w.final_gate_anim, 0);

        w.toggle_laser_gate(0, true);
        assert_eq!(w.level.grid.get(1, 0), 1050);
        assert!(!w.laser_gate_active[0]);
        assert_eq!(w.final_gate_anim, 60);
    }
}
