extern crate pancurses;

use std::env;

use pancurses::{curs_set, endwin, initscr, noecho, Input, Window};
use pancurses as pc;

#[derive(Copy, Clone, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn rotate_left(self) -> Direction {
        use Direction::*;
        match self {
            Up => Left,
            Left => Down,
            Down => Right,
            Right => Up,
        }
    }
    fn rotate_right(self) -> Direction {
        use Direction::*;
        match self {
            Up => Right,
            Right => Down,
            Down => Left,
            Left => Up,
        }
    }
    fn offset(&self) -> (i8, i8) {
        use Direction::*;
        match *self {
            Up => (0, 1),
            Down => (0, -1),
            Left => (1, 0),
            Right => (-1, 0),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash)]
enum State {
    White,
    Black,
}

impl State {
    fn toggle(self) -> State {
        match self {
            State::White => State::Black,
            State::Black => State::White,
        }
    }
}

/// A cell in the grid.
#[derive(Copy, Clone, Debug, Hash)]
struct Cell {
    state: State,
}

struct Main {
    /// The terminal window
    window: Window,
    /// The grid.
    ///
    /// The cells are enumerated like you would read a book. Left to right, until you reach the
    /// line ending.
    grid: Box<[Box<[Cell]>]>,
    /// The x coordinate.
    x: u16,
    /// The y coordinate.
    y: u16,
    /// Current heading of the ant
    heading: Direction,
    /// Delay between steps
    delay: u64,
    /// Whether or not to show path
    path: bool,
    /// Whether or not to show step counter
    show_counter: bool,
}

fn init(w: u16, h: u16, window: Window, delay: u64, path: bool, counter: bool) {
    let mut main = Main {
        x: h / 2,
        y: w / 2,
        window,
        grid: vec![
            vec![
                Cell {
                    state: State::Black,
                };
                w as usize
            ].into_boxed_slice();
            h as usize
        ].into_boxed_slice(),
        heading: Direction::Right,
        delay,
        path,
        show_counter: counter,
    };

    // Start the loop.
    main.start();
}

impl Drop for Main {
    fn drop(&mut self) {
        // When done, restore the defaults to avoid messing with the terminal.
        endwin();
    }
}

impl Main {
    fn start(&mut self) {
        let mut index = 0;
        loop {
            index += 1;
            if self.show_counter {
            self.window.mvprintw(0, 0, &index.to_string());

            }
            if let Some(Input::Character('q')) = self.window.getch() {
                break;
            }

            // Offsets
            let (oy, ox) = self.heading.offset();
            self.x = (self.x as isize + ox as isize) as u16;
            self.y = (self.y as isize + oy as isize) as u16;

            let x = self.x as usize;
            let y = self.y as usize;
            if x >= self.grid.len() || y >= self.grid[x].len() {
                return;
            }

            let current = self.grid[x][y];

            let new_char = match current.state {
                State::White => {
                    self.grid[x][y].state = State::Black;
                    self.heading = self.heading.rotate_left();
                    if self.path {
                        "░"
                    } else {
                        " "
                    }
                }
                State::Black => {
                    self.grid[x][y].state = State::White;
                    self.heading = self.heading.rotate_right();
                    "█"
                }
            };
            self.window.mvaddstr(self.x as i32, self.y as i32, new_char);

            // Toggle current cells state
            self.grid[x][y].state = current.state.toggle();

            self.window.refresh();
            std::thread::sleep(std::time::Duration::from_millis(self.delay));
        }
    }
}

fn main() {
    let mut show_path = false;
    let mut delay = 20;
    let mut show_counter = true;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{}", HELP);
                std::process::exit(0)
            }
            "-p" | "--path" => {
                show_path = true;
            }
            "-d" | "--delay" => {
                delay = args.next()
                    .unwrap_or_else(|| {
                        eprintln!("No delay given.");
                        std::process::exit(1)
                    })
                    .parse()
                    .unwrap_or_else(|_| {
                        eprintln!("Invalid integer given");
                        std::process::exit(1)
                    });
            }
            "-c" | "--no-counter" => {
                show_counter = false;
            }
            _ => {}
        }
    }

    let window = initscr();
    noecho();
    curs_set(0);

    window.nodelay(true);
    let (rows, columns) = window.get_max_yx();

    if pc::has_colors() {
        pc::start_color();
    }

    pc::init_pair(1, pc::COLOR_BLACK, pc::COLOR_WHITE);
    window.bkgd(pc::COLOR_PAIR(1));

    init(
        columns as u16,
        rows as u16,
        window,
        delay,
        show_path,
        show_counter,
    );
}

const HELP: &'static str = r#"
langtons_ant: Simple terminal implementation of Langton's ant
flags:
    -h | --help        ~ This help page.
    -p | --path        ~ Show path
    -d | --delay       ~ Delay between steps in milliseconds, defaults to 20
    -c | --no-counter  ~ Hide step counter
"#;