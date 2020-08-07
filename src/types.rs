#[derive(Copy, Clone)]
pub enum Tile {
    Wall,
    Player,
    PlayerGoal,
    Crate,
    CrateGoal,
    Goal,
    Floor,
}

#[derive(Clone)]
pub struct TileMatrix {
    pub width: usize,
    pub data: Vec<Tile>,
} 
impl TileMatrix {
    //pub fn get(&self, x: usize, y: usize) -> Tile {
    //    self.data[y * self.width + x]
    //}
    pub fn get(&self, p: Point2D) -> Tile {
        self.data[p.y * self.width + p.x]
    }
    pub fn set(&mut self, p: Point2D, val: Tile) {
        self.data[p.y * self.width + p.x] = val;
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
                }
                print!("\n{} ", i / self.width);
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
    pub fn pos(&self) -> (usize, usize) {
        (self.x, self.y)
    }
    pub fn from(&self, action: Action) -> Point2D {
        match action {
            Action::Up => Point2D::new(self.x, self.y - 1),
            Action::Down => Point2D::new(self.x, self.y + 1),
            Action::Left => Point2D::new(self.x - 1, self.y),
            Action::Right => Point2D::new(self.x + 1, self.y),
            _ => self.clone(),
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

pub enum Output {
    Found,
    Value(usize),
}

// This structure stores data about the analysis.
pub struct RunDat {
    pub nodes_checked: usize,
    pub nodes_generated: usize,
}
impl RunDat {
    pub fn new() -> RunDat {
        RunDat {
            nodes_checked: 0,
            nodes_generated: 0,
        }
    }

    pub fn print(&self) {
        println!("----------");
        println!("Rundata:");
        println!("nodes checked = {}", self.nodes_checked);
        println!("nodes generated = {}", self.nodes_generated);
        println!("----------");
    }
}