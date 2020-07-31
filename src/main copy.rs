use std::{
    io,
    io::{stdout, Write}
};
use std::error::Error;
use crossterm::{
    //event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    event::{read, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::terminal::{Terminal};
use tui::backend::{CrosstermBackend};
use tui::widgets::{Block, Borders};
use tui::layout::{Layout, Constraint, Direction, Rect};
use tui::style::{Style, Color};

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new()?;
    app.run()?;
    Ok(())
}

// The cursor can intelligently move through the window as long as the items
// are placed in the correct orientations in the matrix.
pub struct Cursor {
    position: u32,
}
impl Cursor {
    pub fn new() -> Cursor {
        Cursor { position: 0 }
    }

    pub fn move_left(&mut self, window: &Window) {
        let width = window.children.width as u32;
        if (self.position - 1 >= 0) && (width != 0) && (self.position % width > 0) {
            self.position -= 1;
        }
    }

    pub fn move_right(&mut self, window: &Window) {
        let width = window.children.width as u32;
        let len = window.children.data.len() as u32;
        if (self.position + 1 < len) && (width != 0) && (self.position % width < (width-1)) {
            self.position += 1;
        }
    }

    pub fn move_up(&mut self, window: &Window) {
        let width = window.children.width as u32;
        if self.position - width >= 0 {
            self.position -= width;
        }
    }

    pub fn move_down(&mut self, window: &Window) {
        let width = window.children.width as u32;
        let len = window.children.data.len() as u32;
        if self.position + width < len {
            self.position += width;
        }
    }

    pub fn get_pos(&self) -> u32 {
        self.position
    }

    pub fn into_window(&mut self) {
        self.position = 0;
    }
    pub fn exit_window(&mut self, last_window: &Window) {
        self.position = last_window.item;
    }
}

// This structure handles working with terminal for us
pub struct App {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    cursor: Cursor,
    should_quit: bool,
    root_window: Window,
}
impl App {
    pub fn new() -> Result<App, Box<dyn Error>> {
        // setup stdout
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;

        // setup terminal object
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        // give object
        let root_window = Window::generate_tree();
        let mut app = App {
            terminal: terminal,
            cursor: Cursor::new(),
            should_quit: false,
            root_window: root_window,
        };
        
        Ok(app)
    }

