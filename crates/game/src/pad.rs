//! Controllers, which the original does not have. Any controller drives
//! the current character: [`Held`] is what they hold between them, and
//! [`Pad`] also keeps the device last used, which the on-screen prompts
//! follow. The simulation only sees `logic::Input`, as from the keyboard.
//!
//! Prompts name a device's button with an icon: a control character in
//! the text ([`Icon`]) stands for the picture, over its own column and
//! the blank after it. Text written for the keyboard is swapped with
//! [`menu_prompt`] and [`sign_prompt`].
//!
//! `WL_DEVICE=xbox|ps3|ps4|ps5|nintendo` starts on that device, for
//! checking its prompts in a screenshot.

use std::borrow::Cow;

use bevy::input::InputSystems;
use bevy::input::gamepad::{Gamepad, GamepadConnection, GamepadConnectionEvent};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const SONY: u16 = 0x054C;
const NINTENDO: u16 = 0x057E;

/// How far a stick must lean to count, of its full travel.
const DEADZONE: f32 = 0.5;

/// What the player last used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Device {
    #[default]
    Keyboard,
    Xbox,
    Ps3,
    Ps4,
    Ps5,
    Nintendo,
}

impl Device {
    /// By USB ids (SDL's `usb_ids.h`). Other makers' pads show as Xbox.
    pub fn from_ids(vendor: Option<u16>, product: Option<u16>) -> Self {
        match (vendor, product) {
            (Some(SONY), Some(0x0268)) => Self::Ps3,
            (Some(SONY), Some(0x0CE6 | 0x0DF2)) => Self::Ps5,
            (Some(SONY), _) => Self::Ps4,
            (Some(NINTENDO), _) => Self::Nintendo,
            _ => Self::Xbox,
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "xbox" => Some(Self::Xbox),
            "ps3" => Some(Self::Ps3),
            "ps4" => Some(Self::Ps4),
            "ps5" => Some(Self::Ps5),
            "nintendo" => Some(Self::Nintendo),
            _ => None,
        }
    }

    pub fn is_pad(self) -> bool {
        self != Self::Keyboard
    }

    /// Nintendo's A is on the right, where the others have back.
    fn confirm_button(self) -> GamepadButton {
        if self == Self::Nintendo {
            GamepadButton::East
        } else {
            GamepadButton::South
        }
    }

    fn back_button(self) -> GamepadButton {
        if self == Self::Nintendo {
            GamepadButton::South
        } else {
            GamepadButton::East
        }
    }
}

/// A button's picture. The value is the control character in the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Confirm = 1,
    Back = 2,
    Move = 3,
    Switch = 4,
    Look = 5,
    Menu = 6,
}

impl Icon {
    pub const ALL: [Icon; 6] = [
        Icon::Confirm,
        Icon::Back,
        Icon::Move,
        Icon::Switch,
        Icon::Look,
        Icon::Menu,
    ];

    /// The icon a text byte stands for.
    pub fn from_byte(b: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|&i| i as u8 == b)
    }
}

macro_rules! prompts {
    ($($file:literal),* $(,)?) => {
        [$(include_bytes!(concat!("../prompts/", $file, ".png")).as_slice()),*]
    };
}

/// The PNG of `icon` on `device`, from Kenney's Input Prompts (CC0).
pub fn icon_png(device: Device, icon: Icon) -> Option<&'static [u8]> {
    let set = match device {
        Device::Keyboard => return None,
        Device::Xbox => prompts!(
            "xbox_button_color_a",
            "xbox_button_color_b",
            "xbox_dpad",
            "xbox_button_color_y",
            "xbox_stick_r",
            "xbox_button_menu",
        ),
        Device::Ps3 => prompts!(
            "playstation_button_color_cross",
            "playstation_button_color_circle",
            "playstation_dpad",
            "playstation_button_color_triangle",
            "playstation_stick_r",
            "playstation3_button_start",
        ),
        Device::Ps4 => prompts!(
            "playstation_button_color_cross",
            "playstation_button_color_circle",
            "playstation_dpad",
            "playstation_button_color_triangle",
            "playstation_stick_r",
            "playstation4_button_options",
        ),
        Device::Ps5 => prompts!(
            "playstation_button_color_cross",
            "playstation_button_color_circle",
            "playstation_dpad",
            "playstation_button_color_triangle",
            "playstation_stick_r",
            "playstation5_button_options",
        ),
        Device::Nintendo => prompts!(
            "switch_button_a",
            "switch_button_b",
            "switch_dpad",
            "switch_button_x",
            "switch_stick_r",
            "switch_button_plus",
        ),
    };
    Some(set[icon as usize - 1])
}

