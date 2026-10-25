//! The tile map (`MapTile`).
//!
//! The original keeps one `Dim MapTile(n, n)` for the whole session and each
//! load overwrites only the level's own `width * height` corner, so a read
//! past the level's edge sees a previous, larger level's tiles. Here every
//! level starts from a zeroed array of the same size; the code mostly
//! bounds-checks against the level size before looking at a neighbour.

/// A tile grid sized like the game's `MapTile` array.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    size: usize,
    width: usize,
    height: usize,
    tiles: Vec<i32>,
}

impl Grid {
    /// A zeroed `size * size` array holding a `width * height` level.
    pub fn new(size: usize, width: usize, height: usize) -> Self {
        assert!(width <= size && height <= size, "level larger than grid");
        Self {
            size,
            width,
            height,
            tiles: vec![0; size * size],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Whether `(x, y)` is inside the level (not just the array).
    pub fn in_level(&self, x: i32, y: i32) -> bool {
        (0..self.width as i32).contains(&x) && (0..self.height as i32).contains(&y)
    }

    fn index(&self, x: i32, y: i32) -> usize {
        let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) else {
            panic!("MapTile({x}, {y}) is out of bounds");
        };
        assert!(
            x < self.size && y < self.size,
            "MapTile({x}, {y}) is out of bounds"
        );
        y * self.size + x
    }

    /// `MapTile(x, y)`. Panics outside the array. Blitz only checks bounds in
    /// debug builds, so such a read is an original bug to look into.
    pub fn get(&self, x: i32, y: i32) -> i32 {
        self.tiles[self.index(x, y)]
    }

    pub fn set(&mut self, x: i32, y: i32, tile: i32) {
        let i = self.index(x, y);
        self.tiles[i] = tile;
    }

    /// Every `(x, y)` in the level, row by row, as the loaders scan it.
    pub fn cells(&self) -> impl Iterator<Item = (i32, i32)> + use<> {
        let (w, h) = (self.width as i32, self.height as i32);
        (0..h).flat_map(move |y| (0..w).map(move |x| (x, y)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_past_the_level_edge_but_not_the_array() {
        let mut g = Grid::new(4, 2, 3);
        g.set(1, 2, 300);
        assert_eq!(g.get(1, 2), 300);
        assert_eq!(g.get(3, 3), 0);
        assert!(!g.in_level(2, 0));
        assert_eq!(g.cells().collect::<Vec<_>>()[..3], [(0, 0), (1, 0), (0, 1)]);
        assert!(std::panic::catch_unwind(|| g.get(4, 0)).is_err());
        assert!(std::panic::catch_unwind(|| g.get(-1, 0)).is_err());
    }
}
