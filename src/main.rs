//! # rust_snake
//!
//! Terminal implementation of the game "Snake".
//!
//! Captures user inputs and uses the active shell to display
//! a game of "Snake".
//!
//! # Controls
//!
//! Backspace : Exit the program
//! Arrow keys : turn

use std::sync::{Arc, Mutex};
use std::{io, thread};
use std::thread::JoinHandle;
use std::time::Duration;
use console::Key;
use rand::Rng;

/// The width of the [Map] area.
const MAP_WIDTH: usize = 15;
/// The Height of the [Map] area.
const MAP_HEIGHT: usize = 15;

/// Type used to store the position of snake and food tiles.
type Map = [[Tile; MAP_HEIGHT]; MAP_WIDTH];

/// The width of the displayed game in characters.
const GAME_WIDTH: usize = MAP_WIDTH * 2 + 2;

/// The four directions the [Snake] can face and `NONE` in case of a new snake.
#[derive(Copy, Clone)]
#[derive(PartialEq)]
enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN,
    NONE
}

impl Direction {
    /// The change on the x-axis.
    fn x(&self) -> isize {
        match self {
            Direction::LEFT => -1,
            Direction::RIGHT => 1,
            _ => 0
        }
    }

    /// The change on the y-axis.
    fn y(&self) -> isize {
        match self {
            Direction::UP => -1,
            Direction::DOWN => 1,
            _ => 0
        }
    }

    /// Returns if this and the given `Direction` oppose one another.
    ///
    /// # Examples
    /// ```
    /// assert!(Direction::LEFT.opposite(Direction::RIGHT))
    /// ```
    fn opposite(&self, dir: &Direction) -> bool {
        self.x() + dir.x() == 0 && self.y() + dir.y() == 0
    }
}

/// The Tiles contained in the [Map].
#[derive(PartialEq)]
#[derive(Copy, Clone)]
enum Tile {
    EMPTY,
    SNAKE,
    FOOD,
}

/// The Symbol representing a border tile.
const BORDER_SYMBOL: &str = "#";

/// The tiles contained in the [Map] representing game elements.
impl Tile {
    fn symbol(&self) -> &str {
        match self {
            Tile::EMPTY => " ",
            Tile::FOOD => "◯",
            Tile::SNAKE => "□"
        }
    }
}

/// Structure representing the snake.
struct Snake {
    head: (isize, isize),
    dir: Direction,
    size: usize,
    tail: Vec<(usize, usize)>,
}

impl Snake {
    /// Creates a new snake in the middle of the [Map] with a length of 3 and facing [NONE](Direction).
    fn new() -> Snake {
        Snake {
            head: ((MAP_WIDTH / 2) as isize, (MAP_HEIGHT / 2) as isize),
            dir: Direction::NONE,
            size: 3,
            tail: Vec::new(),
        }
    }

    /// Sets the `Snake`'s direction to the given one if it doesn't [oppose](Direction::opposite()) the current one.
    fn turn(&mut self, dir: Direction) -> () {
        if !self.dir.opposite(&dir) {
            self.dir = dir;
        }
    }

    /// Moves the `Snake` one space forward.
    ///
    /// The current position will be appended to the tail.
    /// If the tail's length reached the size of this `Snake`,
    /// the last tail piece is removed from the [Map]
    fn forward(&mut self, map: &mut Map) -> () {
        // Add head to the tail and put snake tile on the map.
        self.tail.push((self.x(), self.y()));
        map[self.x()][self.y()] = Tile::SNAKE;

        // Move in the current direction.
        self.head.0 += self.dir.x();
        self.head.1 += self.dir.y();

        // Remove the last tail piece, if size is reached.
        if self.tail.len() > self.size {
            let rem = self.tail.remove(0);
            map[rem.0][rem.1] = Tile::EMPTY;
        }
    }

    /// Returns if this `Snake` is out of the [Map] boundaries.
    ///
    /// The boundaries range from `0` to [MAP_WIDTH] / [MAP_HEIGHT].
    fn out_of_bounds(&self) -> bool {
        self.head.0 < 0
            || self.head.0 >= MAP_WIDTH as isize
            || self.head.1 < 0
            || self.head.1 >= MAP_HEIGHT as isize
    }

    /// The x coordinate of the `Snake`'s head.
    fn x(&self) -> usize { self.head.0 as usize }

    /// The y coordinate of the `Snake`'s head.
    fn y(&self) -> usize { self.head.1 as usize }
}

/// The time delay between every [Snake] move.
const DELAY: usize = 100;

