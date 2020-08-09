use bit_vec::BitVec;

#[derive(Copy, Clone, PartialEq, Eq)]
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

    pub fn to_tile_data(&self) -> TileData {
        match self {
            Tile::Wall => TileData::Wall,
            Tile::Player => TileData::Player,
            Tile::PlayerGoal => TileData::Goal,
            Tile::Crate => TileData::Crate,
            Tile::CrateGoal => TileData::CrateGoal,
            Tile::Goal => TileData::Goal,
            Tile::Floor => TileData::Floor(false, false, false, false),
        }
    }
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
    pub nodes_deadlocked: usize,
}
impl RunDat {
    pub fn new() -> RunDat {
        RunDat {
            nodes_checked: 0,
            nodes_generated: 0,
            nodes_deadlocked: 0,
        }
    }

    pub fn print(&self) {
        println!("-------- Run Data: --------");
        println!("nodes checked = {}", self.nodes_checked);
        println!("nodes generated = {}", self.nodes_generated);
        println!("nodes deadlocked = {}", self.nodes_deadlocked);
    }
}


// Rules:
// - a dead space is a space in which it is impossible for the player to ever move behind the tile.
// - a tile cannot be pushed into a dead space.
// -


// TODO: this
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TileData {
    Wall,
    Crate(bool, bool, bool, bool),
    CrateGoal(bool, bool, bool, bool),
    Goal,
    //up, right, down, left -> true means that a box cannot be pushed from that direction into the space.
    Floor(bool, bool, bool, bool),  
}
impl TileData {
    
}

#[derive(Clone)]
pub struct TileDataMatrix {
    pub width: usize,
    pub data: Vec<TileData>,
} 
impl TileDataMatrix {
    pub fn get(&self, p: Point2D) -> TileData {
        self.data[p.y * self.width + p.x]
    }
    pub fn set(&mut self, p: Point2D, val: TileData) {
        self.data[p.y * self.width + p.x] = val;
    }

    fn apply_wall_check(&self) {
        // ignore the direct edges (because they can never be useful tiles.)
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                if !(y == 0 || x == 0 || y == height-1 || x == width-1) {
                    let cur_pos = tile_data_map.get(Point2D::new(x, y));
                    match cur_pos {
                        TileData::Floor(up, right, down, left) => {
                            let up_tile_data = tile_data_map.get(cur_pos.from(Action::Up));
                            let right_tile_data = tile_data_map.get(cur_pos.from(Action::Right));
                            let down_tile_data = tile_data_map.get(cur_pos.from(Action::Down));
                            let left_tile_data = tile_data_map.get(cur_pos.from(Action::Left));
                            if up_tile_data == TileData::Wall || up_tile_data.can_move_perpendicular(cur_pos) {
                                if !up || !down {
                                    up = true;
                                    down = true;
                                    updated_tile += 1;
                                }
                            }
                            if right_tile_data == TileData::Wall || right_tile_data.can_move_perpendicular(cur_pos) {
                                if !right || !left {
                                    left = true;
                                    right = true;
                                    updated_tile += 1;
                                }
                            }
                            if down_tile_data == TileData::Wall || down_tile_data.can_move_perpendicular(cur_pos) {
                                if !down || !up {
                                    up = true;
                                    down = true;
                                    updated_tile += 1;
                                }
                            }
                            if left_tile_data == TileData::Wall || left_tile_data.can_move_perpendicular(cur_pos) {
                                if !left || !right {
                                    left = true;
                                    right = true;
                                    updated_tile += 1;
                                }
                            }
                            tile_data_map.set(cur_pos, TileData::floor(up, right, down, left));
                        }
                        _ => (),
                    };
                }
            }
        }
    }

    fn apply_wall_check(&self) {
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                if !(y == 0 || x == 0 || y == height-1 || x == width-1) {
                    let cur_pos = tile_data_map.get(Point2D::new(x, y));
                    match cur_pos {
                        TileData::Crate(up, right, down, left) => {
                            let up_tile_data = tile_data_map.get(cur_pos.from(Action::Up));
                            let right_tile_data = tile_data_map.get(cur_pos.from(Action::Right));
                            let down_tile_data = tile_data_map.get(cur_pos.from(Action::Down));
                            let left_tile_data = tile_data_map.get(cur_pos.from(Action::Left));
                            if up_tile_data == TileData::Wall || up_tile_data.can_move_perpendicular(cur_pos) {
                                if !up || !down {
                                    up = true;
                                    down = true;
                                    updated_tile += 1;
                                }
                            }
                            if right_tile_data == TileData::Wall || right_tile_data.can_move_perpendicular(cur_pos) {
                                if !right || !left {
                                    left = true;
                                    right = true;
                                    updated_tile += 1;
                                }
                            }
                            if down_tile_data == TileData::Wall || down_tile_data.can_move_perpendicular(cur_pos) {
                                if !down || !up {
                                    up = true;
                                    down = true;
                                    updated_tile += 1;
                                }
                            }
                            if left_tile_data == TileData::Wall || left_tile_data.can_move_perpendicular(cur_pos) {
                                if !left || !right {
                                    left = true;
                                    right = true;
                                    updated_tile += 1;
                                }
                            }
                            tile_data_map.set(cur_pos, TileData::floor(up, right, down, left));
                        }
                        _ => (),
                    };
                }
            }
        }
    }

    pub fn from_init(tile_map: &TileMatrix) -> TileDataMatrix {
        let (width, height) = (tile_map.width, tile_map.data.len() / width);
        let data Vec<TileData> = Vec::with_capacity(tile_map.data.len());
        
        for tile in &tile_map.data {
            data.push( tile.to_tile_data() );
        }

        let mut tile_data_map = TileDataMatrix {
            width, data
        };

        self.apply_wall_check(); // checks walls for dead spaces
        // TODO: interpolate dead spaces between flat walls.
        
        let mut tiles_updated = 0;
        while tiles_updated != 0 {
            tiles_updated = 0;
            tiles_updated += self.apply_crate_check();
            tiles_updated += self.apply_dead_space_check();
        }
        
    }

    pub fn is_valid(&self, crate_pos: Point2D, action: Action) {
        // TODO: return if a crate move is allowed to be made
        match self.get(crate_pos.from(action)) {
            TileData::Floor(up, right, down, left) {
                
            }
            TileData::Wall => return false,
            _ => (),
        }
        true
    }
}
