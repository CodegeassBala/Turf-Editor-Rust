use std::{
    cmp::{max, min},
    env,
    error::Error,
    fs::File,
    io::{stdout, BufRead, BufReader, Write},
    time::Duration,
};

use crossterm::{
    cursor::{self, Hide, MoveTo, Show},
    event::{self, poll, read, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind},
    style::Print,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        ScrollDown,
    },
    ExecutableCommand, QueueableCommand,
};

struct Buffer {
    lines: Vec<String>,
}

struct TerminalSize {
    height: u16,
    width: u16,
}
struct Position {
    x: u16,
    y: u16,
}

struct View {
    buffer: Buffer,
    file_argument: Option<String>,
    cursor_position: Position,
    row_off: u16,
    terminal_size: TerminalSize,
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
        let (c, r) = terminal::size().unwrap();
        View {
            buffer: Buffer::new(),
            file_argument,
            cursor_position: Position { x: 0, y: 0 },
            row_off: 0,
            terminal_size: TerminalSize {
                height: r,
                width: c,
            },
        }
    }

    pub fn display_line(
        &mut self,
        line: String,
        lines_printed: &mut u16,
    ) -> Result<(), Box<dyn Error>> {
        let mut line_width = line.len() as u16;
        let mut current_pointer = 0 as usize;
        let (c, r) = terminal::size()?;
        while line_width > c - 2 && *lines_printed < r - 1 {
            let end = current_pointer + (c - 2) as usize;
            let sub_string = &line[current_pointer..end];
            stdout().queue(Print(sub_string))?;
            stdout().queue(Print("\r\n"))?;
            line_width -= c - 2;
            *lines_printed += 1;
            current_pointer = end;
        }
        if *lines_printed < r - 1 {
            let sub_string = &line[current_pointer..line.len()];
            stdout().queue(Print(sub_string))?;
            stdout().queue(Print("\r\n"))?;
            *lines_printed += 1;
        }
        Ok(())
    }
    pub fn display_file(&mut self, file_name: String) -> Result<u16, Box<dyn Error>> {
        let current_dir = env::current_dir()?;
        let file_path = current_dir.join(file_name);
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut no_of_lines_printed = 0;
        for line in reader.lines() {
            let content = line?;
            self.buffer.lines.push(content.clone());
            self.display_line(content, &mut no_of_lines_printed)?;
        }
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
    pub fn reload_screen(&mut self) -> Result<(), Box<dyn Error>> {
        let (c_size, r_size) = terminal::size()?;
        self.terminal_size.height = r_size;
        self.terminal_size.width = c_size;
        self.handle_scroll();
        stdout().queue(Hide)?;
        for i in self.row_off..r_size + self.row_off {
            if let Some(line) = self.buffer.lines.get(i as usize) {
                stdout().queue(Print(line))?;
                stdout().queue(Print("\r\n"))?;
            } else {
                stdout().queue(Print("~\r\n"))?;
            }
        }
        stdout().queue(Show)?;
        stdout().flush()?;
        Ok(())
    }

    pub fn handle_scroll(&mut self) {
        if self.cursor_position.y < self.row_off {
            self.row_off = self.cursor_position.y;
        } else if self.cursor_position.y >= self.row_off + self.terminal_size.height {
            self.row_off = self.cursor_position.y - self.terminal_size.height + 1;
        }
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
                self.view.cursor_position.y = min(
                    self.view.buffer.lines.len() as u16,
                    self.view.cursor_position.y + 1,
                );
                self.move_cursor(col, min(self.terminal_size.1, row + 1))?;
                Ok(())
            }
            KeyCode::Left => {
                self.view.cursor_position.x = if self.view.cursor_position.x > 0 {
                    self.view.cursor_position.x - 1
                } else {
                    self.view.cursor_position.x
                };
                self.move_cursor(if col > 1 { col - 1 } else { col }, row)?;
                Ok(())
            }
            KeyCode::Right => {
                self.view.cursor_position.x = min(
                    self.view.terminal_size.width,
                    self.view.cursor_position.x + 1,
                );
                self.move_cursor(min(col + 1, self.terminal_size.0), row)?;
                Ok(())
            }
            KeyCode::Up => {
                self.view.cursor_position.y = if self.view.cursor_position.y > 0 {
                    self.view.cursor_position.y - 1
                } else {
                    self.view.cursor_position.y
                };
                self.move_cursor(col, if row > 0 { row - 1 } else { row })?;
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
            self.view.reload_screen()?;
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