/// The main Function.
///
/// Starts a new game of snake and terminates when the game ends.
fn main() {
    // Used to handle input and output.
    let term = Arc::new(console::Term::stdout());
    term.hide_cursor().ok();  //Ignore potentially occurring error.

    // A flag to determine if the game should keep running.
    let running = Arc::new(Mutex::new(true));

    // The last inputted direction.
    let dir = Arc::new(Mutex::new(Direction::NONE));

    // Start a thread capturing user inputs.
    let input_handle = capture_inputs(Arc::clone(&term),
                                      Arc::clone(&dir),
                                      Arc::clone(&running));

    // Initialize the map and snake.
    let mut map: Map = [[Tile::EMPTY; MAP_HEIGHT]; MAP_WIDTH];
    let mut snake = Snake::new();

    // Create a food and draw the map.
    make_food(&mut map);
    map[snake.x()][snake.y()] = Tile::SNAKE;
    draw(&term, &mut map).unwrap(); // Panic if unable to print map.

    // The game loop.
    while running.lock().map_or(false, |x| *x) {
        if let Ok(dir) = dir.lock() {
            // Turn the snake to the last inputted direction and move it forward.
            snake.turn(*dir);
        } else {
            // End the loop if the capturing thread panicked.
            break;
        }

        // Check if the user has inputted a valid direction
        if snake.dir != Direction::NONE {
            snake.forward(&mut map);

            // Clear the drawn map.
            term.clear_last_lines(MAP_HEIGHT + 2).unwrap(); // Panic if unable to clear the map

            // Check if the snake went out of bounds or ran into itself.
            if snake.out_of_bounds() || map[snake.x()][snake.y()] == Tile::SNAKE {
                term.write_line("Game Over!").unwrap();

                // Let the loop and the input capturing thread terminate.
                if let Ok(mut running) = running.lock() {
                    *running = false;
                } else {
                    break;
                }
            } else {
                // Check if the snake touched food.
                if map[snake.x()][snake.y()] == Tile::FOOD {
                    snake.size += 1;

                    // Generate a new food tile
                    make_food(&mut map);
                }

                // Set the current snake head position to a snake tile.
                map[snake.x()][snake.y()] = Tile::SNAKE;

                // Draw the map.
                draw(&term, &mut map).unwrap();
            }
        }

        // Sleep before attempting to move the snake again.
        spin_sleep::sleep(Duration::from_millis(DELAY as u64));
    }

    // Wait for the input capturing thread to terminate.
    input_handle.join().expect("Capturing user inputs failed.");

    // Restore the cursor location and visibility
    term.move_cursor_left(10).unwrap();
    term.show_cursor().unwrap();
}

/// Create a thread continuously capturing user inputs from the terminal.
///
/// It will repeatedly lock and update the [Direction] according to user inputs.
/// The thread will stop looping if `running` becomes `false`.
///
/// Note the arguments are wrapped in [Arc] and [Mutex], to allow shared ownership
/// and parallel access between game loop and the created thread.
fn capture_inputs(term: Arc<console::Term>, dir: Arc<Mutex<Direction>>, running: Arc<Mutex<bool>>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            // Capture the next input key.
            let key = term.read_key().unwrap();

            // lock the direction until the next loop iteration.
            let mut dir = dir.lock().unwrap();

            // Update the direction according to user input.
            match key {
                Key::ArrowLeft => *dir = Direction::LEFT,
                Key::ArrowRight => *dir = Direction::RIGHT,
                Key::ArrowUp => *dir = Direction::UP,
                Key::ArrowDown => *dir = Direction::DOWN,
                Key::Backspace => {
                    // Set the flag to stop the game.
                    *running.lock().unwrap() = false;
                }
                _ => (),
            };

            // Stop capturing inputs if the game stopped running.
            if !*running.lock().unwrap() {
                break;
            }
        }
    })
}

/// Create a food tile at a random location, which is not occupied by the snake.
fn make_food(map: &mut Map) -> () {
    let mut rng = rand::thread_rng();

    // Loop through random locations until an applicable one is found.
    loop {
        let fx = rng.gen_range(0..MAP_WIDTH);
        let fy = rng.gen_range(0..MAP_HEIGHT);

        if map[fx][fy] == Tile::EMPTY {
            map[fx][fy] = Tile::FOOD;

            break;
        }
    }
}

/// Prints out the map in the terminal.
///
/// The Map will be encased by a border, made up of [BORDER_SYMBOL].
///
/// Returns an [Err] if a terminal operation fails.
fn draw(term: &console::Term, map: &Map) -> io::Result<()> {
    // A full row filled with the border symbol.
    let border = &str::repeat(BORDER_SYMBOL, GAME_WIDTH);

    // Print the top border row
    term.write_line(border)?;
    term.move_cursor_left(GAME_WIDTH)?;

    // Iterate over the map rows and print them.
    for y in 0..MAP_HEIGHT {
        let mut line = String::new();

        line.push_str(BORDER_SYMBOL);
        line.push_str(map[0][y].symbol());

        // Append each symbol with spaces in between.
        for x in 1..MAP_WIDTH {
            line.push_str(" ");
            line.push_str(map[x][y].symbol());
        }
        line.push_str(" ");
        line.push_str(BORDER_SYMBOL);

        // Print the line and reset the cursor position.
        term.write_line(&line)?;
        term.move_cursor_left(GAME_WIDTH)?;
    }

    // Print the bottom border row.
    term.write_line(border)?;
    term.move_cursor_left(GAME_WIDTH)?;

    Ok(())
}