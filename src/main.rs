use std::{
    io,
    io::{stdout, Write},
    fs::{File, OpenOptions},
    panic,
    error::Error,
    sync::atomic::{AtomicU32, Ordering},
};
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
    setup_panic_hook();
    let mut app = App::new()?;
    app.run()?;
    Ok(())
}

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
        //disable_raw_mode();
        //execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        //self.terminal.show_cursor();
        //println!("{:?}", info);
        write_log(&format!("{:?}", info));
    }));
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
        if (self.position >= 1) && (width != 0) && (self.position % width > 0) {
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
        if self.position >= width {
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
            write_log("bef draw\n");
            self.draw_screen()?;
            write_log("after draw\n");
            self.wait_for_event();
        }
        Ok(())
    }
    
    pub fn draw_screen(&mut self) -> Result<(), io::Error> {
        // Early resizing allows us to render directly to the frame, then perform
        // an empty draw call which sets up the buffers for the next draw. 
        self.terminal.autoresize()?;     
        self.render_windows(); 
        write_log("after window render\n");

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
                Err(_) => write_log("an ERROR occured\n"),
            };
        }

        write_log("check events\n");
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
        write_log("event done\n");
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
        write_log("entering\n");
        if current_window.children.data.len() == 0 {
            current_window.activate();
        } else {
            current_window.children.data[child_id].is_active = true;
            self.cursor.into_window();
        }
        write_log("entering2\n");
    }

    // Will recursively render all sub-windows.
    //
    // Since we know the root is always 'not-rendered' between calls, we can take the first non-rendered window and
    // infer that its children are not-rendered as well. In this way, by reseting the root window to 'rendered = false'
    // after every time the window is rendered, we can infer which window is currently being rendered in the draw func
    // by using the `current_render_window()` function. -> otherwise we would have to pass a reference to window with a
    // mutable reference to app at the same time, which is not allowed >:c 
    fn render_windows(&mut self) {
        let window_rect = self.terminal.get_frame().size();
        let func = self.root_window.render_function;
        Window::render(func, self, window_rect);
        self.root_window.rendered = false;
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

// for generating unique ids.
static COUNTER: AtomicU32 = AtomicU32::new(0);

//TODO: window class
// Window class is like a single item of a tree of Rectangles. Each rectangle has 
// more rectangles inside of it & optionally attached behaviours.
// The render area of the root node is `terminal.get_frame().size()`
pub struct Window {
    id: u32,
    item: u32,
    pub is_active: bool,
    pub rendered: bool,
    children: SimpleMatrix<Window>,
    activate_function: Option<fn()>,  // does this work?
    render_function: Option<fn(&mut App, Rect)>,
}
impl Window {
    pub fn new( item: u32, 
                children: SimpleMatrix<Window>,  //<'a> 
                activate_function: Option<fn()>, 
                render_function: Option<fn(&mut App, Rect)> ) -> Window {
        Window {
            id: Window::new_id(),
            item,
            is_active: false,
            rendered: false,
            children,
            activate_function,
            render_function,
        }
    }

    // for generating unique ids.
    fn new_id() -> u32 { COUNTER.fetch_add(1, Ordering::Relaxed) }

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

    // this function is called in order to render every window.
    pub fn render(func: Option<fn(&mut App, Rect)>, app: &mut App, render_area: Rect)  {
        for child in &mut app.root_window.current_render_window().children.data {
            child.rendered = false;
        }

        match func {
            Some(f) => f(app, render_area),
            None => (),
        };

        app.root_window.current_render_window().rendered = true;
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

    // returns the window currently being rendered
    fn current_render_window(&mut self) -> &mut Window {
        let mut index = 0;
        for child in &self.children.data {
            if child.rendered {
                return self.children.data[index].current_render_window();
            }
            index += 1;
        }
        self  // base-case
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

    pub fn render_main(app: &mut App, rect: Rect) {
        write_log("doing main render\n");
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
        let selected_window_id = app.root_window.get_current_window().id;
        let window = app.root_window.current_render_window();
        if window.is_active && window.id == selected_window_id {
            // TODO: do this better somehow?
            let cursor_pos = app.cursor.get_pos() as usize;
            panes[cursor_pos].0 = panes[cursor_pos].0.clone().style( Style::default().fg(Color::Yellow) );
        }

        // render to terminal & recursively render sub trees.
        let mut i = 0;
        for tup in panes.clone() { 
            //app.terminal.get_frame().render_widget(tup.0, tup.1);
            let func = app.root_window.current_render_window().children.data[i].render_function;
            Window::render(func, app, tup.1);
            //panes.append(&mut vec);
            i += 1;
        }

        // TODO: fix this, but it does technically work.
        for tup in panes { 
            app.terminal.get_frame().render_widget(tup.0, tup.1);
        }
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