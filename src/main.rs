use std::{
    error::Error,
    io::{stdin, stdout, Stdout, Write},
};

use termion::{
    clear, color, cursor,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    screen::{self, AlternateScreen, IntoAlternateScreen},
    terminal_size,
};

struct Position {
    x: u16,
    y: u16,
}

struct Editor {
    should_quit: bool,
    screen: AlternateScreen<RawTerminal<Stdout>>,
    buffer: Vec<u8>,
    cursor_position: Position,
}

impl Editor {
    fn initalize() -> Editor {
        return Editor {
            should_quit: false,
            screen: stdout()
                .into_raw_mode()
                .unwrap()
                .into_alternate_screen()
                .unwrap(),
            buffer: Vec::new(),
            cursor_position: Position { x: 1, y: 1 },
        };
    }
    fn move_cursor(&mut self, n_x: u16, n_y: u16) -> Result<(), std::io::Error> {
        write!(self.screen, "{}", cursor::Goto(n_x, n_y))?;
        self.cursor_position.x = n_x;
        self.cursor_position.y = n_y;
        Ok(())
    }
    fn handle_key_event(&mut self, event: Key) -> Result<(), std::io::Error> {
        let (col, row) = terminal_size()?;
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
                }
                if v == '\n' {
                    write!(self.screen, "\r\n ")?;
                }
            }
            Key::Left => {
                if self.cursor_position.x > 2 {
                    self.move_cursor(self.cursor_position.x - 1, self.cursor_position.y)?;
                }
            }
            Key::Right => {
                if self.cursor_position.x < col {
                    self.move_cursor(self.cursor_position.x + 1, self.cursor_position.y)?;
                }
            }
            Key::Up => {
                if self.cursor_position.y > 3 {
                    self.move_cursor(self.cursor_position.x, self.cursor_position.y - 1)?;
                }
            }
            Key::Down => {
                if self.cursor_position.y < row - 5 {
                    self.move_cursor(self.cursor_position.x, self.cursor_position.y + 1)?;
                }
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
        let (mut _col, row) = termion::terminal_size()?;
        for _i in 0..row - 10 {
            write!(self.buffer, "~\r\n")?;
        }
        write!(self.buffer, "{}", cursor::Goto(2, 3))?;
        self.cursor_position.x = 2;
        self.cursor_position.y = 3;
        Ok(())
    }
    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        // hide the cursor
        write!(self.buffer, "{}", cursor::Hide)?;
        // move the cursor to the top left corner
        write!(self.buffer, "{}", cursor::Goto(1, 1))?;
        // welcome message
        self.display_hello_message()?;
        self.draw_rows()?;
        // show cursor
        write!(self.buffer, "{}", cursor::Show)?;
        self.screen.write_all(&self.buffer)?;
        self.buffer.clear();
        Ok(())
    }
    fn render(&mut self) -> Result<(), std::io::Error> {
        // write!(
        //     self.screen,
        //     "{}{}",
        //     termion::clear::All,
        //     termion::cursor::Goto(1, 1)
        // )?;

        // self.display_hello_message()?;
        // self.draw_rows()?;
        self.refresh_screen()?;
        self.screen.flush()?;

        for input in stdin().keys() {
            if let Ok(event) = input {
                self.handle_key_event(event)?;
            }
            if self.should_quit {
                break;
            }
        }
        Ok(())
    }
}

fn main() {
    let mut editor = Editor::initalize();
    editor.render().unwrap();
}
