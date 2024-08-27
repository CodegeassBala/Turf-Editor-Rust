use std::{
    cmp::min,
    error::Error,
    io::{stdout, Write},
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyModifiers},
    style::Print,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand, QueueableCommand,
};

struct Vieew;

impl Vieew {
    fn paint_rows(&mut self) -> Result<(), Box<dyn Error>> {
        let mut stdout = stdout();
        stdout.queue(Hide)?;
        let (rows, _cols) = size()?;
        for i in 0..rows {
            stdout.queue(MoveTo(0, i))?.queue(Print("~"))?;
        }
        stdout
            .queue(MoveTo(0, 0))?
            .queue(Print("Welcome to Turf Editor, Happy Editing!!!"))?
            .queue(MoveTo(1, 1))?
            .queue(Show)?;
        stdout.flush()?;
        Ok(())
    }
}

pub struct Editor {
    cursor_position: (u16, u16),
    terminal_size: (u16, u16),
}

impl Editor {
    pub fn new() -> Self {
        Editor {
            cursor_position: (0, 0),
            terminal_size: terminal::size().unwrap(),
        }
    }
    pub fn move_cursor(&mut self, col: u16, row: u16) -> Result<(), Box<dyn Error>> {
        stdout().queue(MoveTo(col, row))?;
        self.cursor_position = (col, row);
        Ok(())
    }
    pub fn set_cursor(&mut self, col: u16, row: u16) {
        self.cursor_position = (col, row);
    }
    pub fn move_cursor_left(&mut self) -> Result<(), Box<dyn Error>> {
        let (col, row) = self.cursor_position;
        stdout().queue(MoveTo(if col > 1 { col - 1 } else { col }, row))?;
        stdout().queue(Print(""))?;
        self.cursor_position = (if col > 1 { col - 1 } else { col }, row);
        Ok(())
    }
    fn handle_key_event(&mut self, code: KeyCode) -> Result<(), Box<dyn Error>> {
        let (col, row) = self.cursor_position;
        match code {
            KeyCode::Down => {
                self.move_cursor(col, min(self.terminal_size.1, row + 1))?;
                stdout().flush()?;
                Ok(())
            }
            KeyCode::Left => {
                self.move_cursor(if col > 1 { col - 1 } else { col }, row)?;
                stdout().flush()?;
                Ok(())
            }
            KeyCode::Right => {
                self.move_cursor(min(col + 1, self.terminal_size.0), row)?;
                stdout().flush()?;
                Ok(())
            }
            KeyCode::Up => {
                self.move_cursor(col, if row > 1 { row - 1 } else { row })?;
                stdout().flush()?;
                Ok(())
            }
            KeyCode::PageUp => {
                self.move_cursor(col, 1)?;
                stdout().flush()?;
                Ok(())
            }
            KeyCode::PageDown => {
                self.move_cursor(col, row)?;
                stdout().flush()?;
                Ok(())
            }
            KeyCode::Home => {
                self.move_cursor(1, row)?;
                stdout().flush()?;
                Ok(())
            }
            KeyCode::End => {
                self.move_cursor(col, row)?;
                stdout().flush()?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        stdout().execute(EnterAlternateScreen)?;
        Vieew.paint_rows()?;
        enable_raw_mode().unwrap();
        self.move_cursor(1, 1)?;
        let mut handle = stdout().lock();
        loop {
            if let Ok(Event::Key(e)) = read() {
                if let KeyCode::Char(c) = e.code {
                    if c == 'c' && e.modifiers.contains(KeyModifiers::CONTROL) {
                        break;
                    } else {
                        stdout().execute(Print(c))?;
                        self.cursor_position = (self.cursor_position.0 + 1, self.cursor_position.1);
                        handle.flush().unwrap();
                    }
                } else {
                    self.handle_key_event(e.code)?;
                }
            }
        }
        disable_raw_mode().unwrap();
        stdout().execute(Clear(ClearType::FromCursorDown))?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
}
