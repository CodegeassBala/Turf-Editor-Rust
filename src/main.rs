use std::{env, error::Error, io::stdout};

use crossterm::{cursor::Show, terminal::disable_raw_mode, ExecutableCommand};

mod editor;

fn main() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() < 2 {
        println!("Expected file name path...");
        return Ok(());
    }
    let file_name = arguments[1].clone();
    let mut editor = editor::Editor::new(Some(file_name));
    match editor.run() {
        Ok(_res) => {}
        Err(err) => {
            disable_raw_mode()?;
            stdout().execute(Show)?;
            return Err(err);
        }
    }
    Ok(())
}
