use std::io;
use std::fs::{self, DirEntry};
use std::path::Path;

use std::cmp::Ordering;

use tui::terminal::Frame;
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders, List, ListState, ListItem, Paragraph};
use tui::layout::{Layout, Constraint, Direction, Rect, Alignment};
use tui::style::{Style, Color};
use tui::text::Span;

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

        vec![ (1, left_chunks[0]), (2, right_chunks[0]) ]
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
        let old = self.children[self.selected_child];
        if self.selected_child == 1 {
            self.selected_child = 0;
        }
        let new = self.children[self.selected_child];
        (old, new)
    }
    fn move_right(&mut self) -> (usize, usize) { 
        let old = self.children[self.selected_child];
        if self.selected_child == 0 {
            self.selected_child = 1;
        }
        let new = self.children[self.selected_child];
        (old, new)
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
    pub is_active: bool, // represents if the user is in this window.
    pub parent: usize,
    pub state: ListState,
    pub items: Vec<String>,
    pub selected_item: Option<usize>,
    pub old_item_name: Option<String>,
    pub dir_name: String,
    pub rect: Rect,
    //selected_child: usize,
    //children: Vec<usize>,
}
impl SelectWindow {
    // TODO: add more information to this / make it reoccurring / add an update button.
    pub fn load_directory(&mut self) {
        // TODO: this line -> self.items.clear();
        let path = Path::new("./puzzles/");
        let path = match fs::read_dir(path) {
            Ok(dir_read) => {
                self.dir_name = path.to_string_lossy().into_owned();
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
                    self.dir_name = path.to_string_lossy().into_owned();
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
            Some(_) => self.dir_name = "could not find any files at ./puzzles/ or ../../puzzles/".to_string(),
            None => (),
        };
        // sort by path length
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
    
    // TODO: this
    pub fn get_loaded_puzzle() -> String {
        "".to_string()
    }
}
impl Window for SelectWindow {
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>) {
        let separator = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints( [ Constraint::Length(3), Constraint::Min(0) ].as_ref() )
            .split(self.rect);

        // the block with the bar & directory name
        let mut select_bar = Block::default().title("Select Puzzle").borders(Borders::ALL);
        if self.is_selected {
            select_bar = select_bar.style( Style::default().fg(Color::Yellow) );
        }
        let select_bar_text = Paragraph::new( Span::raw(format!("puzzle directory = {}", &self.dir_name)) )
            .block(select_bar)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left);
        f.render_widget(select_bar_text, separator[0]);

        // the body which holds the list.
        let mut select_block = Block::default().borders(Borders::ALL);
        if self.is_selected {
            select_block = select_block.style( Style::default().fg(Color::Yellow) );
        }
        let items: Vec<ListItem> = match self.selected_item {
            Some(item_val) => {
                self.items.iter().enumerate().map(|(i, item)|
                    if i == item_val {
                        ListItem::new( item.as_ref() )
                            .style(Style::default().fg(Color::LightGreen))
                    } else {
                        ListItem::new(item.as_ref())
                    }
                ).collect()
            },
            None => {
                self.items.iter().map(|item|
                    ListItem::new(item.as_ref())
                ).collect()
            },
        };
        let mut select_list = List::new(items)
            .block(select_block)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol("> ");

        if !self.is_active {
            select_list = select_list.highlight_style( Style::default().fg(Color::LightGreen) );
        }

        f.render_stateful_widget(select_list, separator[1], &mut self.state);
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
    // 'selects' the current puzzle when doing this option.
    fn enter(&mut self) -> Option<usize> {
        match &self.old_item_name {
            Some(s) => match self.selected_item {
                Some(old_item) => self.items[old_item] = s.clone(),
                None => (), // this will never get here
            },
            None => (),
        };
        self.selected_item = self.state.selected().clone();  // update select id
        match self.selected_item {
            Some(new_item) => {
                self.old_item_name = Some(self.items[new_item].clone());  // save newly selected item string
                self.items[new_item].push_str(" (selected)");  // update new string
            },
            None => {
                self.old_item_name = None;
            },
        };
        None
    }
    fn exit(&mut self) -> Option<usize> {
        Some(self.parent)
    }
    fn entered(&mut self) {
        self.is_active = true;
        self.state.select( self.selected_item.or(Some(0)) );
    }
    fn exited(&mut self) {
        self.is_active = false;
        self.state.select( self.selected_item.clone() );
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

// TODO: may need to implement a pre-draw function which gets called so that this class
// can be edited by the SelectWindow class --> or at least so that these classes can communicate the puzzle string.
pub struct PuzzleWindow {
    pub index: usize,
    pub is_selected: bool,
    pub parent: usize,
    pub rect: Rect,
    //pub puzzle: String,  // stores the puzzle string which can be scaled when drawing.
    //selected_child: usize,
    //children: Vec<usize>,
}
impl PuzzleWindow {
    pub fn _init(&mut self) {
        //TODO: do or remove this.
    }
}
impl Window for PuzzleWindow {
    // todo: do this.
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<io::Stdout>>) {
        let separator = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints( [ Constraint::Min(0), Constraint::Length(3) ].as_ref() )
            .split(self.rect);
        
        let mut puzzle_box = Block::default().title("Select Puzzle").borders(Borders::ALL);
        if self.is_selected {
            puzzle_box = puzzle_box.style( Style::default().fg(Color::Yellow) );
        }
        let mut puzzle_box_bar = Block::default().borders(Borders::ALL);
        if self.is_selected {
            puzzle_box_bar = puzzle_box_bar.style( Style::default().fg(Color::Yellow) );
        }

        f.render_widget(puzzle_box, separator[0]);
        f.render_widget(puzzle_box_bar, separator[1]);
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
    }
    fn exited(&mut self) {
    } 
    
    // todo: move internal position when in this part.
    fn move_left(&mut self) -> (usize, usize) { WINDOW_DO_NOTHING }
    fn move_right(&mut self) -> (usize, usize) { WINDOW_DO_NOTHING }
    fn move_up(&mut self) -> (usize, usize) { WINDOW_DO_NOTHING }
    fn move_down(&mut self) -> (usize, usize) { WINDOW_DO_NOTHING }
}
