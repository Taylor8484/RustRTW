//! Proven beatable: the inputs of a won test play, kept and played again.
//!
//! A [`Recording`] holds what was pressed on each tick. [`Solution::new`]
//! plays it through the level once more, and it counts only if it wins, so a
//! solution is known to work. [`Solution::check`] plays it on the level as it
//! now stands and says whether it still wins or where it broke, which it finds
//! from the trail of cells the players took the first time. The trail is
//! relative to the first player's start, as growing the level moves the cells.
//!
//! Both runs have the same [`fair`] options and [`Session::default`], since
//! the options change the random numbers. A run holds one [`World`] and stops
//! after the solution's ticks, so its memory is bounded by the level's.
//!
//! A solution is saved beside its level, in a file of its own, so the level
//! file stays one the original game reads.

use wonderland_formats::Level;
use wonderland_logic::{Completion, Input, Options, Ruleset, Session, World};

const MAGIC: &[u8; 8] = b"WLWIN01\n";
/// The most ticks a solution keeps: an hour at 85 ticks a second.
pub const MAX_TICKS: u32 = 306_000;

/// The options a proof runs under, whatever the player has set.
pub fn fair(options: Options) -> Options {
    Options {
        no_time_limit: false,
        language: 0,
        ..Options {
            load: options.load,
            ..Options::default()
        }
    }
}

/// The keys held on each tick of a test play.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Recording {
    /// The keys and how many ticks in a row they stayed held.
    runs: Vec<(u8, u32)>,
    ticks: u32,
}

impl Recording {
    /// Adds a tick. Past [`MAX_TICKS`] the rest is dropped, and such a play
    /// cannot win when it is played again.
    pub fn push(&mut self, input: Input) {
        if self.ticks == MAX_TICKS {
            return;
        }
        self.ticks += 1;
        let bits = input.to_bits();
        match self.runs.last_mut() {
            Some((last, count)) if *last == bits => *count += 1,
            _ => self.runs.push((bits, 1)),
        }
    }

    fn inputs(&self) -> impl Iterator<Item = Input> + '_ {
        self.runs
            .iter()
            .flat_map(|&(bits, count)| (0..count).map(move |_| Input::from_bits(bits)))
    }
}

/// A player reaching a cell, relative to the first player's start.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mark {
    tick: u32,
    player: u8,
    x: i32,
    y: i32,
}

/// How a run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    Won,
    Lost,
    OutOfInputs,
}

struct Run {
    end: End,
    ticks: u32,
    start: (i32, i32),
    trail: Vec<Mark>,
}

/// Plays `recording` on `level`, until the level is won or lost.
fn run(
    level: &Level,
    ruleset: &Ruleset,
    options: Options,
    recording: &Recording,
) -> Result<Run, String> {
    let mut world = World::start(level, ruleset.clone(), options, Session::default())
        .map_err(|e| e.to_string())?;
    let cells = |world: &World| -> Vec<Option<(i32, i32)>> {
        let players = world.players.iter();
        players.map(|p| p.exists.then_some((p.x, p.y))).collect()
    };
    let mut last = cells(&world);
    let start = last.iter().flatten().next().copied().unwrap_or_default();
    let mut trail = vec![];
    let mut ticks = 0;
    let mut end = End::OutOfInputs;
    for input in recording.inputs() {
        world.step(input);
        ticks += 1;
        let now = cells(&world);
        for (player, (&cell, &before)) in now.iter().zip(&last).enumerate() {
            if let (Some((x, y)), true) = (cell, cell != before) {
                trail.push(Mark {
                    tick: ticks,
                    player: player as u8,
                    x: x - start.0,
                    y: y - start.1,
                });
            }
        }
        last = now;
        if matches!(world.completion, Completion::Won | Completion::Done) {
            end = End::Won;
            break;
        }
        // The level restarts only after the dying is over, which may be after
        // the inputs end.
        if world.completion == Completion::Restart
            || world.players.iter().any(|p| p.dying || p.dead)
        {
            end = End::Lost;
            break;
        }
    }
    Ok(Run {
        end,
        ticks,
        start,
        trail,
    })
}

/// How a solution stopped working.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// The players died or the time ran out.
    Lost,
    /// The inputs ended with the level not won.
    Unfinished,
    /// The level would not start.
    NoStart(String),
}

