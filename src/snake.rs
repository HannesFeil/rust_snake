use crate::map;

/// The four directions the [Snake] can face and `None` in case of a new snake.
#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
    None
}

impl Direction {
    /// The change on the x-axis.
    ///
    /// # Examples
    /// ```
    /// use rust_snake::snake::Direction;
    /// assert_eq!(Direction::Left.x(), -1);
    /// assert_eq!(Direction::Up.x(), 0);
    /// ```
    pub fn x(&self) -> isize {
        match self {
            Direction::Left => -1,
            Direction::Right => 1,
            _ => 0
        }
    }

    /// The change on the y-axis.
    ///
    /// # Examples
    /// ```
    /// use rust_snake::snake::Direction;
    /// assert_eq!(Direction::Up.y(), -1);
    /// assert_eq!(Direction::Left.y(), 0);
    /// ```
    pub fn y(&self) -> isize {
        match self {
            Direction::Up => -1,
            Direction::Down => 1,
            _ => 0
        }
    }

    /// Returns if this and the given `Direction` oppose one another.
    ///
    /// # Examples
    /// ```
    /// use rust_snake::snake::Direction;
    ///
    /// assert!(Direction::Left.opposite(Direction::Right))
    /// ```
    pub fn opposite(&self, dir: Direction) -> bool {
        self.x() + dir.x() == 0 && self.y() + dir.y() == 0
    }
}

/// Struct representing the snake.
pub struct Snake {
    head: (isize, isize),
    dir: Direction,
    pub size: usize,
    tail: Vec<(usize, usize)>,
}

impl Snake {
    /// Creates a new snake at location `(x,y)` with the given size and facing [None](Direction).
    pub fn new(x: usize, y: usize, size: usize) -> Snake {
        Snake {
            head: (x as isize, y as isize),
            dir: Direction::None,
            size,
            tail: Vec::new(),
        }
    }

    /// The x coordinate of the `Snake`'s head.
    pub fn x(&self) -> isize { self.head.0 }

    /// The y coordinate of the `Snake`'s head.
    pub fn y(&self) -> isize { self.head.1 }

    /// Sets the `Snake`'s direction to the given one if it doesn't [oppose](Direction::opposite()) the current one.
    pub fn turn(&mut self, dir: Direction) {
        if !self.dir.opposite(dir) {
            self.dir = dir;
        }
    }

    /// Moves the snake one space forward and appends it's previous location to the tail.
    ///
    /// The snake may end up [out of bounds](Snake::in_bounds) afterwards.
    pub fn forward(&mut self) {
        // Add head to the tail and put snake tile on the map.
        self.tail.push((self.x() as usize, self.y() as usize));

        // Move in the current direction.
        self.head.0 += self.dir.x();
        self.head.1 += self.dir.y();
    }

    /// Removes the last tail piece, if the tail reached the snake size.
    ///
    /// The tail piece will also be removed from the passed [Map](map::Map).
    pub fn cut_tail<const W: usize, const H: usize>(&mut self, map: &mut map::Map<W, H>) {
        if self.tail.len() >= self.size {
            let (x, y) = self.tail.remove(0);
            map.set(x, y, map::Tile::Empty);
        }
    }

    /// Returns the [Tile] at the location of the snake.
    ///
    /// This call is equivalent to
    /// ```
    /// # use rust_snake::map::Map;
    /// # use rust_snake::snake::Snake;
    /// # let map = Map::<1, 1>::new();
    /// # let snake = Snake::new(0, 0, 0);
    /// map.get(snake.x() as usize, snake.y() as usize);
    /// ```
    /// # Panics
    ///
    /// If the snake is [out of bounds](Snake::in_bounds).
    pub fn touching_tile<const W: usize, const H: usize>(&self, map: &map::Map<W, H>) -> map::Tile {
        map.get(self.x() as usize, self.y() as usize)
    }

    /// Sets the [Tile] at the location of the snake to a snake tile.
    ///
    /// This call is equivalent to
    /// ```
    /// # use rust_snake::map::{Map, Tile};
    /// # use rust_snake::snake::Snake;
    /// # let mut map = Map::<1, 1>::new();
    /// # let snake = Snake::new(0, 0, 0);
    /// map.set(snake.x() as usize, snake.y() as usize, Tile::Snake);
    /// ```
    /// # Panics
    ///
    /// If the snake is [out of bounds](Snake::in_bounds).
    pub fn place_head<const W: usize, const H: usize>(&self, map: &mut map::Map<W, H>) {
        map.set(self.x() as usize, self.y() as usize, map::Tile::Snake);
    }

    /// Returns if the snake is inside [Map](map::Map) boundaries.
    ///
    /// The boundaries range from `0`, inclusive,  to the map boundaries, exclusive.
    pub fn in_bounds<const W: usize, const H: usize>(&self, map: &map::Map<W, H>) -> bool {
        0 <= self.x() && 0 <= self.y() && map.in_bounds(self.x() as usize, self.y() as usize)
    }
}