const CONFIRM: &str = "\x01 ";
const MOVE: &str = "\x03 ";

/// The menus' key names, in all three languages.
const MENU_SWAPS: [(&str, &str); 7] = [
    ("'Enter'", CONFIRM),
    ("'Entree'", CONFIRM),
    ("Enter to Select", "\x01  to Select"),
    ("Space Key", CONFIRM),
    ("Cursor Keys", MOVE),
    ("Pfeiltasten", MOVE),
    ("fleches", MOVE),
];

/// What the shipped levels' signs say about keys. A sign's whole phrase
/// is swapped where the keyboard's way of doing it differs.
const SIGN_SWAPS: [(&str, &str); 17] = [
    (
        "Hold down the 'Shift'-Key while using #the Cursor-Keys to explore a level.",
        "Use \x05  to explore a level.",
    ),
    (
        "press and hold the 'Shift'#key, then use the arrows keys to explore the#level.",
        "use \x05  to explore#the level.",
    ),
    (
        "by#holding down \"Shift\" and using the Arrow Keys.",
        "#with \x05 .",
    ),
    (
        "Press and hold the \"Shift\"-Key, then use#the Cursor Keys to explore this level.",
        "Use \x05  to explore this level.",
    ),
    (
        "Use \"Shift\" and Cursor Keys to explore.",
        "Use \x05  to explore.",
    ),
    (
        "Remember to use the Shift and Cursor Keys#to explore a level.",
        "Remember to use \x05 #to explore a level.",
    ),
    ("the 'Enter'-Key.", "\x04 ."),
    ("Press the \"Enter\" key", "Press \x04 "),
    ("Use the \"Enter\" Key", "Use \x04 "),
    (
        "just press the 'Esc' key to restart it",
        "just press \x06  to restart it",
    ),
    ("press 'Esc'", "press \x06 "),
    ("press Esc", "press \x06 "),
    ("the Keyboard", "the Controller"),
    ("the Arrow Keys", MOVE),
    ("the Cursor Keys", MOVE),
    ("Arrow Keys", MOVE),
    ("Cursor Keys", MOVE),
];

fn swap<'a>(text: &'a str, device: Device, swaps: &[(&str, &str)]) -> Cow<'a, str> {
    if !device.is_pad() {
        return Cow::Borrowed(text);
    }
    let mut out = Cow::Borrowed(text);
    for (from, to) in swaps {
        if out.contains(from) {
            out = Cow::Owned(out.replace(from, to));
        }
    }
    out
}

/// A menu line for `device`.
pub fn menu_prompt(text: &str, device: Device) -> Cow<'_, str> {
    swap(text, device, &MENU_SWAPS)
}

/// A level sign's text for `device`.
pub fn sign_prompt(text: &str, device: Device) -> Cow<'_, str> {
    swap(text, device, &SIGN_SWAPS)
}

/// The directions a stick leans: past the deadzone, on its stronger axis
/// only, so one push is one direction.
fn lean(stick: Vec2) -> Held {
    if stick.x.abs().max(stick.y.abs()) < DEADZONE {
        Held::default()
    } else if stick.y.abs() >= stick.x.abs() {
        if stick.y > 0.0 {
            Held(Held::UP)
        } else {
            Held(Held::DOWN)
        }
    } else if stick.x > 0.0 {
        Held(Held::RIGHT)
    } else {
        Held(Held::LEFT)
    }
}

/// What the controllers hold, as bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Held(u32);

impl Held {
    pub const UP: u32 = 1;
    pub const DOWN: u32 = 1 << 1;
    pub const LEFT: u32 = 1 << 2;
    pub const RIGHT: u32 = 1 << 3;
    /// The right stick; the shift gives its direction on [`Self::look`].
    const LOOK_SHIFT: u32 = 4;
    pub const CONFIRM: u32 = 1 << 8;
    pub const BACK: u32 = 1 << 9;
    pub const SWITCH: u32 = 1 << 10;
    pub const MENU: u32 = 1 << 11;
    pub const DELETE: u32 = 1 << 12;
    pub const PAGE_UP: u32 = 1 << 13;
    pub const PAGE_DOWN: u32 = 1 << 14;
    pub const SORT_NAME: u32 = 1 << 15;
    pub const SORT_FILE: u32 = 1 << 16;

    pub fn has(self, bit: u32) -> bool {
        self.0 & bit != 0
    }

    /// The right stick's direction, as the same bits as the left's.
    pub fn look(self) -> Self {
        Self(self.0 >> Self::LOOK_SHIFT & 0xF)
    }

    fn any(self) -> bool {
        self.0 != 0
    }

