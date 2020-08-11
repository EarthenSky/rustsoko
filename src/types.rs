use bit_vec::BitVec;
use std::process;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Tile {
    Wall,
    Player,
    PlayerGoal,
    Crate,
    CrateGoal,
    Goal,
    Floor,
}
impl Tile {
    // for freeze deadlocks
    pub fn is_freezable(&self) -> bool {
        match self {
            Tile::Wall => true,
            Tile::Crate => true,
            Tile::CrateGoal => true,  // this is true because self is crate.
            _ => false,
        }
    }

    pub fn is_pure_crate(&self) -> bool {
        match self {
            Tile::Crate => true,
            _ => false,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TileMatrix {
    pub width: usize,
    pub data: Vec<Tile>,
}
impl TileMatrix {
    pub fn from_string(puzzle_string: &str) -> TileMatrix {
        match puzzle_string.find('\n') {
            Some(v) => v,
            None => {
                println!("Error: puzzle file is malformed.\nreason: must include newline.");
                process::exit(1);
            },
        };
    
        // find puzzle width.
        let mut puzzle_width: usize = 0;
        let mut beg_pos: usize = 0;
        loop {
            match puzzle_string[beg_pos..].find('\n') {
                Some(v) => {
                    beg_pos += v + 1;
                    if v > puzzle_width {
                        puzzle_width = v;
                    }
                },
                None => break,
            };
            if beg_pos > puzzle_string.len() { 
                break; 
            }
        }
    
        if puzzle_width == 0 {
            println!("Error: puzzle file is malformed.\nreason: newline cannot be first character.");
            process::exit(1);
        }
    
        let mut player_count = 0;
        let mut goal_count = 0;
        let mut crate_count = 0;
    
        // map string to Tile enum.
        let mut line_number = 0;
        let mut index = 0;
        let mut tile_vec: Vec<Tile> = Vec::new();
        for ch in puzzle_string.chars() {
            match ch {
                '#' => tile_vec.push(Tile::Wall),
                '@' => {
                    tile_vec.push(Tile::Player); 
                    player_count += 1; 
                },
                '+' => {
                    tile_vec.push(Tile::PlayerGoal); 
                    player_count += 1; 
                    goal_count += 1;
                },
                '$' => {
                    tile_vec.push(Tile::Crate);
                    crate_count += 1;
                },
                '*' => {
                    tile_vec.push(Tile::CrateGoal);
                    goal_count += 1;
                    crate_count += 1;
                },
                '.' => {
                    tile_vec.push(Tile::Goal); 
                    goal_count += 1;
                },
                ' ' => tile_vec.push(Tile::Floor),
                '\n' => {
                    // add extra padding to the map.
                    line_number += 1;
                    for _ in index..(line_number * puzzle_width) {
                        tile_vec.push(Tile::Floor);
                    }
                    index = line_number * puzzle_width;
                    index -= 1;
                },
                '\r' => index -= 1,
                _ => {
                    println!("Error: puzzle file is malformed.\nreason: invalid character in puzzle file, \"{}\".\npuzzle can only contain the characters \"#@+$*. \"", ch);
                    process::exit(1);
                },
            };
            index += 1;
        }
    
        if player_count != 1 {
            println!("Error: puzzle file is malformed.\nreason: There must be exactly 1 player tile, \"@\".");
            process::exit(1);
        }
    
        if crate_count != goal_count {
            println!("Error: puzzle file is malformed.\nreason: There must be the same number of goals and crates.");
            process::exit(1);
        }
    
        TileMatrix {
            width: puzzle_width, 
            data: tile_vec,
        }
    }
    pub fn from_string_bare(puzzle_string: &str) -> TileMatrix {
        match puzzle_string.find('\n') {
            Some(v) => v,
            None => {
                println!("Error: puzzle file is malformed.\nreason: must include newline.");
                process::exit(1);
            },
        };
    
        // find puzzle width.
        let mut puzzle_width: usize = 0;
        let mut beg_pos: usize = 0;
        loop {
            match puzzle_string[beg_pos..].find('\n') {
                Some(v) => {
                    beg_pos += v + 1;
                    if v > puzzle_width {
                        puzzle_width = v;
                    }
                },
                None => break,
            };
            if beg_pos > puzzle_string.len() { 
                break; 
            }
        }
    
        if puzzle_width == 0 {
            println!("Error: puzzle file is malformed.\nreason: newline cannot be first character.");
            process::exit(1);
        }
    
        // map string to Tile enum.
        let mut line_number = 0;
        let mut index = 0;
        let mut tile_vec: Vec<Tile> = Vec::new();
        for ch in puzzle_string.chars() {
            match ch {
                '#' => tile_vec.push(Tile::Wall),
                '@' => tile_vec.push(Tile::Player),
                '+' => tile_vec.push(Tile::PlayerGoal),
                '$' => tile_vec.push(Tile::Crate),
                '*' => tile_vec.push(Tile::CrateGoal),
                '.' => tile_vec.push(Tile::Goal),
                ' ' => tile_vec.push(Tile::Floor),
                '\n' => {
                    // add extra padding to the map.
                    line_number += 1;
                    for _ in index..(line_number * puzzle_width) {
                        tile_vec.push(Tile::Floor);
                    }
                    index = line_number * puzzle_width;
                    index -= 1;
                },
                '\r' => index -= 1,
                _ => {
                    println!("Error: puzzle file is malformed.\nreason: invalid character in puzzle file, \"{}\".\npuzzle can only contain the characters \"#@+$*. \"", ch);
                    process::exit(1);
                },
            };
            index += 1;
        }
    
        TileMatrix {
            width: puzzle_width, 
            data: tile_vec,
        }
    }
    pub fn get(&self, p: Point2D) -> Tile {
        self.data[p.y * self.width + p.x]
    }
    pub fn set(&mut self, p: Point2D, val: Tile) {
        self.data[p.y * self.width + p.x] = val;
    }
    pub fn apply_action_and_move(&mut self, action: Action, crate_start: Point2D, old_map: &TileMatrix, inital_player: Point2D) {
        let crate_end = crate_start.from(action);
        match old_map.get(inital_player) { // update the position the player leaves from.
            Tile::Player => self.set(inital_player, Tile::Floor),
            Tile::PlayerGoal => self.set(inital_player, Tile::Goal),
            _ => (),
        }
        match self.get(crate_start) {  // update the position where the player ends up
            Tile::Crate => self.set(crate_start, Tile::Player),
            Tile::CrateGoal => self.set(crate_start, Tile::PlayerGoal),
            _ => (),
        }
        match self.get(crate_end) {  // update position where the crate ends up
            Tile::Goal => self.set(crate_end, Tile::CrateGoal),
            Tile::PlayerGoal => self.set(crate_end, Tile::CrateGoal),
            _ => self.set(crate_end, Tile::Crate),
        }
    }
    pub fn undo_action(&mut self, action: Action, player_start: Point2D) {
        let empty_pos = player_start.from(action.inverse());
        let crate_start = player_start.from(action);
        match self.get(empty_pos) { // update the position the player leaves from.
            Tile::Floor => self.set(empty_pos, Tile::Player),
            Tile::Goal => self.set(empty_pos, Tile::PlayerGoal),
            _ => (),
        }
        match self.get(player_start) {  // update the position where the player ends up
            Tile::Player  => self.set(player_start, Tile::Crate),
            Tile::PlayerGoal  => self.set(player_start, Tile::CrateGoal),
            _ => (),
        }
        match self.get(crate_start) {  // update position where the crate ends up
            Tile::Crate => self.set(crate_start, Tile::Floor),
            Tile::CrateGoal => self.set(crate_start, Tile::Goal),
            _ => (),
        }
    }
    pub fn print(&self) {
        print!("  ");
        for i in 0..self.width {
            if i < 10 {
                print!("{}", i);
            }
        }
        print!("\n");
        for (i, tile) in self.data.iter().enumerate() {
            if i % self.width == 0 {
                if i / self.width >= 10 {
                    print!("\n  ");
                } else {
                    print!("\n{} ", i / self.width);
                }
            }
            print!("{}", match tile {
                Tile::Wall => '#',
                Tile::Player => '@',
                Tile::PlayerGoal => '+',
                Tile::Crate => '$',
                Tile::CrateGoal => '*',
                Tile::Goal => '.',
                Tile::Floor => ' ',
            });
            
        }
        print!("\n\n");
    }
}

#[derive(Clone)]
pub struct BitMatrix {
    pub width: usize,
    pub bv: BitVec,
} 
impl BitMatrix {
    pub fn new(width: usize, len: usize) -> BitMatrix {
        BitMatrix {
            width, bv: BitVec::from_elem(len, false)
        }
    }
    pub fn get(&self, p: Point2D) -> Option<bool> {
        self.bv.get(p.y * self.width + p.x)
    }
    pub fn set(&mut self, p: Point2D, val: bool) {
        self.bv.set(p.y * self.width + p.x, val);
    }
}

// Simple point struct
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Point2D {
    pub x: usize,
    pub y: usize,
}
impl Point2D {
    pub fn new(x: usize, y: usize) -> Point2D {
        Point2D { x, y }
    }
    pub fn from(&self, action: Action) -> Point2D {
        match action {
            Action::Up => Point2D::new(self.x, self.y - 1),
            Action::Down => Point2D::new(self.x, self.y + 1),
            Action::Left => Point2D::new(self.x - 1, self.y),
            Action::Right => Point2D::new(self.x + 1, self.y),
            Action::PushUp => Point2D::new(self.x, self.y - 1),
            Action::PushDown => Point2D::new(self.x, self.y + 1),
            Action::PushLeft => Point2D::new(self.x - 1, self.y),
            Action::PushRight => Point2D::new(self.x + 1, self.y),
            Action::NoMove => self.clone(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    PushUp,
    PushDown,
    PushLeft,
    PushRight,
    NoMove,
}
impl Action {
    pub fn to_string(actions: &Vec<Action>) -> String {
        let mut s = String::with_capacity(actions.len());
        for item in actions {
            match item {
                Action::Up => s.push('u'),
                Action::Down => s.push('d'),
                Action::Left => s.push('l'),
                Action::Right => s.push('r'),
                Action::PushUp => s.push('U'),
                Action::PushDown => s.push('D'),
                Action::PushLeft => s.push('L'),
                Action::PushRight => s.push('R'),
                Action::NoMove => (),
            }
        }
        s
    }

    // inverse of no move is no move.
    pub fn inverse(&self) -> Action {
        match self {
            Action::Up => Action::Down,
            Action::Down => Action::Up,
            Action::Left => Action::Right,
            Action::Right => Action::Left,
            Action::PushUp => Action::PushDown,
            Action::PushDown => Action::PushUp,
            Action::PushLeft => Action::PushRight,
            Action::PushRight => Action::PushLeft,
            Action::NoMove => Action::NoMove,
        }
    }
}

// This structure stores data about the analysis.
pub struct RunDat {
    pub nodes_checked: usize,
    pub nodes_generated: usize,
    pub nodes_deadlocked: usize,
    pub nodes_skipped: usize,
}
impl RunDat {
    pub fn new() -> RunDat {
        RunDat {
            nodes_checked: 0,
            nodes_generated: 0,
            nodes_deadlocked: 0,
            nodes_skipped: 0,
        }
    }

    pub fn print(&self) {
        println!("-------- Run Data: --------");
        println!("nodes checked = {}", self.nodes_checked);
        println!("nodes generated = {}", self.nodes_generated);
        println!("nodes deadlocked = {}", self.nodes_deadlocked);
        println!("nodes skipped = {}", self.nodes_skipped);
    }
}
