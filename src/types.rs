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
}
//impl Eq for ShouldBeEq {}

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
}