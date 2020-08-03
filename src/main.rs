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
use tui::widgets::{Block, Borders, Widget, StatefulWidget, List, ListState, ListItem};
use tui::layout::{Layout, Constraint, Direction, Rect};
use tui::style::{Style, Color, Modifier};

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
        write_log(&format!("{:?}\n", info));
    }));
}

// this allows the terminal window to be switched back on panics.
struct SafeTermWrapper(Terminal<CrosstermBackend<io::Stdout>>);
impl Drop for SafeTermWrapper {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        // for terminal niceness.
        disable_raw_mode(); 
        execute!(self.0.backend_mut(), LeaveAlternateScreen);
    }
}

// ************************************************************************** //
// app controller class

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
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Esc => {
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
                },
                KeyCode::Enter => {
                    self.windows[self.current_window].exited();
                    match self.get_current_window().enter() {
                        Some(val) => {
                            self.windows[val].select(false);
                            self.current_window = val;
                            match self.get_current_window().enter() {
                                Some(val) => self.windows[val].select(true),
                                None =>(),
                            }; 
                        }
                        // case: the current window doesn't change & gets to 
                        // decide what it wants to do.
                        None => (), 
                    };
                    self.windows[self.current_window].entered();
                },
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

    // returns the root node of a tree of tui nodes.
    // tree edges must be manually constructed.
    pub fn generate_window_tree() -> Vec<Box<dyn Window>> {
        let mut select = Box::new(SelectWindow {
            index: 1,
            is_selected: false,
            parent: 0,
            state: ListState::default(),
            items: Vec::new(),
            rect: Rect::default(),
        });
        select.init();

        let main = Box::new(MainWindow {
            //index: 0,
            is_selected: true,
            selected_child: 0,
            children: vec![1],
            rect: Rect::default(),
        });

        vec![main, select]
    }
}

// ************************************************************************** //
// Window stuff

pub trait Window {
    // it is NOT the draw function's responsibility to draw its children.
    // No state changes occur during the draw function.
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>);
    // However, in the size update call, it is each function's responsibility to
    // update the size of their children. This is like a pre-draw function.
    fn size_update(&self) -> Vec<(usize, Rect)>;
    fn update_rect(&mut self, rect: Rect);

    fn select(&mut self, is_selected: bool); // controls the variable on a window
    // The return value is the new current window. If this is None, then the
    // function can do anything it wants, perhaps functioning like a button press.
    fn enter(&mut self) -> Option<usize>;
    fn exit(&mut self) -> Option<usize>; 
    fn entered(&mut self);
    fn exited(&mut self); 
    // returns id of old window to deselect, and new window to select. -> (old, new)
    fn move_left(&mut self) -> (usize, usize);
    fn move_right(&mut self) -> (usize, usize);
    fn move_up(&mut self) -> (usize, usize);
    fn move_down(&mut self) -> (usize, usize);
}

pub struct MainWindow {
    //index: usize,
    is_selected: bool,  // TODO: remove this because it has no use
    selected_child: usize,
    children: Vec<usize>,
    rect: Rect,
}
impl Window for MainWindow {
    #[allow(unused_variables)]
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>) {
        // do nothing?
    }
    fn size_update(&self) -> Vec<(usize, Rect)> {
        // partition window.
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints( [ Constraint::Percentage(40), Constraint::Percentage(60) ].as_ref() )
            .split(self.rect);  // root node.rect always == f.size()
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints( [ Constraint::Percentage(60), Constraint::Percentage(40) ].as_ref() )
            .split(chunks[0]);
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints( [ Constraint::Percentage(70), Constraint::Percentage(30) ].as_ref() )
            .split(chunks[1]);

        vec![ (1, left_chunks[0]) ]
    }
    fn update_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }
    
    fn select(&mut self, is_selected: bool) {
        self.is_selected = is_selected;
    }
    fn enter(&mut self) -> Option<usize> {
        Some(self.children[self.selected_child])
    }
    fn exit(&mut self) -> Option<usize> {
        None // this means to exit
    }
    fn entered(&mut self) { }
    fn exited(&mut self) { }
    
    fn move_left(&mut self) -> (usize, usize) { 
        let child = self.children[self.selected_child];
        (child, child)
    }
    fn move_right(&mut self) -> (usize, usize) { 
        let child = self.children[self.selected_child];
        (child, child)
    }
    fn move_up(&mut self) -> (usize, usize) { 
        let child = self.children[self.selected_child];
        (child, child)
    }
    fn move_down(&mut self) -> (usize, usize) { 
        let child = self.children[self.selected_child];
        (child, child)
    }
}

const WINDOW_DO_NOTHING: (usize, usize) = (0, 0);
pub struct SelectWindow {
    index: usize,
    is_selected: bool,
    parent: usize,
    state: ListState,
    items: Vec<String>,
    rect: Rect,
    //selected_child: usize,
    //children: Vec<usize>,
}
impl SelectWindow {
    // TODO: implement this to read from default directories.
    fn init(&mut self) {
        self.items.append( &mut vec!["./puzzles/".to_string(), "file 1".to_string(), "file 2".to_string(), "dir 1".to_string()] );
    }
}
impl Window for SelectWindow {
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>) {
        // TODO: set aside one level for a paragraph. -> this paragraph will be the directory name.
        
        let mut block_select = Block::default().title("Select Puzzle").borders(Borders::ALL);
        if self.is_selected {
            block_select = block_select.style( Style::default().fg(Color::Yellow) );
        }

        let items: Vec<ListItem> = self.items.iter().map(|item| 
            ListItem::new(item.as_ref())
        ).collect();
        let select_list = List::new(items)
            .block(block_select)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol("> ");

        f.render_stateful_widget(select_list, self.rect, &mut self.state);
    }
    fn size_update(&self) -> Vec<(usize, Rect)> {
        Vec::new()
    }
    fn update_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }
    
    fn select(&mut self, is_selected: bool) {
        self.is_selected = is_selected;
    }
    fn enter(&mut self) -> Option<usize> {
        // TODO: 'select' the current puzzle when doing this option.
        None
    }
    fn exit(&mut self) -> Option<usize> {
        Some(self.parent)
    }
    fn entered(&mut self) {
        self.state.select( Some(0) );
    }
    fn exited(&mut self) {
        self.state.select( None );
    } 
    
    // TODO: highlight children.
    fn move_left(&mut self) -> (usize, usize) { WINDOW_DO_NOTHING }
    fn move_right(&mut self) -> (usize, usize) { WINDOW_DO_NOTHING }
    fn move_up(&mut self) -> (usize, usize) { 
        let item = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            },
            None => 0,
        };
        self.state.select( Some(item) );
        (self.index, 0)
    }
    fn move_down(&mut self) -> (usize, usize) { 
        let item = match self.state.selected() {
            Some(i) => {
                if (i + 1) == self.items.len() {
                    self.items.len() - 1
                } else {
                    i + 1
                }
            },
            None => 0,
        };
        self.state.select( Some(item) );
        (self.index, 0)
    }
}
