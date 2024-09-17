use std::{
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