    // the main draw loop.
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while !self.should_quit {
            self.draw_screen()?;
            self.wait_for_event();
        }
        Ok(())
    }
    
    pub fn draw_screen(&mut self) -> Result<(), io::Error> {
        // Early resizing allows us to render directly to the frame, then perform
        // an empty draw call which sets up the buffers for the next draw. 
        self.terminal.autoresize()?;     
        self.render_windows(); 

        self.terminal.draw(|_| {})?;
        Ok(())
    }

    // return value describes if app should exit.
    pub fn wait_for_event(&mut self) {
        // if an Event error occurs, just keep polling.
        let event: Event;
        loop {
            match read() {
                Ok(e) => { event = e; break; },
                Err(_) => println!("An error occured"),
            };
        }

        // a list of all the inputs a user can take.
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') => self.exit_current(),
                KeyCode::Esc => self.exit_current(),
                KeyCode::Left => self.cursor.move_left(self.root_window.get_current_window()),
                KeyCode::Up => self.cursor.move_up(self.root_window.get_current_window()),
                KeyCode::Right => self.cursor.move_right(self.root_window.get_current_window()),
                KeyCode::Down => self.cursor.move_down(self.root_window.get_current_window()),
                KeyCode::Enter => self.enter_current(),
                _ => (),
            },
            _ => (),
        };
    }

    pub fn exit_current(&mut self) {
        let current_window = self.root_window.get_current_window();
        self.cursor.exit_window(current_window);
        match self.root_window.get_current_parent(None) {
            None => self.should_quit = true,
            Some(_) => {
                self.root_window.get_current_window().is_active = false; 
            },
        };
    }

    pub fn enter_current(&mut self) {
        let current_window = self.root_window.get_current_window();
        let child_id = self.cursor.get_pos() as usize;
        if current_window.children.data.len() == 0 {
            current_window.activate();
        } else {
            current_window.children.data[child_id].is_active = true;
            self.cursor.into_window();
        }
    }

    /*
    // this function simply renders to the frame directly.
    fn render_windows_direct(&mut self)  {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints( [ Constraint::Percentage(40), Constraint::Percentage(60) ].as_ref() )
            .split(self.terminal.get_frame().size());
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints( [ Constraint::Percentage(60), Constraint::Percentage(40) ].as_ref() )
            .split(chunks[0]);
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints( [ Constraint::Percentage(70), Constraint::Percentage(30) ].as_ref() )
            .split(chunks[1]);

        let mut panes: Vec<(Block, Rect)> = Vec::new();
        let block_select = Block::default().title("Select Puzzle").borders(Borders::ALL);
        panes.push( (block_select, left_chunks[0]) );
        let block_puzzle = Block::default().title("Puzzle Area").borders(Borders::ALL);
        panes.push( (block_puzzle, right_chunks[0]) );
        let block_settings = Block::default().title("Settings").borders(Borders::ALL);
        panes.push( (block_settings, left_chunks[1]) );
        let block_console = Block::default().title("Console").borders(Borders::ALL);
        panes.push( (block_console, right_chunks[1]) );

        // highlight selected title.
        // TODO: do this better somehow?
        if self.is_active {
            panes[self.cursor.get_pos() as usize].0 = panes[self.cursor.get_pos() as usize].0.clone().style( Style::default().fg(Color::Yellow) );
        }
        
        for tup in panes { 
            self.terminal.get_frame().render_widget(tup.0, tup.1); 
        }
    }
    */

    // will recursively render all sub-windows.
    fn render_windows(&mut self) {
        let window_rect = self.terminal.get_frame().size();
        let mut panes: Vec<(Block, Rect)> = self.root_window.render(self, window_rect);
        
        for tup in panes { 
            self.terminal.get_frame().render_widget(tup.0, tup.1);
        }
    }
}
impl Drop for App {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        // clean-up stuff
        disable_raw_mode();
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        self.terminal.show_cursor();
    }
}

