//! The editor's proof: keeps the [`Solution`] of the level on show checked
//! as it changes, and those of a level pack.
//!
//! A check runs on a thread of its own, as a long solution can take a moment.
//! Only one runs at a time, with the level's own copy, and an edit made
//! meanwhile is checked when it ends, so a burst of edits does not pile up.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::thread;

use wonderland_campaign::pack::solution_path;
use wonderland_editor::Editor;
use wonderland_editor::links::warnings;
use wonderland_editor::proof::{Solution, Verdict};
use wonderland_formats::{Game, Level};
use wonderland_logic::{Options, Ruleset};

/// A running check's answers. Behind a lock only for `Sync`.
#[derive(Debug)]
pub struct Answers<T>(Mutex<Receiver<T>>);

impl<T> Answers<T> {
    /// The next answer, and whether more can come.
    pub fn next(&self) -> (Option<T>, bool) {
        let Ok(receiver) = self.0.lock() else {
            return (None, false);
        };
        match receiver.try_recv() {
            Ok(answer) => (Some(answer), true),
            Err(TryRecvError::Empty) => (None, true),
            Err(TryRecvError::Disconnected) => (None, false),
        }
    }
}

/// The way to win the level on show, and whether it still does.
#[derive(Debug, Default)]
pub struct Proof {
    pub solution: Option<Solution>,
    /// What the check of [`Proof::checked`] said.
    pub verdict: Option<Verdict>,
    /// The revision the verdict is for.
    checked: Option<u64>,
    running: Option<(u64, Answers<Verdict>)>,
}

impl Proof {
    pub fn new(solution: Option<Solution>) -> Self {
        Self {
            solution,
            ..Self::default()
        }
    }

    /// A win that was just played on revision `revision`.
    pub fn won(&mut self, solution: Solution, revision: u64) {
        *self = Self {
            verdict: Some(Verdict::Proven(solution.ticks())),
            checked: Some(revision),
            solution: Some(solution),
            running: None,
        };
    }

    /// Whether the verdict is not yet for `revision`.
    pub fn checking(&self, revision: u64) -> bool {
        self.solution.is_some() && self.checked != Some(revision)
    }

    /// Takes in a finished check, and starts one if `editor` has moved on.
    /// `settled` is false in the middle of a gesture.
    pub fn follow(&mut self, editor: &Editor, options: Options, settled: bool) {
        if let Some((revision, answers)) = &self.running {
            let (verdict, more) = answers.next();
            if let Some(verdict) = verdict {
                self.verdict = Some(verdict);
                self.checked = Some(*revision);
                self.running = None;
            } else if !more {
                self.running = None;
            }
        }
        let revision = editor.revision();
        if self.running.is_some() || !settled || !self.checking(revision) {
            return;
        }
        let Some(solution) = self.solution.clone() else {
            return;
        };
        let level = editor.playable();
        let (sender, receiver) = channel();
        thread::spawn(move || {
            let verdict = solution.check(&level, &Ruleset::rtwp(), options);
            // Nobody is waiting if the level has been left.
            let _ = sender.send(verdict);
        });
        self.running = Some((revision, Answers(Mutex::new(receiver))));
    }
}

/// What the pack dialog says of a level's proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackProof {
    Checking,
    None,
    Done(Verdict),
}

/// One level in the pack dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackLine {
    pub file: String,
    pub title: String,
    pub warnings: usize,
    pub proof: PackProof,
}

/// Reads a level for the editor.
pub fn read_level(path: &Path) -> Result<Level, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Level::parse(&bytes, Game::Rtwp).map_err(|e| format!("{}: {e}", path.display()))
}

/// The solution saved beside the level at `path`, if it is there and whole.
pub fn read_solution(path: &Path) -> Option<Solution> {
    Solution::parse(&std::fs::read(solution_path(path)).ok()?)
}

/// A line for each of `files` in `dir`, with its proof still to come.
pub fn pack_lines(dir: &Path, files: Vec<String>) -> Vec<PackLine> {
    let line = |file: String| {
        let level = read_level(&dir.join(&file)).ok();
        PackLine {
            title: level
                .as_ref()
                .map_or_else(|| "(cannot be read)".to_owned(), |l| l.description.clone()),
            warnings: level.map_or(0, |l| warnings(Editor::open(l).level()).len()),
            proof: PackProof::Checking,
            file,
        }
    };
    files.into_iter().map(line).collect()
}

/// Checks the levels of a pack in turn, on one thread. Stops if nobody is
/// listening any more.
pub fn check_pack(
    dir: PathBuf,
    files: Vec<String>,
    options: Options,
) -> Answers<(usize, PackProof)> {
    let (sender, receiver) = channel();
    thread::spawn(move || {
        for (i, file) in files.iter().enumerate() {
            let path = dir.join(file);
            let proof = match (read_solution(&path), read_level(&path)) {
                (Some(solution), Ok(level)) => {
                    let level = Editor::open(level).playable();
                    PackProof::Done(solution.check(&level, &Ruleset::rtwp(), options))
                }
                _ => PackProof::None,
            };
            if sender.send((i, proof)).is_err() {
                return;
            }
        }
    });
    Answers(Mutex::new(receiver))
}
