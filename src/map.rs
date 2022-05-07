/// Struct representing the map, containing snake and food locations.
pub struct Map<const W: usize, const H: usize> {
    data: [[Tile; H]; W]
}

impl <const W: usize, const H: usize> Map<W, H> {
    /// Creates a map filled with [Tile::Empty].
    pub fn new() -> Self {
        Map {
            data: [[Tile::Empty; H]; W]
        }
    }

    /// Returns the [Tile] at location `(x,y)`.
    pub fn get(&self, x: usize, y: usize) -> Tile {
        assert!(self.in_bounds(x, y));

        self.data[x][y]
    }

    /// Sets the [Tile] at location `(x,y)`.
    pub fn set(&mut self, x: usize, y: usize, tile: Tile) {
        assert!(self.in_bounds(x, y));

        self.data[x][y] = tile;
    }

    /// Returns if the requested location `(x,y)` is contained.
    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < W && y < H
    }
}

impl <const W: usize, const H: usize> Default for Map<W, H> {
    fn default() -> Self {
        Map::new()
    }
}

/// The Tiles contained in the [Map].
#[derive(PartialEq)]
#[derive(Copy, Clone)]
pub enum Tile {
    Empty,
    Snake,
    Food,
}