/// What playing a solution showed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// It wins, after this many ticks.
    Proven(u32),
    Broke {
        failure: Failure,
        /// The tick the players left the solution's path.
        tick: u32,
        /// The cell the solution went to next, if that is on the level.
        cell: Option<(usize, usize)>,
    },
}

/// A way to win a level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solution {
    inputs: Recording,
    trail: Vec<Mark>,
}

impl Solution {
    /// The solution in `recording`, cut at the win. `None` if it does not win
    /// when it is played again.
    pub fn new(
        level: &Level,
        ruleset: &Ruleset,
        options: Options,
        recording: &Recording,
    ) -> Option<Self> {
        let run = run(level, ruleset, options, recording).ok()?;
        if run.end != End::Won {
            return None;
        }
        let mut inputs = Recording::default();
        recording
            .inputs()
            .take(run.ticks as usize)
            .for_each(|i| inputs.push(i));
        Some(Self {
            inputs,
            trail: run.trail,
        })
    }

    /// The ticks it takes to win.
    pub fn ticks(&self) -> u32 {
        self.inputs.ticks
    }

    /// Plays it on `level`.
    pub fn check(&self, level: &Level, ruleset: &Ruleset, options: Options) -> Verdict {
        let run = match run(level, ruleset, options, &self.inputs) {
            Ok(run) => run,
            Err(e) => {
                return Verdict::Broke {
                    failure: Failure::NoStart(e),
                    tick: 0,
                    cell: None,
                };
            }
        };
        let failure = match run.end {
            End::Won => return Verdict::Proven(run.ticks),
            End::Lost => Failure::Lost,
            End::OutOfInputs => Failure::Unfinished,
        };
        let same = self.trail.iter().zip(&run.trail);
        let agreed = same.take_while(|(a, b)| a == b).count();
        let left = self.trail.get(agreed).or(run.trail.get(agreed));
        let tick = left.map_or(run.ticks, |mark| mark.tick);
        let cell = left.and_then(|mark| {
            let x = usize::try_from(mark.x + run.start.0).ok()?;
            let y = usize::try_from(mark.y + run.start.1).ok()?;
            (x < level.width && y < level.height).then_some((x, y))
        });
        Verdict::Broke {
            failure,
            tick,
            cell,
        }
    }

    /// The file saved beside the level.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend((self.inputs.runs.len() as u32).to_le_bytes());
        for &(bits, count) in &self.inputs.runs {
            out.push(bits);
            out.extend(count.to_le_bytes());
        }
        out.extend((self.trail.len() as u32).to_le_bytes());
        for mark in &self.trail {
            out.extend(mark.tick.to_le_bytes());
            out.push(mark.player);
            out.extend(mark.x.to_le_bytes());
            out.extend(mark.y.to_le_bytes());
        }
        out
    }

    /// Reads a saved solution. `None` if it is damaged.
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        let mut r = crate::reader::Reader::new(bytes.strip_prefix(MAGIC)?);
        let mut inputs = Recording::default();
        for _ in 0..r.u32()? {
            let bits = r.u8()?;
            let count = r.u32()?;
            let ticks = inputs
                .ticks
                .checked_add(count)
                .filter(|&t| t <= MAX_TICKS)?;
            if bits >= 64 || count == 0 {
                return None;
            }
            inputs.runs.push((bits, count));
            inputs.ticks = ticks;
        }
        let mut trail = vec![];
        for _ in 0..r.u32()? {
            trail.push(Mark {
                tick: r.u32()?,
                player: r.u8()?,
                x: r.i32()?,
                y: r.i32()?,
            });
        }
        r.done().then_some(Self { inputs, trail })
    }
}

#[cfg(test)]
mod tests {
    use wonderland_formats::catalogue::Layer;

    use super::*;
    use crate::{Editor, Side};

    const COIN: i32 = 5;
    const GATE: i32 = 6;
    const RIGHT: Input = Input {
        right: true,
        up: false,
        down: false,
        left: false,
        camera: false,
        switch: false,
    };
    const WALL: i32 = 200;
    const WATER: i32 = 300;
    const ICE: i32 = 400;

    /// Stinky, then a clear run to the exit at (6, 5).
    fn corridor() -> Editor {
        let mut editor = Editor::new();
        editor.place(2, 5, Layer::Object, 1);
        editor.place(6, 5, Layer::Object, GATE);
        editor.commit();
        editor
    }

