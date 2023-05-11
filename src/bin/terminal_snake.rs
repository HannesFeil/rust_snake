extern crate rust_snake;

use crossterm::style::{Color, Stylize};
use crossterm::{cursor, event, execute, style, ExecutableCommand, QueueableCommand};
use rust_snake::{map, snake, Game, State};
use std::io::{stdout, Write};
use std::sync::mpsc;
use std::{io, thread, time};

/// The width of the map.
const MAP_WIDTH: usize = 30;
/// The Height of the map area.
const MAP_HEIGHT: usize = 30;
/// The time delay between every snake move, in milliseconds.
const DELAY: usize = 100;

/// The main Function.
///
/// Starts a new game of snake in the terminal and terminates when the game ends.
pub fn main() {
    // Used to transfer directions from the input thread to the main thread.
    let (sender, receiver) = mpsc::channel();

    crossterm::terminal::enable_raw_mode().unwrap();
    execute!(stdout(), crossterm::terminal::EnterAlternateScreen).unwrap();

    // Start a thread capturing user inputs.
    let input_handle = capture_inputs(sender);

    // Initialize the game.
    let mut game = Game::<MAP_WIDTH, MAP_HEIGHT>::new();

    execute!(stdout(), cursor::Hide).unwrap();
    for _ in 0..MAP_HEIGHT + 1 {
        write!(stdout(), "\n").unwrap();
    }
    execute!(stdout(), cursor::MoveToPreviousLine(MAP_HEIGHT as u16 + 2)).unwrap();

    // Draw the map and panic if the draw function returns an error.
    game.display(|map| draw(map).unwrap());

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
            stdout()
                .execute(crossterm::terminal::Clear(
                    crossterm::terminal::ClearType::All,
                ))
                .unwrap(); // Panic if unable to clear the map
            stdout()
                .execute(cursor::MoveToPreviousLine(MAP_HEIGHT as u16 + 1))
                .unwrap();

            // Draw the map.
            game.display(|map| draw(map)).unwrap();
        }

        // Sleep before attempting to move the snake again.
        spin_sleep::sleep(time::Duration::from_millis(DELAY as u64));
    }

    execute!(
        stdout(),
        cursor::MoveToNextLine(1),
        style::SetForegroundColor(Color::Red),
        style::SetAttribute(style::Attribute::Bold),
        style::Print("Game Over!\r\n"),
        style::ResetColor,
        style::SetAttribute(style::Attribute::Reset),
    )
    .unwrap();

    spin_sleep::sleep(time::Duration::from_secs(1));

    write!(stdout(), "Press any Key to continue ... \r\n").unwrap();

    // Drop the receiver, so the input thread terminates.
    drop(receiver);

    // Wait for the input capturing thread to terminate.
    input_handle
        .join()
        .expect("joining threads failed")
        .expect("Capturing user inputs failed.");

    crossterm::terminal::disable_raw_mode().unwrap();
    execute!(stdout(), crossterm::terminal::LeaveAlternateScreen).unwrap();
    execute!(stdout(), cursor::Show).unwrap();
}

/// Create a thread continuously capturing user inputs from the terminal.
///
/// It will repeatedly lock and update the [Direction] according to user inputs.
/// The thread will stop looping if `running` becomes `false`.
///
/// Note the arguments are wrapped in [Arc] and [Mutex], to allow shared ownership
/// and parallel access between game loop and the created thread.
fn capture_inputs(
    sender: mpsc::Sender<snake::Direction>,
) -> thread::JoinHandle<crossterm::Result<()>> {
    thread::spawn(move || {
        let mut last = snake::Direction::None;
        loop {
            // Capture the next input key.
            let key = event::read()?;

            // Update the direction according to user input.
            let dir = match key {
                event::Event::Key(event::KeyEvent {
                    code,
                    kind: event::KeyEventKind::Press,
                    ..
                }) => match code {
                    event::KeyCode::Left => Some(snake::Direction::Left),
                    event::KeyCode::Right => Some(snake::Direction::Right),
                    event::KeyCode::Up => Some(snake::Direction::Up),
                    event::KeyCode::Down => Some(snake::Direction::Down),
                    event::KeyCode::Backspace => Some(snake::Direction::None),
                    _ => None,
                },
                _ => None,
            };

            if let Some(dir) = dir {
                if !last.opposite(dir) || dir == snake::Direction::None {
                    last = dir;

                    if sender.send(dir).is_err() {
                        break;
                    }
                }
            } else if sender.send(last).is_err() {
                break;
            }
        }
        Ok(())
    })
}

/// Prints out the map in the terminal.
///
/// The Map will be encased by a border, made up of [BORDER_SYMBOL].
///
/// Returns an [Err] if a terminal operation fails.
fn draw<const W: usize, const H: usize>(map: &map::Map<W, H>) -> io::Result<()> {
    let border = "  ".on(Color::DarkGrey);
    let snake = "  ".on(Color::Green);
    let food = "  ".on(Color::Yellow);
    let empty = "  ".on(Color::Black);

    for _ in 0..W + 2 {
        stdout().queue(style::PrintStyledContent(border)).unwrap();
    }
    stdout().queue(cursor::MoveToNextLine(1)).unwrap();
    // Iterate over the map rows and print them.
    for y in 0..MAP_HEIGHT {
        stdout().queue(style::PrintStyledContent(border)).unwrap();

        // Append each symbol with spaces in between.
        for x in 0..MAP_WIDTH {
            stdout()
                .queue(style::PrintStyledContent(match map.get(x, y) {
                    map::Tile::Empty => empty,
                    map::Tile::Snake => snake,
                    map::Tile::Food => food,
                }))
                .unwrap();
        }

        stdout().queue(style::PrintStyledContent(border)).unwrap();
        stdout().queue(cursor::MoveToNextLine(1)).unwrap();
    }

    for _ in 0..W + 2 {
        stdout().queue(style::PrintStyledContent(border)).unwrap();
    }

    stdout().flush().unwrap();

    Ok(())
}
