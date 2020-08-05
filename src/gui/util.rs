use std::{
    io,
    io::Write,
    fs::{File, OpenOptions},
    panic,
};

use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

use tui::terminal::Terminal;
use tui::backend::CrosstermBackend;

// ************************************************************************** //
// util & debug code:

// for testing purposes.
pub fn write_log(s: &str) {
    let mut file = match OpenOptions::new().append(true).open("log.txt") {
        Ok(f) => f,
        Err(_) => File::create("log.txt").expect("logger function failed create D:"),
    };
    file.write_all(s.as_bytes()).expect("logger function failed write D:");
}

// debug messages go to log file.
pub fn setup_panic_hook() {
    panic::set_hook(Box::new(|info| {
        write_log(&format!("{:?}\n", info));
    }));
}

// this allows the terminal window to be switched back on panics.
pub struct SafeTermWrapper(pub Terminal<CrosstermBackend<io::Stdout>>);
impl Drop for SafeTermWrapper {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        // for terminal niceness.
        disable_raw_mode(); 
        execute!(self.0.backend_mut(), LeaveAlternateScreen);
    }
}