    fn hold_right() -> Recording {
        let mut recording = Recording::default();
        (0..2000).for_each(|_| recording.push(RIGHT));
        recording
    }

    fn solved(editor: &Editor) -> Solution {
        let level = editor.playable();
        let options = fair(Options::default());
        Solution::new(&level, &Ruleset::rtwp(), options, &hold_right()).expect("it wins")
    }

    fn verdict(editor: &Editor, solution: &Solution) -> Verdict {
        let options = fair(Options::default());
        solution.check(&editor.playable(), &Ruleset::rtwp(), options)
    }

    #[test]
    fn runs_join_up_and_stop_at_the_limit() {
        let mut recording = Recording::default();
        recording.push(RIGHT);
        recording.push(RIGHT);
        recording.push(Input::default());
        assert_eq!(recording.runs, [(RIGHT.to_bits(), 2), (0, 1)]);
        let all: Vec<Input> = recording.inputs().collect();
        assert_eq!(all, [RIGHT, RIGHT, Input::default()]);

        let mut full = Recording {
            runs: vec![(0, MAX_TICKS)],
            ticks: MAX_TICKS,
        };
        full.push(Input::default());
        assert_eq!(full.inputs().count(), MAX_TICKS as usize);
    }

    #[test]
    fn a_solution_is_cut_at_the_win_and_only_a_win_counts() {
        let editor = corridor();
        let solution = solved(&editor);
        assert!(solution.ticks() < 2000);
        assert_eq!(
            verdict(&editor, &solution),
            Verdict::Proven(solution.ticks())
        );

        let mut stops_short = Recording::default();
        (0..50).for_each(|_| stops_short.push(RIGHT));
        let options = fair(Options::default());
        let level = editor.playable();
        assert_eq!(
            Solution::new(&level, &Ruleset::rtwp(), options, &stops_short),
            None
        );
    }

    #[test]
    fn an_edit_off_the_path_keeps_the_proof() {
        let mut editor = corridor();
        let solution = solved(&editor);
        editor.place(9, 9, Layer::Object, COIN);
        editor.commit();
        assert!(matches!(verdict(&editor, &solution), Verdict::Proven(_)));
    }

    #[test]
    fn growing_the_level_keeps_the_proof() {
        let mut editor = corridor();
        let solution = solved(&editor);
        editor.grow(Side::Left);
        editor.grow(Side::Top);
        editor.commit();
        assert!(matches!(verdict(&editor, &solution), Verdict::Proven(_)));
    }

    #[test]
    fn a_wall_in_the_way_is_reported_at_the_cell_it_stopped() {
        let mut editor = corridor();
        let solution = solved(&editor);
        editor.place(4, 5, Layer::Tile, WALL);
        editor.commit();
        let Verdict::Broke {
            failure,
            tick,
            cell,
        } = verdict(&editor, &solution)
        else {
            panic!("the way is shut");
        };
        assert_eq!(failure, Failure::Unfinished);
        assert_eq!(cell, Some((4, 5)));
        assert!(tick > 0 && tick < solution.ticks());
    }

    #[test]
    fn the_report_follows_a_level_that_grew() {
        let mut editor = corridor();
        let solution = solved(&editor);
        editor.grow(Side::Left);
        editor.place(5, 5, Layer::Tile, WALL);
        editor.commit();
        let Verdict::Broke { cell, .. } = verdict(&editor, &solution) else {
            panic!("the way is shut");
        };
        assert_eq!(cell, Some((5, 5)));
    }

    #[test]
    fn sliding_into_water_loses() {
        let mut editor = corridor();
        let solution = solved(&editor);
        editor.place(3, 5, Layer::Tile, ICE);
        editor.place(4, 5, Layer::Tile, WATER);
        editor.commit();
        let Verdict::Broke { failure, cell, .. } = verdict(&editor, &solution) else {
            panic!("the way is water");
        };
        assert_eq!(failure, Failure::Lost);
        assert!(cell.is_some());
    }

    #[test]
    fn solutions_survive_the_file_and_damage_is_refused() {
        let solution = solved(&corridor());
        let bytes = solution.to_bytes();
        assert_eq!(Solution::parse(&bytes), Some(solution));
        assert_eq!(Solution::parse(&bytes[..bytes.len() - 1]), None);
        assert_eq!(Solution::parse(b"not a solution"), None);
        let mut extra = bytes.clone();
        extra.push(0);
        assert_eq!(Solution::parse(&extra), None);
    }
}
