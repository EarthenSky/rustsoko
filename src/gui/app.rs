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
use windows::{Window, MainWindow, SelectWindow, PuzzleWindow};

use crate::util::{write_log, setup_panic_hook, SafeTermWrapper};

// ************************************************************************** //
// App controller class

// This structure handles working with terminal for us
pub struct App {
    should_quit: bool,
    //windows: Vec<Box<dyn Window>>,  // The order of this vector is important --> consider z-sorting.
    root_window: usize,
    current_window: usize,
}
impl App {
    pub fn new() -> App {
        App {
            should_quit: false,
            //windows: windows,
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

    // the main draw loop.
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let (mut main_window, mut select_window, mut puzzle_window) = App::generate_windows();
        select_window.select(true); // first child of main_window.

        // It's important that the terminal is not a property of App.
        // Also, wrapper is for safe drop (otherwise terminal will break on panic)
        let mut terminal = SafeTermWrapper( App::create_terminal()? );

        while !self.should_quit {
            {
                // create identity references and draw them using Window trait.
                let mut windows: Vec<&mut dyn Window> = Vec::new();
                windows.push(&mut main_window as &mut dyn Window); 
                windows.push(&mut select_window as &mut dyn Window);
                windows.push(&mut puzzle_window as &mut dyn Window);

                terminal.0.draw(
                    |f| self.draw( f, &mut windows )
                )?;
                self.wait_for_event(&mut windows);
            }
            
            // In this function, window specific interactions can occur because 
            // mutable references don't exist to any of the children.
            self.event_reactions(&mut main_window, &mut select_window, &mut puzzle_window);
        }
        Ok(())
    }

    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>, windows: &mut Vec<&mut dyn Window>) {
        // pre-draw stage
        windows[self.root_window].update_rect(f.size());
        let mut rect_updates: Vec<(usize, Rect)> = Vec::new();
        for window in windows.iter_mut() {
            rect_updates.append(&mut window.size_update());
        }

        // update all rect sizes
        for (i, rect) in rect_updates.drain(..) {  //TODO: make sure this is a move
            windows[i].update_rect(rect);
        }

        // actually draw
        for window in windows.iter_mut() {
            window.draw(f);
        }
    }

    fn wait_for_event(&mut self, windows: &mut Vec<&mut dyn Window>) {
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
                KeyCode::Char('q') => self.exit(windows),
                KeyCode::Esc => self.exit(windows),
                KeyCode::Enter => self.enter(windows),
                KeyCode::Left => {
                    let (old, new) = windows[self.current_window].move_left();
                    windows[old].select(false);
                    windows[new].select(true);
                },
                KeyCode::Up => {
                    let (old, new) = windows[self.current_window].move_up();
                    windows[old].select(false);
                    windows[new].select(true);
                },
                KeyCode::Right => {
                    let (old, new) = windows[self.current_window].move_right();
                    windows[old].select(false);
                    windows[new].select(true);
                },
                KeyCode::Down => {
                    let (old, new) = windows[self.current_window].move_down();
                    windows[old].select(false);
                    windows[new].select(true);
                },
                _ => (),
            },
            _ => (),
        };
    }

    fn event_reactions(&mut self,
        main_window: &mut MainWindow, 
        select_window: &mut SelectWindow, 
        puzzle_window: &mut PuzzleWindow) {
        // if select_window.has_selected {
        //      puzzle_window.puzzle = select_window.get_puzzle();   
        // }
    }

    fn exit(&mut self, windows: &mut Vec<&mut dyn Window>) {
        windows[self.current_window].exited();
        match windows[self.current_window].exit() {
            Some(val) => {
                windows[val].select(false);
                self.current_window = val;
                match windows[self.current_window].enter() {
                    Some(val) => windows[val].select(true),
                    None =>(),
                }; 
            },
            None => self.should_quit = true,
        };
        windows[self.current_window].entered();
    }

    fn enter(&mut self, windows: &mut Vec<&mut dyn Window>) {
        match windows[self.current_window].enter() {
            Some(val) => {
                windows[self.current_window].exited();
                windows[val].select(false);
                self.current_window = val;
                match windows[self.current_window].enter() {
                    Some(val) => windows[val].select(true),
                    None =>(),
                }; 
                windows[self.current_window].entered();
            }
            // case: the current window doesn't change & gets to 
            // decide what it wants to do.
            None => (), 
        };
    }

    // returns the root node of a tree of tui nodes.
    // tree edges must be manually constructed.
    // TODO: remeber to update all of these!!
    pub fn generate_windows() -> (MainWindow, SelectWindow, PuzzleWindow) {
        let mut select = SelectWindow {
            index: 1,
            is_selected: false,
            is_active: false,
            parent: 0,
            state: tui::widgets::ListState::default(),
            items: Vec::new(),
            selected_item: None,
            old_item_name: None,
            dir_name: "".to_string(),
            rect: Rect::default(),
        };
        select.load_directory();

        let puzzle = PuzzleWindow {
            index: 2,
            is_selected: false,
            parent: 0,
            rect: Rect::default(),
        };

        let main = MainWindow {
            //index: 0,
            is_selected: true,
            selected_child: 0,
            children: vec![1, 2],
            rect: Rect::default(),
        };

        (main, select, puzzle)
    }
}