use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use std::collections::HashSet;
use std::cmp::Ordering;

use crate::types::{Tile, Point2D, TileMatrix, Action};
use crate::util;

// Desc:
//   This solver works in terms of pushes, using shortest moves to break ties. Uses
//   IDA* to find shortest paths during runtime.

pub mod heuristic {
    use super::*;

    // sum the manhattan distance from the closest box to each goal.
    // This Heuristic is admissible because there is no way to push a box in fewer spaces than the manhattan distance,
    // however there are many ways to push the box in more ways. Thus h(x) <= h*(x).
    pub fn closest_box (solver: &IDAStarSolver, node: &Node) -> usize {  // pub
        let mut distance: usize = 0;
        for crate_pos in &node.crates {
            let mut min: usize = std::usize::MAX;
            for goal_pos in &solver.goals {
                let dis = util::manhattan_distance(*crate_pos, *goal_pos);
                if min > dis {
                    min = dis;
                }
            }
            distance += min;
        }
        distance
    }
}

// TODO: move to types.
enum Output {
    Found,
    Value(usize),
}

// TODO: this
pub struct RunDat {
    pub nodes_checked: usize,
    pub nodes_generated: usize,
}
impl RunDat {
    pub fn print(&self) {
        println!("nodes checked = {}", self.nodes_checked);
        println!("nodes checked = {}", self.nodes_generated);
    }
}

#[derive(Clone)]
pub struct Node {
    pub actions: Vec<Action>,
    pub map: TileMatrix,
    pub crates: Vec<Point2D>,
    pub player: Point2D,
    pub g: usize,  // this is number of pushes
    pub h: usize,  // for storing heuristic(node)
    pub moves: usize,  // used as a secondary sorting element.
    pub hash: u64,  // odd but okay
}
impl Node {
    // make root
    pub fn default(map: TileMatrix, crates: Vec<Point2D>, player: Point2D) -> Node {
        let hash = Node::hash(&crates, &player);
        Node {
            actions: vec![Action::NoMove], map, crates, player, g: 0, moves: 0, h: 0, hash
        }
    }

    pub fn make_new(actions: Vec<Action>, map: TileMatrix, 
                crates: Vec<Point2D>, player: Point2D, g: usize, moves: usize) -> Node {
        let hash = Node::hash(&crates, &player);
        Node {
            actions, map, crates, player, g, moves, h: 0, hash 
        }
    }

    // hashes crate positions & player position.
    fn hash(crates: &Vec<Point2D>, player: &Point2D) -> u64 {
        let mut s = DefaultHasher::new();
        crates.hash(&mut s);
        player.hash(&mut s);
        s.finish()
    }

    // effectively a recursive floodfill algorithm.
    // TODO: do non-recursive version.
    pub fn find_walkable_spaces(&self, current: Point2D, walkable_spaces: &mut HashSet<Point2D>) {
        let (x, y) = current.pos();
        let adjacent: Vec<Point2D> = vec![ 
            Point2D::new(x+1, y), Point2D::new(x, y+1), 
            Point2D::new(x-1, y), Point2D::new(x, y-1) 
        ];
        // DEBUG: println!("xy:{},{}", current.x, current.y);
        for point in adjacent {
            if !walkable_spaces.contains(&point) {
                match self.map.get(point) {
                    Tile::Floor => {
                        walkable_spaces.insert(point);
                        self.find_walkable_spaces(point, walkable_spaces);
                    },
                    Tile::Goal => {
                        walkable_spaces.insert(point);
                        self.find_walkable_spaces(point, walkable_spaces);
                    },
                    _ => (),
                }
            }
        }
    }
}

