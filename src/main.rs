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

    // draw loop
    let mut should_quit = false;
    while !should_quit {
        app.draw_screen()?;
        should_quit = app.wait_for_event();
    }
    
    // todo: go up the entire screen length after done
    Ok(())
}

pub struct Cursor {
    position: u8,
}
impl Cursor {
    pub fn new(position: u8) -> Cursor {
        // cursor position is in [0..3]
        if position > 3 {
            panic!("cursor position must be in [0..3]");
        }
        Cursor { position }
    }

    pub fn move_left(&mut self) {
        match self.position {
            0 => (),
            2 => (),
            _ => { self.position -= 1; },
        };
    }

    pub fn move_right(&mut self) {
        match self.position {
            1 => (),
            3 => (),
            _ => { self.position += 1; },
        };
    }

    pub fn move_up(&mut self) {
        match self.position {
            0 => (),
            1 => (),
            _ => { self.position -= 2; },
        };
    }

    pub fn move_down(&mut self) {
        match self.position {
            2 => (),
            3 => (),
            _ => { self.position += 2; },
        };
    }

    pub fn get_pos(&self) -> u8 {
        self.position
    }
}

// This structure handles working with terminal for us
pub struct App {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    cursor: Cursor,
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
        Ok(App {
            terminal: terminal,
            cursor: Cursor::new(0),
        })
    }
    
    pub fn draw_screen(&mut self) -> Result<(), io::Error> {
        // Early resizing allows us to render directly to the frame, then perform
        // an empty draw call which sets up the buffers for the next draw. 
        self.terminal.autoresize()?;     
        self.render_windows_direct(); 

        self.terminal.draw(|_| {})?;
        Ok(())
    }

    // return value describes if app should exit.
    pub fn wait_for_event(&mut self) -> bool {
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
                KeyCode::Char('q') => return true,
                KeyCode::Esc => return true,
                KeyCode::Left => self.cursor.move_left(),
                KeyCode::Up => self.cursor.move_up(),
                KeyCode::Right => self.cursor.move_right(),
                KeyCode::Down => self.cursor.move_down(),
                KeyCode::Enter => self.enter_current(),
                _ => (),
            },
            _ => (),
        };
        false
    }

    pub fn enter_current(&self) {

    }

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
        panes[self.cursor.get_pos() as usize].0 = panes[self.cursor.get_pos() as usize].0.clone().style( Style::default().fg(Color::Yellow) );
        
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

// TODO: 
//       allow the user to look at all the files in the puzzles directory.
//       