use std::{
    io,
    io::{stdout, Write},
    error::Error,
};
use crossterm::{
    event::{read, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};

use tui::terminal::{Terminal, Frame};
use tui::backend::CrosstermBackend;
use tui::layout::Rect;

mod windows;
use windows::Window;

use crate::util::{write_log, setup_panic_hook, SafeTermWrapper};

// ************************************************************************** //
// App controller class

// This structure handles working with terminal for us
pub struct App {
    should_quit: bool,
    windows: Vec<Box<dyn Window>>,  // The order of this vector is important --> consider z-sorting.
    root_window: usize,
    current_window: usize,
}
impl App {
    pub fn new() -> App {
        let mut app = App {
            should_quit: false,
            windows: App::generate_window_tree(),
            root_window: 0,
            current_window: 0,
        };
        match app.windows[app.root_window].enter() {
            Some(val) => app.windows[val].select(true),
            None => (),
        };
        app
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

    fn get_current_window(&mut self) -> &mut Box<dyn Window> { // impl Window
        let index = self.current_window;
        &mut self.windows[index]
    }

    
    fn get_root_window_mut(&mut self) -> &mut Box<dyn Window> {
        let index = self.root_window;
        &mut self.windows[index]
    }
    /*fn get_root_window(&self) -> &Box<dyn Window> {
        let index = self.root_window;
        &self.windows[index]
    }*/

    // the main draw loop.
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // it's important that the terminal is not a property of App.
        // Also, wrapper is for safe drop (otherwise terminal will break on panic)
        let mut terminal = SafeTermWrapper( App::create_terminal()? );

        while !self.should_quit {
            terminal.0.draw(
                |f| self.draw( f )
            )?;
            self.wait_for_event();
        }
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>) {
        // pre-draw stage
        self.get_root_window_mut().update_rect(f.size());
        let mut rect_updates: Vec<(usize, Rect)> = Vec::new();
        for window in &self.windows {
            rect_updates.append(&mut window.size_update());
        }

        // update all rect sizes
        for (i, rect) in rect_updates.drain(..) {  //TODO: make sure this is a move
            self.windows[i].update_rect(rect);
        }

        // actually draw
        for window in &mut self.windows {
            window.draw(f);
        }
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
                KeyCode::Char('q') => self.exit(),
                KeyCode::Esc => self.exit(),
                KeyCode::Enter => self.enter(),
                KeyCode::Left => {
                    let (old, new) = self.get_current_window().move_left();
                    self.windows[old].select(false);
                    self.windows[new].select(true);
                },
                KeyCode::Up => {
                    let (old, new) = self.get_current_window().move_up();
                    self.windows[old].select(false);
                    self.windows[new].select(true);
                },
                KeyCode::Right => {
                    let (old, new) = self.get_current_window().move_right();
                    self.windows[old].select(false);
                    self.windows[new].select(true);
                },
                KeyCode::Down => {
                    let (old, new) = self.get_current_window().move_down();
                    self.windows[old].select(false);
                    self.windows[new].select(true);
                },
                _ => (),
            },
            _ => (),
        };
    }

    fn exit(&mut self) {
        self.windows[self.current_window].exited();
        match self.get_current_window().exit() {
            Some(val) => {
                self.windows[val].select(false);
                self.current_window = val;
                match self.get_current_window().enter() {
                    Some(val) => self.windows[val].select(true),
                    None =>(),
                }; 
            },
            None => self.should_quit = true,
        };
        self.windows[self.current_window].entered();
    }

    fn enter(&mut self) {
        match self.get_current_window().enter() {
            Some(val) => {
                self.windows[self.current_window].exited();
                self.windows[val].select(false);
                self.current_window = val;
                match self.get_current_window().enter() {
                    Some(val) => self.windows[val].select(true),
                    None =>(),
                }; 
                self.windows[self.current_window].entered();
            }
            // case: the current window doesn't change & gets to 
            // decide what it wants to do.
            None => (), 
        };
    }

    // returns the root node of a tree of tui nodes.
    // tree edges must be manually constructed.
    pub fn generate_window_tree() -> Vec<Box<dyn Window>> {
        let mut select = Box::new(windows::SelectWindow {
            index: 1,
            is_selected: false,
            parent: 0,
            state: tui::widgets::ListState::default(),
            items: Vec::new(),
            rect: Rect::default(),
        });
        select.init();

        let main = Box::new(windows::MainWindow {
            //index: 0,
            is_selected: true,
            selected_child: 0,
            children: vec![1],
            rect: Rect::default(),
        });

        vec![main, select]
    }
}