pub struct IDAStarSolver {
    goals: Vec<Point2D>,
    path: Vec<Node>,  // current search path (acts like a stack)
    heuristic: fn(&IDAStarSolver, &Node) -> usize,  // estimated cost of the cheapest path (node..goal)
}
impl IDAStarSolver {
    pub fn new(puzzle: TileMatrix, heuristic: fn(&IDAStarSolver, &Node) -> usize) -> IDAStarSolver {
        // remove static pieces from the puzzle.
        let map: TileMatrix = TileMatrix { width: puzzle.width, data: Vec::new(), };
        let mut goals: Vec<Point2D> = Vec::new();
        let mut crates: Vec<Point2D> = Vec::new();
        let mut player: Option<Point2D> = None;
        for (i, tile) in puzzle.data.iter().enumerate() {
            let (x, y) = (i % puzzle.width, i / puzzle.width);
            match tile {
                Tile::Player => {
                    player = Some(Point2D::new(x, y));
                },
                Tile::PlayerGoal => {
                    player = Some(Point2D::new(x, y));
                    goals.push( Point2D::new(x, y) );
                },
                Tile::Crate => {
                    crates.push( Point2D::new(x, y) );
                },
                Tile::CrateGoal => {
                    goals.push( Point2D::new(x, y) );
                    crates.push( Point2D::new(x, y) );
                },
                Tile::Goal => {
                    goals.push( Point2D::new(x, y) );
                },
                _ => (),
            };
        }

        // Map size is a good vector size estimate which should increase performance because IDA* doesn't particularly
        // need lots of memory.
        let mut path: Vec<Node> = Vec::with_capacity(map.data.len());
        let root_node = Node::default(puzzle, crates, player.unwrap());
        path.push(root_node);
        
        let mut solver = IDAStarSolver {
            goals, path, heuristic
        };
        solver.path[0].h = (solver.heuristic)(&solver, &solver.path[0]); 
        solver
    }
    
    // if a crate is not on a goal, then it is not solved.
    // TODO: have node store how many crates are on nodes.
    fn is_goal(&self, node: &Node) -> bool {
        for tile in &node.map.data {
            if let Tile::Crate = tile {
                return false;
            }
        }
        true
    }

