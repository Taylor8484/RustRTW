//! Edits every shipped RTWP level: undo and redo must walk back and forth
//! exactly, and what is saved must load and run.

use std::path::Path;

use wonderland_editor::links::warnings;
use wonderland_editor::{Editor, Side};
use wonderland_formats::catalogue::{Group, Layer, PIECES};
use wonderland_formats::{Game, GameAssets, Level};
use wonderland_logic::{Input, Options, Ruleset, Session, World};

/// A small repeatable generator; the edits only need to vary.
struct Dice(u32);

impl Dice {
    fn roll(&mut self, sides: usize) -> usize {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 16) as usize % sides
    }
}

/// Makes one commit's worth of random edits.
fn edit(editor: &mut Editor, dice: &mut Dice) {
    let (width, height) = (editor.level().width, editor.level().height);
    let cell = |dice: &mut Dice| (dice.roll(width), dice.roll(height));
    let side = [Side::Left, Side::Right, Side::Top, Side::Bottom][dice.roll(4)];
    match dice.roll(7) {
        0 | 1 => {
            let piece = &PIECES[dice.roll(PIECES.len())];
            let code = piece.base + dice.roll(piece.count() as usize) as i32;
            for _ in 0..1 + dice.roll(8) {
                let (x, y) = cell(dice);
                editor.place(x, y, piece.layer, code);
            }
        }
        2 => {
            let (x, y) = cell(dice);
            editor.erase(x, y);
        }
        3 => {
            let (x, y) = cell(dice);
            let clip = editor.copy(x, y, (width - x).min(5), (height - y).min(5));
            let (x, y) = cell(dice);
            editor.paste(x, y, &clip);
        }
        4 => {
            editor.grow(side);
        }
        5 => {
            editor.shrink(side);
        }
        _ => {
            editor.fit(dice.roll(5) as isize - 2, dice.roll(5) as isize - 2, 2, 2);
        }
    }
    editor.commit();
}

#[test]
fn every_level_edits_undoes_and_saves() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/RTWP");
    if !dir.is_dir() {
        eprintln!("skipping: {} not found", dir.display());
        return;
    }
    let assets = GameAssets::open(&dir).unwrap();
    let pack = assets.pack().unwrap();
    let mut levels = 0;
    for (n, entry) in pack.index().entries().iter().enumerate() {
        let name = entry.name.to_lowercase();
        if !name.ends_with(".lev") && !name.ends_with(".lv5") {
            continue;
        }
        levels += 1;
        let level = Level::parse(pack.entry_data(entry), Game::Rtwp).unwrap();
        let opened = level.upgraded();
        let mut editor = Editor::open(level);
        assert_eq!(*editor.level(), opened);

        let mut dice = Dice(n as u32);
        let mut history = vec![editor.level().clone()];
        for _ in 0..30 {
            edit(&mut editor, &mut dice);
            if *editor.level() != *history.last().unwrap() {
                history.push(editor.level().clone());
            }
        }
        for state in history.iter().rev().skip(1) {
            assert!(editor.undo(), "{name}");
            assert_eq!(editor.level(), state, "{name}");
        }
        assert!(!editor.undo(), "{name}");
        for state in history.iter().skip(1) {
            assert!(editor.redo(), "{name}");
            assert_eq!(editor.level(), state, "{name}");
        }
        assert!(!editor.redo(), "{name}");

        let bytes = editor.save(1000 + n as i32);
        let saved = Level::parse(&bytes, Game::Rtwp).unwrap();
        assert_eq!(saved, *editor.level(), "{name}");
        let characters = saved.objects.iter().filter(|&&code| {
            PIECES.iter().any(|p| {
                p.group == Group::Character && p.layer == Layer::Object && p.contains(code)
            })
        });
        assert!(characters.count() >= 1, "{name}");

        let session = Session::default();
        let mut world = World::start(&saved, Ruleset::rtwp(), Options::default(), session).unwrap();
        for _ in 0..200 {
            world.step(Input::default());
        }
    }
    assert!(levels > 100, "only {levels} levels");
}

/// The shipped levels all work, so the warnings should rarely speak of them.
/// `cargo test warnings -- --nocapture` lists what they say.
#[test]
fn warnings_on_shipped_levels() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/RTWP");
    if !dir.is_dir() {
        eprintln!("skipping: {} not found", dir.display());
        return;
    }
    let assets = GameAssets::open(&dir).unwrap();
    let pack = assets.pack().unwrap();
    let mut seen = std::collections::BTreeMap::<String, Vec<String>>::new();
    let mut levels = 0;
    for entry in pack.index().entries() {
        let name = entry.name.to_lowercase();
        if !name.ends_with(".lev") && !name.ends_with(".lv5") {
            continue;
        }
        levels += 1;
        let level = Level::parse(pack.entry_data(entry), Game::Rtwp).unwrap();
        let editor = Editor::open(level);
        for w in warnings(editor.level()) {
            seen.entry(w.message).or_default().push(name.clone());
        }
    }
    println!("{levels} levels");
    for (message, names) in &seen {
        println!(
            "{message}: {} {:?}",
            names.len(),
            &names[..names.len().min(5)]
        );
    }
    let total: usize = seen.values().map(Vec::len).sum();
    assert!(total <= 4, "false alarms: {seen:?}");
}