    /// What is held here and was not in `before`.
    fn since(self, before: Self) -> Self {
        Self(self.0 & !before.0)
    }

    fn from_sticks(left: Vec2, right: Vec2) -> Self {
        Self(lean(left).0 | lean(right).0 << Self::LOOK_SHIFT)
    }

    fn read(pad: &Gamepad, device: Device) -> Self {
        let mut held = Self::from_sticks(pad.left_stick(), pad.right_stick());
        for (button, bit) in [
            (GamepadButton::DPadUp, Self::UP),
            (GamepadButton::DPadDown, Self::DOWN),
            (GamepadButton::DPadLeft, Self::LEFT),
            (GamepadButton::DPadRight, Self::RIGHT),
            (device.confirm_button(), Self::CONFIRM),
            (device.back_button(), Self::BACK),
            (GamepadButton::North, Self::SWITCH),
            (GamepadButton::West, Self::DELETE),
            (GamepadButton::Start, Self::MENU),
            (GamepadButton::LeftTrigger, Self::PAGE_UP),
            (GamepadButton::RightTrigger, Self::PAGE_DOWN),
            (GamepadButton::LeftTrigger2, Self::SORT_NAME),
            (GamepadButton::RightTrigger2, Self::SORT_FILE),
        ] {
            if pad.pressed(button) {
                held.0 |= bit;
            }
        }
        held
    }
}

/// The controllers this frame.
#[derive(Resource, Debug, Default)]
pub struct Pad {
    pub held: Held,
    /// What went down this frame.
    pub pressed: Held,
    pub device: Device,
    /// The controller last used.
    active: Option<Entity>,
}

impl Pad {
    /// Follows the last-used device: a key press, a press on `used`, or
    /// `active` going away, for another controller or the keyboard.
    fn track(
        &mut self,
        key_pressed: bool,
        used: Option<(Entity, Device)>,
        present: &[(Entity, Device)],
    ) {
        if let Some((entity, device)) = used {
            self.active = Some(entity);
            self.device = device;
        } else if key_pressed {
            self.active = None;
            self.device = Device::Keyboard;
        } else if let Some(active) = self.active
            && !present.iter().any(|(e, _)| *e == active)
        {
            self.active = present.first().map(|p| p.0);
            self.device = present.first().map_or(Device::Keyboard, |p| p.1);
        }
    }
}

pub struct PadPlugin;

impl Plugin for PadPlugin {
    fn build(&self, app: &mut App) {
        let device = std::env::var("WL_DEVICE")
            .ok()
            .and_then(|d| Device::from_name(&d));
        app.insert_resource(Pad {
            device: device.unwrap_or_default(),
            ..Pad::default()
        })
        .add_systems(PreUpdate, read_pads.after(InputSystems));
    }
}