    // We know that crates will never be on the edge of the map.
    // TODO: check that all maps are always surrounded by walls. in the loading phase.
    // Node expanding function, expand nodes ordered by g + h(node). Additionally, there is a secondary value which is
    // used to break ties.
    // Step cost is updated in here.
    fn successors(&self, node: &Node) -> Vec<Node> {
        // find nodes player can access.
        let mut walkable: HashSet<Point2D> = HashSet::new();
        walkable.insert(node.player.clone());
        node.find_walkable_spaces(node.player, &mut walkable);
        for p in &walkable {
            print!("xy:{},{}  ", p.x, p.y);
        }
        println!("\ndone walk ##########");

        // find all the actions the player can take.
        let mut succ_vec: Vec<Node> = Vec::new();
        for (i, crate_pos) in node.crates.iter().enumerate() {
            let (x, y) = crate_pos.pos();
            let crate_tile = node.map.get(*crate_pos);

            // TODO: don't make Point2Ds for efficiency -> map.get_pos(x, y)
            let adjacent: Vec<(Action, Point2D, Point2D, Tile, bool)> = vec![ 
                (Action::PushRight, Point2D::new(x+1, y), Point2D::new(x-1, y)), 
                (Action::PushLeft, Point2D::new(x-1, y), Point2D::new(x+1, y)), 
                (Action::PushDown, Point2D::new(x, y+1), Point2D::new(x, y-1)), 
                (Action::PushUp, Point2D::new(x, y-1), Point2D::new(x, y+1)) 
            ].iter().map( |tup| (tup.0, tup.1, tup.2, node.map.get(tup.1), walkable.contains(&tup.2)) ).collect();

            for (action, crate_end, push_start, end_tile, can_walk) in adjacent {
                if !can_walk {
                    continue;
                }

                // TODO: put all the internals into the make_new() function.
                match end_tile {
                    Tile::Wall => (),
                    Tile::Crate => (),
                    Tile::CrateGoal => (),
                    _ => {
                        // create new sets of map & crate data. 
                        let mut new_map = node.map.clone();
                        let mut new_crates = node.crates.clone();
                        
                        match node.map.get(node.player) { // update the position the player leaves from.
                            Tile::Player => new_map.set(node.player, Tile::Floor),
                            Tile::PlayerGoal => new_map.set(node.player, Tile::Goal),
                            _ => (),
                        }
                        match crate_tile {  // update the position where the player ends up
                            Tile::Crate => new_map.set(*crate_pos, Tile::Player),
                            Tile::CrateGoal => new_map.set(*crate_pos, Tile::PlayerGoal),
                            _ => (),
                        }
                        match end_tile {  // update position where the crate ends up
                            Tile::Goal => new_map.set(crate_end, Tile::CrateGoal),
                            Tile::PlayerGoal => new_map.set(crate_end, Tile::CrateGoal),
                            _ => new_map.set(crate_end, Tile::Crate),
                        }

                        // TODO: do A* for pathfinding to determine moves needed.
                        // Using A* is better than IDA* here becase the puzzle is a lot smaller so we can store
                        // enough nodes in memory. A* is also faster than IDA* because of its memory constraints.
                        let actions: Vec<Action> = util::astar_pathfind(&new_map, action, node.player, push_start);
                        let moves: usize = 1; //node.moves + actions.len() - 1;
                        
                        // This updates the position of the moved crate.
                        new_crates[i] = crate_end.clone();
                        
                        // every push costs 1
                        let mut new_node = Node::make_new(
                            actions, new_map, new_crates, crate_pos.clone(), node.g + 1, moves
                        );
                        new_node.h = (self.heuristic)(&self, &new_node);
                        succ_vec.push(new_node);
                    }
                }
            }
        }
        // sort by g cost, then by moves to break ties.
        succ_vec.sort_by(|n1, n2| 
            if n1.g + n1.h > n2.g + n2.h {
                Ordering::Greater
            } else if n1.g + n1.h < n2.g + n2.h {
                Ordering::Less
            } else if n1.moves > n2.moves {
                Ordering::Greater
            } else if n1.moves < n2.moves {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        );
        println!("______ found succ!");
        succ_vec
    }  

    // return either NOT_FOUND or a pair with the best path and its cost
    fn ida_star(&mut self) -> (Vec<Node>, usize) {
        let mut bound = self.path.last().unwrap().h; // Oh damn, this is smart.
        loop { // TODO: after 300s give statistics of how close it was.
            let out = self.search(bound);
            match out {
                Output::Found => return (self.path.clone(), bound),
                Output::Value(new_f) => {
                    if new_f == std::usize::MAX {
                        println!("### No Solution ###");
                        return (Vec::new(), 0);
                    }
                    bound = new_f;
                    println!("bound updated {}", bound);
                }
            }
        }
    }

    // adapted from https://en.wikipedia.org/wiki/Iterative_deepening_A*
    fn search(&mut self, bound: usize) -> Output {
        let node: &Node = self.path.last().unwrap();  // End node will allways exist.
        let f_cost = node.g + node.h;  // estimated cost of the cheapest path (root..node..goal)
        
        println!("trying:"); 
        node.map.print();
        println!("player position -> xy:{},{}", &node.player.x, &node.player.y);
        // base cases
        if f_cost > bound { 
            return Output::Value(f_cost);  // end current dls
        } else if self.is_goal(node) {
            return Output::Found;
        }

        let mut min: usize = std::usize::MAX; // infinity
        for succ in self.successors(node) {
            // if by chance there is a hash collision, the worst that will happen is a small
            // performance hit. However this is extremely unlikely so its all good.
            let mut is_duplicate = false;
            for node in &self.path {
                if node.hash == succ.hash {
                    is_duplicate = true;
                    break;
                }
            }

            if !is_duplicate {
                self.path.push(succ);
                let out = self.search(bound);  // recursion

                // pass down base cases
                match out {
                    Output::Found => return Output::Found,
                    Output::Value(new_f) => if new_f < min {
                        min = new_f;
                    },
                }
                
                // hitting this line means that none of this node's children are the goal. (within current bound)
                self.path.pop();
            }
        }
        return Output::Value(min);
    }

    // currently just returns solution as string.
    pub fn solve(&mut self) -> String {
        let (mut path, cost) = self.ida_star();
        println!("path cost: {}", cost);
        println!("path len: {}", path.len());
        //println!("path ac0: {}", path[0].actions.len());
        //println!("path ac1: {}", path[1].actions.len());

        // convert path of nodes to actions, then string.
        let mut action_path: Vec<Action> = Vec::new();
        for node in &mut path {
            action_path.append(&mut node.actions);
        }
        println!("path acpl: {}", action_path.len());
        Action::to_string(&action_path)
    }

}