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

pub struct TileMatrix {
    pub width: usize,
    pub data: Vec<Tile>,
} 
impl TileMatrix {
    pub fn get(&self, x: usize, y: usize) -> Tile {
        self.data[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, val: Tile) {
        self.data[y * self.width + x] = val;
    }
}