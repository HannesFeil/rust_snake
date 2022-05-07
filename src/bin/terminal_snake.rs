extern crate rust_snake;

use std::sync::{Arc, mpsc};
use std::{io, thread, time};
use rust_snake::{Game, map, snake, State};

/// The width of the map.
const MAP_WIDTH: usize = 15;
/// The Height of the map area.
const MAP_HEIGHT: usize = 15;

/// The width of the displayed game in characters.
const GAME_WIDTH: usize = MAP_WIDTH * 2 + 2;

/// The time delay between every snake move, in milliseconds.
const DELAY: usize = 100;

/// The main Function.
///
/// Starts a new game of snake in the terminal and terminates when the game ends.
pub fn main() {
    // Used to handle input and output.
    let term = Arc::new(console::Term::stdout());
    term.hide_cursor().ok();  //Ignore potentially occurring error.

    // Used to transfer directions from the input thread to the main thread.
    let (sender, receiver) = mpsc::channel();

    // Start a thread capturing user inputs.
    let input_handle = capture_inputs(Arc::clone(&term), sender);

    // Initialize the game.
    let mut game = Game::<MAP_WIDTH, MAP_HEIGHT>::new();

    // Draw the map and panic if the draw function returns an error.
    game.display(|map| draw(&term, map)).unwrap();

    // The game loop.
    while game.state != State::GameOver {
        if let Some(dir) = receiver.try_iter().last() {
            match dir {
                // End the game if the user inputs a backspace
                snake::Direction::None => game.game_over(),
                _ => {
                    // Turn the snake to the last inputted direction and move it forward.
                    game.state = State::Running;
                    game.turn_snake(dir);
                }
            }
        }

        // Check if the game isn't paused
        if game.state == State::Running {
            game.move_snake();

            // Clear the last outputted map.
            term.clear_last_lines(MAP_HEIGHT + 2).unwrap(); // Panic if unable to clear the map

            // Draw the map.
            game.display(|map| draw(&term, map)).unwrap();
        }

        // Sleep before attempting to move the snake again.
        spin_sleep::sleep(time::Duration::from_millis(DELAY as u64));
    }

    term.write_line("Game Over!").unwrap();
    term.move_cursor_left(10).unwrap();
    spin_sleep::sleep(time::Duration::from_secs(1));
    term.write_line("Press any key to continue...").unwrap();

    // Drop the receiver, so the input thread terminates.
    drop(receiver);

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
fn capture_inputs(term: Arc<console::Term>, sender: mpsc::Sender<snake::Direction>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut last = snake::Direction::None;
        loop {
            // Capture the next input key.
            let key = term.read_key().unwrap();

            // Update the direction according to user input.
            let dir = match key {
                console::Key::ArrowLeft => Some(snake::Direction::Left),
                console::Key::ArrowRight => Some(snake::Direction::Right),
                console::Key::ArrowUp => Some(snake::Direction::Up),
                console::Key::ArrowDown => Some(snake::Direction::Down),
                console::Key::Backspace => Some(snake::Direction::None),
                _ => None
            };

            if let Some(dir) = dir {
                last = dir;
                if sender.send(dir).is_err() {
                    break;
                }
            } else if sender.send(last).is_err() {
                break;
            }
        }
    })
}

const BORDER_SYMBOL: &str = "#";

/// Prints out the map in the terminal.
///
/// The Map will be encased by a border, made up of [BORDER_SYMBOL].
///
/// Returns an [Err] if a terminal operation fails.
fn draw<const W: usize, const H: usize>(term: &console::Term, map: &map::Map<W, H>) -> io::Result<()> {
    // A full row filled with the border symbol.
    let border = &str::repeat(BORDER_SYMBOL, GAME_WIDTH);

    // Print the top border row
    term.write_line(border)?;
    term.move_cursor_left(GAME_WIDTH)?;

    // Iterate over the map rows and print them.
    for y in 0..MAP_HEIGHT {
        let mut line = String::new();

        line.push_str(BORDER_SYMBOL);
        line.push_str(match map.get(0, y) {
            map::Tile::Empty => " ",
            map::Tile::Snake => "□",
            map::Tile::Food => "◯"
        });

        // Append each symbol with spaces in between.
        for x in 1..MAP_WIDTH {
            line.push(' ');
            line.push_str(match map.get(x, y) {
                map::Tile::Empty => " ",
                map::Tile::Snake => "□",
                map::Tile::Food => "◯"
            });
        }
        line.push(' ');
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