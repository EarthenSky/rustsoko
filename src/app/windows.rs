use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

use std::cmp::Ordering;

use tui::terminal::Frame;
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders, List, ListState, ListItem};
use tui::layout::{Layout, Constraint, Direction, Rect};
use tui::style::{Style, Color};

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
    pub is_selected: bool,  // TODO: remove this because it has no use
    pub selected_child: usize,
    pub children: Vec<usize>,
    pub rect: Rect,
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
    pub index: usize,
    pub is_selected: bool,
    pub parent: usize,
    pub state: ListState,
    pub items: Vec<String>,
    pub rect: Rect,
    //selected_child: usize,
    //children: Vec<usize>,
}
impl SelectWindow {
    // TODO: add more information to this / make it reoccurring / add an update button.
    pub fn init(&mut self) {
        // TODO: this line -> self.items.clear();
        let path = Path::new("./puzzles/");
        let path = match fs::read_dir(path) {
            Ok(dir_read) => {
                self.items.push( path.to_string_lossy().into_owned() );
                for item in dir_read {
                    let item_string = match item {
                        Ok(file_item) => file_item.path().to_string_lossy().into_owned(),
                        Err(_) => "(file reading error)".to_string(),
                    };
                    self.items.push(item_string);
                }
                None
            },
            Err(_) => Some(Path::new("../../puzzles/")), // try debug path next
        };
        // check secondary path
        let path = match path {
            Some(path) => match fs::read_dir(path) {
                Ok(dir_read) => {
                    self.items.push( path.to_string_lossy().into_owned() );
                    for item in dir_read {
                        let item_string = match item {
                            Ok(file_item) => file_item.path().to_string_lossy().into_owned(),
                            Err(_) => "(file reading error)".to_string(),
                        };
                        self.items.push(item_string);
                    }
                    None
                },
                Err(_) => Some(()),
            },
            None => None,
        };
        // both dirs empty
        match path {
            Some(_) => self.items.push("could not find any files at ./puzzles/ or ../../puzzles/".to_string()),
            None => (),
        };
        self.items.sort_by(|a, b| 
            if a.len() > b.len() {
                Ordering::Greater
            } else if a.len() < b.len(){
                Ordering::Less
            } else {
                Ordering::Equal
            }
        );
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
        // TODO: if a state is ever 'entered,' then don't unselect it.
        self.state.select( None );
    } 
    
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
