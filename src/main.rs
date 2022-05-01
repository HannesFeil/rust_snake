use std::sync::{Arc, Mutex};
use std::{io, thread};
use std::time::Duration;
use console::Key;
use rand::Rng;

type Map = [[Tile; SCREEN_HEIGHT]; SCREEN_WIDTH];

#[derive(Copy, Clone)]
struct Direction {
    x: isize,
    y: isize
}

#[derive(Copy, Clone)]
#[derive(PartialEq)]
enum Tile {
    EMPTY,
    SNAKE,
    FOOD
}

struct Snake {
    head: (isize, isize),
    dir: Direction,
    size: usize,
    tail: Vec<(usize, usize)>
}

impl Snake {
    fn new() -> Snake {
        Snake {
            head: ((SCREEN_WIDTH / 2) as isize, (SCREEN_HEIGHT / 2) as isize),
            dir: RIGHT,
            size: 3,
            tail: Vec::new()
        }
    }

    fn turn(&mut self, dir: Direction) -> () {
        if !self.dir.opposite(&dir) {
            self.dir = dir;
        }
    }

    fn forward(&mut self, map: &mut Map) -> () {
        self.tail.push((self.x(), self.y()));
        map[self.x()][self.y()] = Tile::SNAKE;

        self.head.0 += self.dir.x;
        self.head.1 += self.dir.y;

        if self.tail.len() > self.size {
            let rem = self.tail.remove(0);
            map[rem.0][rem.1] = Tile::EMPTY;
        }
    }

    fn out_of_bounds(&self) -> bool {
        self.head.0 < 0 || self.head.0 >= SCREEN_WIDTH as isize || self.head.1 < 0 || self.head.1 >= SCREEN_HEIGHT as isize
    }

    fn touch_tail(&self) -> bool {
        self.tail.iter().any(|tail| { tail.0 == self.x() && tail.1 == self.y() })
    }

    fn x(&self) -> usize { self.head.0 as usize }

    fn y(&self) -> usize { self.head.1 as usize }
}

impl Direction {
    fn opposite(&self, dir: &Direction) -> bool {
        self.x + dir.x == 0 && self.y + dir.y == 0
    }
}

const DELAY: usize = 250;

const LEFT: Direction = Direction { x: -1, y: 0 };
const RIGHT: Direction = Direction { x: 1, y: 0 };
const UP: Direction = Direction { x: 0, y: -1 };
const DOWN: Direction = Direction { x: 0, y: 1 };

const SCREEN_WIDTH: usize = 10;
const SCREEN_HEIGHT: usize = 10;

const EMPTY_SYMBOL: &str = " ";
const BORDER_SYMBOL: &str = "#";
const SNAKE_SYMBOL: &str = "□";
const FOOD_SYMBOL: &str = "◯";

fn main() {
    let gui = false;

    if gui {

    } else {
        console_snake();
    }
}

fn console_snake() -> () {
    let term = Arc::new(console::Term::stdout());
    term.hide_cursor().unwrap();

    let dir = Arc::new(Mutex::new(LEFT));
    let running = Arc::new(Mutex::new(true));

    let dir_clone = Arc::clone(&dir);
    let term_clone = Arc::clone(&term);
    let running_clone = Arc::clone(&running);

    let handle = thread::spawn(move || {
        loop {
            let key = term_clone.read_key().unwrap();

            let mut dir = dir_clone.lock().unwrap();

            match key {
                Key::ArrowLeft => *dir = LEFT,
                Key::ArrowRight => *dir = RIGHT,
                Key::ArrowUp => *dir = UP,
                Key::ArrowDown => *dir = DOWN,
                Key::Backspace => {
                    *running_clone.lock().unwrap() = false;
                },
                _ => (),
            };

            if !*running_clone.lock().unwrap() {
                break;
            }
        }
    });

    let mut map: Map = [[Tile::EMPTY; SCREEN_HEIGHT]; SCREEN_WIDTH];

    let mut snake = Snake::new();

    make_food(&mut map);
    draw(&term, &mut map).unwrap();

    loop {
        snake.turn(*dir.lock().unwrap());
        snake.forward(&mut map);

        term.clear_last_lines(SCREEN_HEIGHT + 2).unwrap();
        if snake.out_of_bounds() || snake.touch_tail() {
            term.write_line("Game Over!").unwrap();

            *running.lock().unwrap() = false;
        } else {
            if map[snake.x()][snake.y()] == Tile::FOOD {
                snake.size += 1;

                make_food(&mut map);
            }

            map[snake.x()][snake.y()] = Tile::SNAKE;

            draw(&term, &mut map).unwrap();
        }

        if !*running.lock().unwrap() {
            break;
        } else {
            spin_sleep::sleep(Duration::from_millis(DELAY as u64));
        }
    }

    handle.join().unwrap();

    term.move_cursor_left(10).unwrap();
    term.show_cursor().unwrap();
}

fn make_food(map: &mut Map) -> () {
    let mut rng = rand::thread_rng();

    loop {
        let fx = rng.gen_range(0..SCREEN_WIDTH);
        let fy = rng.gen_range(0..SCREEN_HEIGHT);

        if map[fx][fy] == Tile::EMPTY {
            map[fx][fy] = Tile::FOOD;

            break;
        }
    }
}

fn draw(term: &console::Term, map: &Map) -> io::Result<()> {
    let border = &str::repeat(BORDER_SYMBOL, SCREEN_WIDTH * 2 as usize + 3);

    term.write_line(border)?;
    term.move_cursor_left(SCREEN_WIDTH as usize * 2 + 3)?;
    for y in 0..SCREEN_HEIGHT {
        let mut line = String::new();

        line.push_str(BORDER_SYMBOL);
        for x in 0..SCREEN_WIDTH {
            line.push_str(" ");
            line.push_str(match map[x][y] {
                Tile::SNAKE => SNAKE_SYMBOL,
                Tile::FOOD => FOOD_SYMBOL,
                Tile::EMPTY => EMPTY_SYMBOL
            });
        }
        line.push_str(" ");
        line.push_str(BORDER_SYMBOL);

        term.write_line(&line)?;
        term.move_cursor_left(SCREEN_WIDTH as usize * 2 + 3)?;
    }
    term.write_line(border)?;
    term.move_cursor_left(SCREEN_WIDTH as usize * 2 + 3)?;

    Ok(())
}