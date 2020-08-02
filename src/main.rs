use std::{
    io,
    io::{stdout, Write},
    fs::{File, OpenOptions},
    panic,
    error::Error,
    //sync::atomic::{AtomicU32, Ordering},
};
use crossterm::{
    //event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    event::{read, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::terminal::{Terminal, Frame};
use tui::backend::{CrosstermBackend};
use tui::widgets::{Block, Borders, Widget, StatefulWidget};
use tui::layout::{Layout, Constraint, Direction, Rect};
use tui::style::{Style, Color};

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    app.run()?;
    Ok(())
}

// ************************************************************************** //
// debug:

// for testing purposes.
fn write_log(s: &str) {
    let mut file = match OpenOptions::new().append(true).open("log.txt") {
        Ok(f) => f,
        Err(_) => File::create("log.txt").expect("logger function failed create D:"),
    };
    file.write_all(s.as_bytes()).expect("logger function failed write D:");
}

// debug messages go to log file.
fn setup_panic_hook() {
    panic::set_hook(Box::new(|info| {
        write_log(&format!("{:?}", info));
    }));
}

// ************************************************************************** //
// app controller class

// This structure handles working with terminal for us
pub struct App {
    should_quit: bool,
    windows: Vec<Box<dyn Window>>,
    root_window: usize,
    current_window: usize,
}

impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            windows: App::generate_window_tree(),
            root_window: 0,
            current_window: 0,
        }
    }
    
    fn create_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn Error>> {
        setup_panic_hook();

        // setup stdout
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;

        // setup terminal object
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(terminal)
    }

    // TODO: change this from borrowing to giving ownership.
    #[allow(unused_must_use)]
    fn drop_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        // clean-up stuff
        disable_raw_mode();
        execute!(terminal.backend_mut(), LeaveAlternateScreen);
        terminal.show_cursor();
    }

    fn get_current_window(&mut self) -> &mut Box<dyn Window> { // impl Window
        let index = self.current_window;
        &mut self.windows[index]
    }

    fn get_root_window(&mut self) -> &mut Box<dyn Window> {
        let index = self.root_window;
        &mut self.windows[index]
    }

    // the main draw loop.
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // it's important that the terminal is not a property of App.
        let mut terminal = App::create_terminal()?;

        while !self.should_quit {
            terminal.draw(
                |f| self.draw( f, f.size() )
            )?;
            self.wait_for_event();
        }

        App::drop_terminal(&mut terminal);
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>, rect: Rect) {
        self.get_root_window().draw(f, rect);
    }

    fn wait_for_event(&mut self) {
        // if an Event error occurs, just keep polling.
        let event: Event;
        loop {
            match read() {
                Ok(e) => { event = e; break; },
                Err(_) => write_log("ERROR: an event could not be read.\n"),
            };
        }

        // a list of all the inputs a user can take.
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Esc => self.current_window = self.get_current_window().exit(),
                KeyCode::Left => self.get_current_window().move_left(),
                KeyCode::Up => self.get_current_window().move_up(),
                KeyCode::Right => self.get_current_window().move_right(),
                KeyCode::Down => self.get_current_window().move_down(),
                KeyCode::Enter => self.current_window = self.get_current_window().enter(),
                _ => (),
            },
            _ => (),
        };
    }

    // returns the root node of a tree of tui nodes.
    pub fn generate_window_tree() -> Vec<Box<dyn Window>> {
        /*let select = SelectWindow {
            index: 1,
            children: Vec::new(),
        };

        let main = MainWindow {
            index: 0,
            children: vec![select],
        };

        vec![main, select];*/
        Vec::new()
    }
}

// ************************************************************************** //
// Window stuff

// TODO: get this working w/ rendering.
pub trait Window {
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>, rect: Rect);
    //fn select(&mut self, is_selected: bool);
    fn enter(&mut self) -> usize;  // the return value is the new current window.
    fn exit(&mut self) -> usize;  // ''
    fn move_left(&mut self);
    fn move_right(&mut self);
    fn move_up(&mut self);
    fn move_down(&mut self);
}

/*
pub struct MainWindow {
    index: usize,
    children: Vec<usize>,
}
impl Window for MainWindow {

}

pub struct SelectWindow {
    index: usize,
    children: Vec<usize>,
}
impl Window for SelectWindow {

}
*/