//TODO: window class
// Window class is like a single item of a tree of Rectangles. Each rectangle has 
// more rectangles inside of it & optionally attached behaviours.
// The render area of the root node is `terminal.get_frame().size()`
pub struct Window {
    item: u32,
    pub is_active: bool,
    children: SimpleMatrix<Window>,
    activate_function: Option<fn()>,  // does this work?
    render_function: Option<for<'a> fn(&'a Window, &'a App, Rect) -> Vec<(Block<'a>, Rect)>>,
}
impl Window {
    pub fn new( item: u32, 
                children: SimpleMatrix<Window>,  //<'a> 
                activate_function: Option<fn()>, 
                render_function: Option<for<'a> fn(&'a Window, &'a App, Rect) -> Vec<(Block<'a>, Rect)>> ) -> Window {
        Window {
            item,
            is_active: false,
            children,
            activate_function,
            render_function,
        }
    }

    // currently holds the 1# spot for most ghetto code
    pub fn placeholder() -> Window {
        Window::new(
            0, 
            SimpleMatrix::new(0, Vec::new()),
            None,
            None,
        )
    }

    pub fn activate(&self) {
        match self.activate_function {
            Some(f) => f(),
            None => (),
        };
    }

    pub fn render<'a>(&'a self, app: &'a App, render_area: Rect) -> Vec<(Block<'a>, Rect)> {
        match self.render_function {
            Some(f) => f(self, app, render_area),
            None => Vec::new(),
        }
    }

    // #2 ghetto
    // gets the current window under & including the self window
    fn get_current_window(&mut self) -> &mut Window {
        let mut index = 0;
        for child in &self.children.data {
            if child.is_active { // only one child will ever be active, so must be under this subtree.
                return self.children.data[index].get_current_window();
            }
            index += 1;
        }
        self  // base-case: no children are active
    }
    
    /*
    // gets the current window under & including the self window
    fn get_current_window(&mut self) -> &mut Window {
        let ret: &mut Window;
        loop {
            for child in self.children.data.iter_mut() {
                if child.is_active { // only one child will ever be active, so must be under this subtree.
                    return child.get_current_window();
                }
            }
        }
        ret
    }*/

    // gets the parent of the current window under & including the self window.
    // must pass None to parent when calling initially & use root_window.
    fn get_current_parent<'a>(&'a self, parent: Option<&'a Window>) -> Option<&'a Window> {
        for child in &self.children.data {
            if child.is_active { // only one child will ever be active, so must be under this subtree.
                return child.get_current_parent( Some(&self) );
            }
        }
        parent  // base-case: no children are active
    }

    // returns the root node of a tree of tui nodes.
    pub fn generate_tree() -> Window {
        let nav = Window::new(
            0, 
            SimpleMatrix::new(0, Vec::new()),
            None,
            None,
        );

        let puzzle = Window::new(
            1, 
            SimpleMatrix::new(0, Vec::new()),
            None,
            None,
        );

        let settings = Window::new(
            2, 
            SimpleMatrix::new(0, Vec::new()),
            None,
            None,
        );

        let console = Window::new(
            3, 
            SimpleMatrix::new(0, Vec::new()),
            None,
            None,
        );
        
        let mut main = Window::new(
            0, 
            SimpleMatrix::new(2, vec![nav, puzzle, settings, console]),
            None, 
            Some(Window::render_main),
        );

        main.is_active = true;
        main
    }

    pub fn render_main<'a>(window: &'a Window, app: &'a App, rect: Rect) -> Vec<(Block<'a>, Rect)> {
        // partition window.
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints( [ Constraint::Percentage(40), Constraint::Percentage(60) ].as_ref() )
            .split(rect);
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints( [ Constraint::Percentage(60), Constraint::Percentage(40) ].as_ref() )
            .split(chunks[0]);
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints( [ Constraint::Percentage(70), Constraint::Percentage(30) ].as_ref() )
            .split(chunks[1]);

        // create & style renderable blocks
        let mut panes: Vec<(Block, Rect)> = Vec::new();
        let block_select = Block::default().title("Select Puzzle").borders(Borders::ALL);
        panes.push( (block_select, left_chunks[0]) );
        let block_puzzle = Block::default().title("Puzzle Area").borders(Borders::ALL);
        panes.push( (block_puzzle, right_chunks[0]) );
        let block_settings = Block::default().title("Settings").borders(Borders::ALL);
        panes.push( (block_settings, left_chunks[1]) );
        let block_console = Block::default().title("Console").borders(Borders::ALL);
        panes.push( (block_console, right_chunks[1]) );

        // highlight selected title.
        if window.is_active {
            // TODO: do this better somehow?
            let cursor_pos = app.cursor.get_pos() as usize;
            panes[cursor_pos].0 = panes[cursor_pos].0.clone().style( Style::default().fg(Color::Yellow) );
        }
        // render to terminal & recursively render sub trees.
        let mut i = 0;
        for tup in panes.clone() { 
            //app.terminal.get_frame().render_widget(tup.0, tup.1);
            let mut vec = window.children.data[i].render(app, tup.1);
            panes.append(&mut vec);
            i += 1;
        }
        panes
    }
}

pub struct SimpleMatrix<T> {
    pub width: usize,
    pub data: Vec<T>,
} 
impl<T> SimpleMatrix<T> {
    pub fn new(width: usize, data: Vec<T>) -> SimpleMatrix<T> {
        SimpleMatrix {
            width,
            data,
        } 
    }
    /*
    pub fn get(&self, x: usize, y: usize) -> T {
       self.data[y * self.width + x]
    }
    pub fn set(&self, x: usize, y: usize, val: T) {  // TODO: needs mut
        self.data[y * self.width + x] = val;
    }
    */
}

// TODO: 
//       allow the user to look at all the files in the puzzles directory.
//       