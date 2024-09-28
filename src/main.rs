use std::{
    cmp::min,
    env,
    fs::File,
    io::{self, stdin, stdout, BufRead, Stdout, Write},
    path::PathBuf,
};

use termion::{
    clear, color, cursor,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    screen::{AlternateScreen, IntoAlternateScreen, ToMainScreen},
    terminal_size,
};
#[derive(Debug)]

struct Position {
    x: u16,
    y: u16,
}

#[derive(Debug)]
struct Content {
    file_content: Vec<String>,
    num_lines: u16,
    row_off: u16,
    col_off: u16,
}

struct Editor {
    should_quit: bool,
    screen: AlternateScreen<RawTerminal<Stdout>>,
    buffer: Vec<u8>,
    cursor_position: Position,
    content: Content,
    refresh: bool,
}

impl Editor {
    fn initalize(content: Content) -> Editor {
        return Editor {
            should_quit: false,
            screen: stdout()
                .into_raw_mode()
                .unwrap()
                .into_alternate_screen()
                .unwrap(),
            buffer: Vec::new(),
            cursor_position: Position { x: 2, y: 3 },
            content,
            refresh: false,
        };
    }
    fn move_cursor(&mut self, n_x: u16, n_y: u16) -> Result<(), std::io::Error> {
        write!(self.screen, "{}", cursor::Goto(n_x, n_y))?;
        self.cursor_position.x = n_x;
        self.cursor_position.y = n_y;
        Ok(())
    }
    fn set_cursor(&mut self, n_x: u16, n_y: u16) {
        // print!("{}{}", n_x, n_y);
        self.cursor_position.x = n_x;
        self.cursor_position.y = n_y;
    }
    fn handle_scroll(&mut self, current_line_index: usize) -> Result<(), std::io::Error> {
        let (col, row) = terminal_size()?;
        let y_limit = min(self.content.num_lines - self.content.row_off + 3, row);
        let current_line_len = self.content.file_content[current_line_index].len() as u16;
        let x_limit = min(current_line_len - self.content.col_off + 2, col);
        if self.cursor_position.y >= y_limit {
            self.content.row_off = min(self.content.num_lines - 1, self.content.row_off + 1);
            self.set_cursor(self.cursor_position.x, self.cursor_position.y - 1);
            self.refresh = true;
        }
        if self.cursor_position.y < 3 {
            self.content.row_off = if self.content.row_off > 0 {
                self.content.row_off - 1
            } else {
                0
            };
            self.set_cursor(self.cursor_position.x, self.cursor_position.y + 1);
            self.refresh = true;
        }
        if self.cursor_position.x >= x_limit {
            if x_limit == 2 {
                self.set_cursor(self.cursor_position.x - 1, self.cursor_position.y);
            } else {
                self.content.col_off = min(current_line_len - 1, self.content.col_off + 1);
                self.set_cursor(self.cursor_position.x - 1, self.cursor_position.y);
                self.refresh = true;
            }
        }
        if self.cursor_position.x < 2 {
            self.content.col_off = if self.content.col_off > 0 {
                self.content.col_off - 1
            } else {
                0
            };
            self.set_cursor(self.cursor_position.x + 1, self.cursor_position.y);
            self.refresh = true;
        }
        self.move_cursor(self.cursor_position.x, self.cursor_position.y)?;
        Ok(())
    }
    fn handle_arrrow_keys(&mut self, event: Key) -> Result<(), std::io::Error> {
        let current_line_index = (self.cursor_position.y - 3 + self.content.row_off) as usize;
        match event {
            Key::Left => {
                self.set_cursor(self.cursor_position.x - 1, self.cursor_position.y);
            }
            Key::Right => {
                self.set_cursor(self.cursor_position.x + 1, self.cursor_position.y);
            }
            Key::Up => {
                self.set_cursor(self.cursor_position.x, self.cursor_position.y - 1);
            }
            Key::Down => {
                self.set_cursor(self.cursor_position.x, self.cursor_position.y + 1);
            }
            _ => {}
        }
        self.handle_scroll(current_line_index)?;
        Ok(())
    }
    fn handle_key_event(&mut self, event: Key) -> Result<(), std::io::Error> {
        match event {
            Key::Ctrl(c) => {
                if c == 'a' {
                    self.should_quit = true;
                } else {
                    write!(self.screen, "{c}")?;
                }
            }
            Key::Char(v) => {
                if v != '\n' && v != '\r' && v != '\t' {
                    write!(self.screen, "{v}")?;
                    self.set_cursor(self.cursor_position.x + 1, self.cursor_position.y);
                }
                if v == '\n' {
                    self.move_cursor(2, self.cursor_position.y + 1)?;
                }
            }
            Key::Up | Key::Down | Key::Left | Key::Right => {
                self.handle_arrrow_keys(event)?;
            }
            Key::Delete => {}
            _ => {}
        }
        stdout().flush()?;
        Ok(())
    }
    fn display_hello_message(&mut self) -> Result<(), std::io::Error> {
        write!(
            self.buffer,
            "{}Welcome To Turf Editor 2.0",
            color::Fg(color::LightBlue)
        )?;

        Ok(())
    }
    fn draw_rows(&mut self) -> Result<(), std::io::Error> {
        write!(
            self.buffer,
            "{}{}{}",
            color::Fg(color::LightWhite),
            cursor::Goto(1, 3),
            clear::CurrentLine
        )?;
        let (col, row) = termion::terminal_size()?;
        for i in 3..row {
            let line_index = (i - 3) + self.content.row_off;
            let threshold = (col - 2 + self.content.col_off) as usize;
            write!(self.buffer, "{}", clear::CurrentLine)?;
            if let Some(line) = self.content.file_content.get(line_index as usize) {
                let current_line_len = line.len() as u16;
                if current_line_len <= self.content.col_off {
                    write!(self.buffer, " \r\n")?;
                } else {
                    let slice = &line[(self.content.col_off as usize)..line.len().min(threshold)];
                    write!(self.buffer, " {}\r\n", slice)?;
                }
            }
        }
        Ok(())
    }
    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        // hide the cursor
        write!(self.buffer, "{}{}", cursor::Hide, cursor::Goto(1, 1))?;
        // move the cursor to the top left corner
        // write!(self.buffer, "{}", cursor::Goto(1, 1))?;
        // welcome message
        self.display_hello_message()?;
        self.draw_rows()?;
        // show cursor
        write!(
            self.buffer,
            "{}{}",
            cursor::Goto(self.cursor_position.x, self.cursor_position.y),
            cursor::Show
        )?;
        self.screen.write_all(&self.buffer)?;
        self.screen.flush()?;
        self.buffer.clear();
        Ok(())
    }
    fn render(&mut self) -> Result<(), std::io::Error> {
        self.refresh_screen()?;
        self.screen.flush()?;

        for input in stdin().keys() {
            if let Ok(event) = input {
                self.handle_key_event(event)?;
            }
            if self.refresh {
                print!("{}", self.content.col_off);
                self.refresh_screen()?;
                self.refresh = false;
            }
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        println!("Code Paniked!! {info}");
    }));

    let args: Vec<String> = env::args().collect();

    // Ensure a file name argument is provided
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        return;
    }

    let current_dir = env::current_dir().unwrap();

    // Get the file name from the arguments
    let file_name = &args[1];

    // Construct the file path relative to the current directory
    let mut file_path = PathBuf::from(current_dir);
    file_path.push(file_name);

    // Open the file
    let file = File::open(file_path).unwrap();

    // Create a BufReader to read the file line by line
    let reader = io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>().unwrap();
    let num_lines = lines.len() as u16;
    let content: Content = Content {
        file_content: lines,
        row_off: 0,
        col_off: 0,
        num_lines,
    };
    // println!("{:?}", content);
    let mut editor = Editor::initalize(content);
    match editor.render() {
        Ok(_) => {}
        Err(err) => {
            write!(stdout(), "{}", ToMainScreen).expect_err("Failed to move to main screen");
            println!("{:?}", err);
            loop {}
        }
    }

    error::Error,
    io::{stdin, stdout, Write},
};

use termion::{
    color, cursor, event::Key, input::TermRead, raw::IntoRawMode, screen::IntoAlternateScreen,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut screen = stdout()
        .into_raw_mode()
        .unwrap()
        .into_alternate_screen()
        .unwrap();
    write!(
        screen,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    )?;
    write!(
        screen,
        "{}Welcome To Turf Editor 2.0",
        color::Fg(color::LightBlue)
    )?;
    write!(
        screen,
        "{}{}",
        color::Fg(color::LightWhite),
        cursor::Goto(1, 3)
    )?;
    screen.flush()?;
    for input in stdin().keys() {
        if let Ok(char) = input {
            match char {
                Key::Ctrl('a') => break,
                Key::Char(v) => {
                    print!("{v}");
                }
                _ => {}
            }
            stdout().flush()?;
        }
    }
    Ok(())
}
