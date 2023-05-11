//! # rust_snake
//!
//! Implementation of the game "Snake" through the struct [Game].
//!
//! A game only handles the logic behind snake. It is up to the programmer, to implement a game
//! loop, capture user inputs and display the game. However existing functions [Game::move_snake],
//! [Game::turn_snake] and [Game::display] should make this easy.
//!
//! Note the game has three states, of which only [State::GameOver] is used internally, while
//! the other two are meant to be interpret and altered by the programmer.
//!
//! # Example
//! ```
//! use rust_snake::{Game, snake, State};
//! let mut game = Game::<10, 10>::new(); //Initialize a new game
//!
//! while game.state != State::GameOver {
//!     let user_input = snake::Direction::Left; // Capture user inputs
//!     game.turn_snake(user_input);
//!     game.move_snake();
//!
//!     game.display(|x| {
//!         // Some display function
//!     });
//! }
//! ```

use rand::Rng;

pub mod map;
pub mod snake;

/// The initial size of the snake.
const INITIAL_SNAKE_SIZE: usize = 3;

/// The different states the [Game] can be in.
#[derive(PartialEq)]
pub enum State {
    Running,
    Paused,
    GameOver,
}

/// Struct representing the state of the game and offering methods to alter it.
pub struct Game<const W: usize, const H: usize> {
    map: map::Map<W, H>,
    snake: snake::Snake,
    pub state: State,
}

impl<const W: usize, const H: usize> Game<W, H> {
    /// Creates a new game with the snake in the middle, facing [None](snake::Direction) and
    /// [paused](State).
    ///
    /// The head of the snake will be placed on the map and a food tile will be
    /// [generated](Game::create_food).
    pub fn new() -> Self {
        let mut game = Game {
            map: map::Map::<W, H>::new(),
            snake: snake::Snake::new(W / 2, H / 2, INITIAL_SNAKE_SIZE),
            state: State::Paused,
        };

        game.snake.place_head(&mut game.map);
        game.create_food();

        game
    }

    /// Tries to turn the snake in the given direction, see [snake::Snake::turn].
    pub fn turn_snake(&mut self, dir: snake::Direction) {
        self.snake.turn(dir);
    }

    /// Moves the snake forward.
    ///
    /// If the snake touches a food tile, the size of the snake will increase by one.
    /// The game state will be set to [GameOver](State) if the snake goes out of bounds or touches
    /// itself.
    ///
    /// Additionally the map will be updated accordingly.
    pub fn move_snake(&mut self) {
        // Move the snake.
        self.snake.forward();
        self.snake.cut_tail(&mut self.map);

        // Check if its in bounds and colliding with something.
        if self.snake.in_bounds(&self.map) {
            match self.snake.touching_tile(&self.map) {
                map::Tile::Snake => {
                    // The snake ran into itself, game over.
                    self.game_over();
                }
                map::Tile::Food => {
                    // Increase the snake size and create a new food tile.
                    self.snake.size += 1;
                    self.create_food();
                }
                map::Tile::Empty => (),
            }

            // Update the snake head on the map
            self.snake.place_head(&mut self.map);
        } else {
            // The snake went out of bounds, game over.
            self.game_over();
        }
    }

    /// Create a food tile on a random, previously unoccupied space.
    pub fn create_food(&mut self) {
        let mut rng = rand::thread_rng();

        // Loop through random locations until an applicable one is found.
        loop {
            let fx = rng.gen_range(0..W);
            let fy = rng.gen_range(0..H);

            // End the loop and place the food, if the tile is unoccupied.
            if self.map.get(fx, fy) == map::Tile::Empty {
                self.map.set(fx, fy, map::Tile::Food);

                break;
            }
        }
    }

    /// Calls the given function with the map of this game, containing empty, snake and food
    /// [tiles](map::Tile).
    ///
    /// This function is intended to only display the state of the map, therefor it receives an
    /// immutable reference.
    ///
    /// Note this method is not innately concurrent, so expensive rendering should be handled
    /// with care.
    ///
    /// # Examples
    /// ```
    /// use rust_snake::{Game, map};
    /// let game = Game::<10, 10>::new();
    /// game.display(|map| {
    ///     for x in 0..10 {
    ///         for y in 0..10 {
    ///             match map.get(x, y) {
    ///                 // display the tile accordingly
    ///                 map::Tile::Empty => (),
    ///                 // ...
    ///                 # _ => ()
    ///             }
    ///         }
    ///     }
    /// })
    pub fn display<F, R>(&self, func: F) -> R
    where
        F: FnOnce(&map::Map<W, H>) -> R,
    {
        func(&self.map)
    }

    /// Gets called when the snake moves out of bounds or into itself.
    ///
    /// Currently this method only sets the game state to [GameOver](State).
    pub fn game_over(&mut self) {
        self.state = State::GameOver;
    }

    /// Clears the map, initializes a new snake and sets the state to [Paused](State).
    pub fn restart(&mut self) {
        for x in 0..W {
            for y in 0..H {
                self.map.set(x, y, map::Tile::Empty);
            }
        }

        self.snake = snake::Snake::new(W / 2, H / 2, INITIAL_SNAKE_SIZE);
        self.snake.place_head(&mut self.map);
        self.create_food();
        self.state = State::Paused;
    }
}

impl<const W: usize, const H: usize> Default for Game<W, H> {
    fn default() -> Self {
        Game::new()
    }
}
