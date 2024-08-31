use std::{
    cmp::min,
    env,
    error::Error,
    fs::File,
    io::{stdout, BufRead, BufReader, Write},
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    style::Print,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand, QueueableCommand,
};

struct Buffer {
    lines: Vec<String>,
}

struct View {
    buffer: Buffer,
    file_argument: Option<String>,
}
impl Buffer {
    pub fn new() -> Self {
        Buffer {
            lines: Vec::<String>::new(),
        }
    }
}

impl View {
    pub fn new(file_argument: Option<String>) -> Self {
        View {
            buffer: Buffer::new(),
            file_argument,
        }
    }
    pub fn display_file(&mut self, file_name: String) -> Result<u16, Box<dyn Error>> {
        let current_dir = env::current_dir()?;
        let file_path = current_dir.join(file_name);
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let content = line?;
            self.buffer.lines.push(content.clone());
            stdout().queue(Print(format!("{content}\r\n")))?;
        }
        let no_of_lines_printed = self.buffer.lines.len() as u16;
        Ok(no_of_lines_printed)
    }
    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        let (_csize, r_size) = terminal::size()?;
        let mut lines_printed = 0;
        match self.file_argument {
            Some(ref file_arg) => {
                lines_printed = self.display_file(file_arg.clone())?;
            }
            _ => {}
        }
        while lines_printed < r_size - 1 {
            stdout().queue(Print("~\r\n"))?;
            lines_printed += 1;
        }
        stdout().queue(Show)?;
        stdout().flush()?;
        Ok(())
    }
}

pub struct Editor {
    cursor_position: (u16, u16),
    terminal_size: (u16, u16),
    view: View,
    should_quit: bool,
}

impl Editor {
    pub fn new(file_name: Option<String>) -> Self {
        Editor {
            cursor_position: (0, 0),
            terminal_size: terminal::size().unwrap(),
            view: View::new(file_name),
            should_quit: false,
        }
    }
    pub fn move_cursor(&mut self, col: u16, row: u16) -> Result<(), Box<dyn Error>> {
        stdout().execute(MoveTo(col, row))?;
        self.cursor_position = (col, row);
        Ok(())
    }
    fn handle_event(&mut self, event: KeyEvent) -> Result<(), Box<dyn Error>> {
        let KeyEvent {
            code, modifiers, ..
        } = event;
        match code {
            KeyCode::Char(c) => {
                if modifiers.contains(KeyModifiers::CONTROL) && c == 'c' {
                    self.should_quit = true;
                    Ok(())
                } else {
                    stdout().execute(Print(c))?;
                    Ok(())
                }
            }
            KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Up
            | KeyCode::Home
            | KeyCode::End
            | KeyCode::PageDown
            | KeyCode::PageUp => {
                self.handle_key_event(code)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn handle_key_event(&mut self, code: KeyCode) -> Result<(), Box<dyn Error>> {
        let (col, row) = self.cursor_position;
        match code {
            KeyCode::Down => {
                self.move_cursor(col, min(self.terminal_size.1, row + 1))?;
                Ok(())
            }
            KeyCode::Left => {
                self.move_cursor(if col > 1 { col - 1 } else { col }, row)?;
                Ok(())
            }
            KeyCode::Right => {
                self.move_cursor(min(col + 1, self.terminal_size.0), row)?;
                Ok(())
            }
            KeyCode::Up => {
                self.move_cursor(col, if row > 1 { row - 1 } else { row })?;
                Ok(())
            }
            KeyCode::PageUp => {
                self.move_cursor(col, 1)?;
                Ok(())
            }
            KeyCode::PageDown => {
                self.move_cursor(col, row)?;
                Ok(())
            }
            KeyCode::Home => {
                self.move_cursor(1, row)?;
                Ok(())
            }
            KeyCode::End => {
                self.move_cursor(col, row)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        self.move_cursor(0, 0)?;
        self.render()?;
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        stdout().queue(Hide)?;
        self.view.render()?;
        self.move_cursor(0, 0)?;
        stdout().queue(Show)?;
        stdout().flush()?;
        loop {
            if self.should_quit {
                break;
            } else {
                if let Ok(true) = poll(Duration::from_millis(50)) {
                    if let Ok(Event::Key(event)) = read() {
                        self.handle_event(event)?;
                    }
                }
            }
        }
        Ok(())
    }
}