fn read_pads(
    mut pad: ResMut<Pad>,
    keys: Res<ButtonInput<KeyCode>>,
    pads: Query<(Entity, &Gamepad)>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut connections: MessageReader<GamepadConnectionEvent>,
) {
    let mut used = None;
    for event in connections.read() {
        if let GamepadConnection::Connected {
            name,
            vendor_id,
            product_id,
        } = &event.connection
        {
            let device = Device::from_ids(*vendor_id, *product_id);
            info!(
                "controller {name:?}: vendor {vendor_id:04X?}, product {product_id:04X?}: {device:?}"
            );
            used = Some((event.gamepad, device));
        }
    }
    let present: Vec<(Entity, Device)> = pads
        .iter()
        .map(|(e, p)| (e, Device::from_ids(p.vendor_id(), p.product_id())))
        .collect();
    // Like the keyboard, controllers belong to the window in front.
    let focused = window.iter().all(|w| w.focused);
    let mut held = Held::default();
    for ((entity, gamepad), (_, device)) in pads.iter().zip(&present) {
        if !focused {
            break;
        }
        let this = Held::read(gamepad, *device);
        if this.since(pad.held).any() {
            used = Some((entity, *device));
        }
        held.0 |= this.0;
    }
    pad.pressed = held.since(pad.held);
    pad.held = held;
    pad.track(keys.get_just_pressed().next().is_some(), used, &present);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn devices_by_id() {
        let d = |v, p| Device::from_ids(Some(v), Some(p));
        assert_eq!(d(0x045E, 0x02FF), Device::Xbox);
        assert_eq!(d(SONY, 0x0268), Device::Ps3);
        assert_eq!(d(SONY, 0x05C4), Device::Ps4);
        assert_eq!(d(SONY, 0x09CC), Device::Ps4);
        assert_eq!(d(SONY, 0x0BA0), Device::Ps4);
        assert_eq!(d(SONY, 0x0CE6), Device::Ps5);
        assert_eq!(d(SONY, 0x0DF2), Device::Ps5);
        assert_eq!(d(NINTENDO, 0x2009), Device::Nintendo);
        assert_eq!(d(NINTENDO, 0x2069), Device::Nintendo);
        assert_eq!(d(0x1234, 0x5678), Device::Xbox);
        assert_eq!(Device::from_ids(None, None), Device::Xbox);
    }

    #[test]
    fn nintendo_swaps_a_and_b() {
        assert_eq!(Device::Xbox.confirm_button(), GamepadButton::South);
        assert_eq!(Device::Ps5.back_button(), GamepadButton::East);
        assert_eq!(Device::Nintendo.confirm_button(), GamepadButton::East);
        assert_eq!(Device::Nintendo.back_button(), GamepadButton::South);
    }

    #[test]
    fn a_stick_gives_its_stronger_axis_past_the_deadzone() {
        assert_eq!(lean(Vec2::new(0.4, -0.49)), Held::default());
        assert_eq!(lean(Vec2::new(0.6, 0.0)), Held(Held::RIGHT));
        assert_eq!(lean(Vec2::new(-0.8, 0.7)), Held(Held::LEFT));
        assert_eq!(lean(Vec2::new(0.7, 0.8)), Held(Held::UP));
        assert_eq!(lean(Vec2::new(0.2, -0.6)), Held(Held::DOWN));
    }

    #[test]
    fn the_right_stick_is_kept_apart_from_the_left() {
        let held = Held::from_sticks(Vec2::new(1.0, 0.0), Vec2::new(0.0, -1.0));
        assert!(held.has(Held::RIGHT) && !held.has(Held::DOWN));
        assert!(held.look().has(Held::DOWN) && !held.look().has(Held::RIGHT));
    }

    #[test]
    fn the_prompts_follow_the_last_used_device() {
        let mut world = World::new();
        let (xbox, ps) = (world.spawn_empty().id(), world.spawn_empty().id());
        let both = [(xbox, Device::Xbox), (ps, Device::Ps5)];
        let mut pad = Pad::default();
        pad.track(false, None, &both);
        assert_eq!(pad.device, Device::Keyboard);
        pad.track(false, Some(both[1]), &both);
        assert_eq!(pad.device, Device::Ps5);
        pad.track(true, None, &both);
        assert_eq!(pad.device, Device::Keyboard);
        pad.track(false, Some(both[0]), &both);
        // Unplugged: the other one, then the keyboard.
        pad.track(false, None, &both[1..]);
        assert_eq!(pad.device, Device::Ps5);
        pad.track(false, None, &[]);
        assert_eq!(pad.device, Device::Keyboard);
    }

    #[test]
    fn every_icon_decodes_on_every_controller() {
        for device in [
            Device::Xbox,
            Device::Ps3,
            Device::Ps4,
            Device::Ps5,
            Device::Nintendo,
        ] {
            for icon in Icon::ALL {
                let png = icon_png(device, icon).unwrap();
                assert!(image::load_from_memory(png).is_ok(), "{device:?} {icon:?}");
            }
        }
        assert_eq!(icon_png(Device::Keyboard, Icon::Confirm), None);
    }

    #[test]
    fn menu_prompts_lose_their_keys_in_all_three_languages() {
        for (text, want) in [
            ("(Press Space Key To Start)", "(Press \x01  To Start)"),
            (
                "Use Cursor Keys and Enter to Select",
                "Use \x03  and \x01  to Select",
            ),
            ("'Enter' zum fortsetzen.", "\x01  zum fortsetzen."),
            (
                "Tape 'Entree' pour continuer.",
                "Tape \x01  pour continuer.",
            ),
            (
                "Benutze die Pfeiltasten zur Levelauswahl.",
                "Benutze die \x03  zur Levelauswahl.",
            ),
            (
                "Pour choisir, utilise les fleches et 'Entree'",
                "Pour choisir, utilise les \x03  et \x01 ",
            ),
        ] {
            assert_eq!(menu_prompt(text, Device::Xbox), want);
            assert_eq!(menu_prompt(text, Device::Keyboard), text);
        }
    }

    #[test]
    fn signs_name_the_controls() {
        assert_eq!(
            sign_prompt("Press the \"Enter\" key#to switch", Device::Ps4),
            "Press \x04 #to switch"
        );
        assert_eq!(
            sign_prompt("(press 'Esc' if that happens)# ", Device::Xbox),
            "(press \x06  if that happens)# "
        );
        let unchanged = "Abandon all hope, #ye who enter here...# # # ";
        assert_eq!(sign_prompt(unchanged, Device::Xbox), unchanged);
    }
}
