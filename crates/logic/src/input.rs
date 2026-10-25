//! The player's input for one tick.
//!
//! The original polls `KeyDown` inside `UpdateGame`, so input is the set of
//! keys held during a tick. Escape (back to the menu) is handled by the frame
//! loop outside the simulation. The debug cheats (Alt+X free camera,
//! Ctrl+F5/F6/F9) are not modelled.

/// Keys held during one tick.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Input {
    /// Cursor or keypad up (scancodes 200, 72).
    pub up: bool,
    /// Cursor or keypad down (208, 80).
    pub down: bool,
    /// Cursor or keypad left (203, 75).
    pub left: bool,
    /// Cursor or keypad right (205, 77).
    pub right: bool,
    /// Either Shift (42, 54): free camera instead of walking, top-down only.
    pub camera: bool,
    /// Either Return (28, 156): switch to the next player.
    pub switch: bool,
}

impl Input {
    /// Packs the keys into one byte for recordings: bit 0 up, then down,
    /// left, right, camera and switch.
    pub fn to_bits(self) -> u8 {
        [
            self.up,
            self.down,
            self.left,
            self.right,
            self.camera,
            self.switch,
        ]
        .iter()
        .enumerate()
        .fold(0, |bits, (i, &held)| bits | (u8::from(held) << i))
    }

    pub fn from_bits(bits: u8) -> Self {
        let held = |i: u8| bits & (1 << i) != 0;
        Self {
            up: held(0),
            down: held(1),
            left: held(2),
            right: held(3),
            camera: held(4),
            switch: held(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_round_trip() {
        for bits in 0..64 {
            assert_eq!(Input::from_bits(bits).to_bits(), bits);
        }
        let i = Input {
            left: true,
            switch: true,
            ..Input::default()
        };
        assert_eq!(i.to_bits(), 0b100100);
    }
}
