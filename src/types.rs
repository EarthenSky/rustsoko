use bit_vec::BitVec;

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
    pub fn pos(&self) -> (usize, usize) {
        (self.x, self.y)
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
}
impl RunDat {
    pub fn new() -> RunDat {
        RunDat {
            nodes_checked: 0,
            nodes_generated: 0,
        }
    }

    pub fn print(&self) {
        println!("-------- Run Data: --------");
        println!("nodes checked = {}", self.nodes_checked);
        println!("nodes generated = {}", self.nodes_generated);
    }
}