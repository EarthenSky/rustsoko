use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use std::collections::HashSet;
use std::cmp::Ordering;

use crate::types::{Tile, Point2D, TileMatrix, Action, RunDat, BitMatrix};
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

    // damn, this heuristic sucks.
    pub fn goal_count (_solver: &IDAStarSolver, node: &Node) -> usize {  // pub
        let mut goals: usize = 0;
        for tile in &node.map.data {
            goals += match tile {
                Tile::Goal => 1,
                Tile::PlayerGoal => 1,
                _ => 0,
            };
        }
        goals
    }
}

#[derive(Clone)]
pub struct Node {
    pub action: Action,
    pub map: TileMatrix,
    pub crates: Vec<Point2D>,
    pub player: Point2D,
    pub g: usize,  // this is number of pushes
    pub h: usize,  // for storing heuristic(node)
    pub hash: u64,  // odd but okay
}
impl Node {
    // make root
    pub fn default(map: TileMatrix, crates: Vec<Point2D>, player: Point2D) -> Node {
        let hash = Node::hash(&crates, &player);
        Node {
            action: Action::NoMove, map, crates, player, g: 0, h: 0, hash
        }
    }

    pub fn make_new(action: Action, map: TileMatrix, 
                crates: Vec<Point2D>, player: Point2D, g: usize) -> Node {
        let hash = Node::hash(&crates, &player);
        Node {
            action, map, crates, player, g, h: 0, hash 
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
    // TODO: do non-recursive version if neccesary.
    pub fn find_walkable_spaces(&self, current: Point2D, walk_map: &mut BitMatrix) {
        let (x, y) = current.pos();
        let adjacent: Vec<Point2D> = vec![ 
            Point2D::new(x+1, y), Point2D::new(x-1, y), 
            Point2D::new(x, y+1), Point2D::new(x, y-1) 
        ];
        for point in adjacent {
            if walk_map.get(point).unwrap() == false {
                match self.map.get(point) {
                    Tile::Floor => {
                        walk_map.set(point, true);
                        self.find_walkable_spaces(point, walk_map);
                    },
                    Tile::Goal => {
                        walk_map.set(point, true);
                        self.find_walkable_spaces(point, walk_map);
                    },
                    _ => (),
                }
            }
        }
    }

    // Simple freezed deadlock detection, just to see how much it helps.
    pub fn is_deadlocked_simple(&self, moved_crate: Point2D) -> bool {
        match self.map.get(moved_crate) {
            Tile::CrateGoal => return false,
            Tile::Crate => {
                let width = 3;
                let sur_map: Vec<Tile> = vec![
                    moved_crate.from(Action::Up).from(Action::Left),
                    moved_crate.from(Action::Up),
                    moved_crate.from(Action::Up).from(Action::Right),
                    moved_crate.from(Action::Left),
                    moved_crate,
                    moved_crate.from(Action::Right),
                    moved_crate.from(Action::Down).from(Action::Left),
                    moved_crate.from(Action::Down),
                    moved_crate.from(Action::Down).from(Action::Right)
                ].iter().map(|p| self.map.get(*p)).collect();  // surround map

                if sur_map[1] == Tile::Wall && sur_map[3] == Tile::Wall {
                    return true;
                } else if sur_map[1] == Tile::Wall && sur_map[5] == Tile::Wall {
                    return true;
                } else if sur_map[3] == Tile::Wall && sur_map[7] == Tile::Wall {
                    return true;
                } else if sur_map[5] == Tile::Wall && sur_map[7] == Tile::Wall {
                    return true;
                }

                if sur_map[0].is_freezable() && sur_map[1].is_freezable() && sur_map[3].is_freezable() {
                    return true;
                } else if sur_map[1].is_freezable() && sur_map[2].is_freezable() && sur_map[5].is_freezable() {
                    return true;
                } else if sur_map[3].is_freezable() && sur_map[6].is_freezable() && sur_map[7].is_freezable() {
                    return true;
                } else if sur_map[5].is_freezable() && sur_map[7].is_freezable() && sur_map[8].is_freezable() {
                    return true;
                }
            }
            _ => (),
        };
        false
    }
}

pub struct IDAStarSolver {
    debug: bool,
    rundat: RunDat,
    goals: Vec<Point2D>,
    path: Vec<Node>,  // current search path (acts like a stack)
    heuristic: fn(&IDAStarSolver, &Node) -> usize,  // estimated cost of the cheapest path (node..goal)
    solutions: Vec<Vec<Node>>,
}
impl IDAStarSolver {
    pub fn new(puzzle: TileMatrix, heuristic: fn(&IDAStarSolver, &Node) -> usize, debug: bool) -> IDAStarSolver {
        // remove static pieces from the puzzle.
        let map: TileMatrix = TileMatrix { width: puzzle.width, data: Vec::new() };
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
            debug, rundat: RunDat::new(), goals, path, heuristic, solutions: Vec::new()
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
    // TODO: check that all maps are always surrounded by walls, in the loading phase.
    // Node expanding function, expand nodes ordered by g + h(node). Additionally, there is a secondary value which is
    // used to break ties.
    // Step cost is updated in here.
    fn successors(&mut self) -> Vec<Node> {
        let node: &Node = self.path.last().unwrap();

        // find nodes player can access.
        let mut walk_map = BitMatrix::new(node.map.width, node.map.data.len());
        walk_map.set(node.player, true);
        node.find_walkable_spaces(node.player, &mut walk_map);
        /*
        for p in &walkable {
            print!("xy:{},{}  ", p.x, p.y);
        }*/

        // find all the actions the player can take.
        let mut succ_vec: Vec<Node> = Vec::new();
        for (i, crate_pos) in node.crates.iter().enumerate() {
            let adjacent: Vec<Action> = vec![
                Action::PushRight, Action::PushLeft, Action::PushDown, Action::PushUp 
            ];

            // check all four directions.
            for action in adjacent {
                let crate_end = crate_pos.from(action);
                let end_tile = node.map.get(crate_end);
                let push_start = crate_pos.from(action.inverse());
                
                let can_walk = walk_map.get(push_start).unwrap();
                if !can_walk {
                    continue;
                }

                // TODO: put all these internals into the make_new() function.
                match end_tile {
                    Tile::Wall => (),
                    Tile::Crate => (),
                    Tile::CrateGoal => (),
                    _ => {
                        // create new sets of map & crate data. 
                        let mut new_map = node.map.clone();
                        new_map.apply_action_and_move(action, *crate_pos, &node.map, node.player);
                    
                        // This updates the position of the moved crate.
                        let mut new_crates = node.crates.clone();
                        new_crates[i] = crate_end.clone();

                        // every push costs 1
                        let mut new_node = Node::make_new(
                            action, new_map, new_crates, crate_pos.clone(), node.g + 1
                        );

                        // ignore node if it is deadlocked.
                        if !new_node.is_deadlocked_simple(crate_end) {
                            self.rundat.nodes_generated += 1;
                            new_node.h = (self.heuristic)(&self, &new_node);
                            succ_vec.push(new_node);
                        } else {
                            self.rundat.nodes_deadlocked += 1;
                        }
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
            } else {
                Ordering::Equal
            }
        );
        succ_vec
    }  

    // return either NOT_FOUND or a pair with the best path and its cost
    fn ida_star(&mut self) -> (Vec<Action>, usize) {
        let mut bound = self.path.last().unwrap().h; // Oh damn, this is smart.
        while self.solutions.is_empty() {  // TODO: after 300s give statistics of how close it was.
            if self.debug {
                println!("bound updated to {}", bound);
            }
            let new_f = self.search(bound);
            if new_f == std::usize::MAX {
                return (Vec::new(), bound);
            }

            bound = new_f;
        }
        
        println!("starting A*");
        // Find the shortest solution of push-len $bound by using A* to do previously assumed pathfinding.
        let mut min_moves = std::usize::MAX;
        let mut best_move_path: Vec<Action> = Vec::new();
        for solution_path in &mut self.solutions {
            // convert path of nodes to actions, then string.
            let mut action_path: Vec<Action> = Vec::new();
            for i in 1..solution_path.len() {
                let pos_before = solution_path[i - 1].player;
                let node = &mut solution_path[i];
                let pos_after = node.player.from(node.action.inverse());

                // for A*
                node.map.undo_action(node.action, node.player);

                // Using A* is better than IDA* here becase the puzzle is comparatively small, thus we can store all the
                // nodes in memory. A* is also faster than IDA* because of its hard memory usage.
                let mut actions: Vec<Action> = util::astar_pathfind(&node.map, node.action, pos_before, pos_after);
                action_path.append(&mut actions);
            }

            // save min $path.len() of all solution paths constructed from $bound pushes.
            if action_path.len() < min_moves {
                min_moves = action_path.len();
                best_move_path = action_path;
            }
        } 

        if self.debug { 
            println!("-------- Stats 1: --------");
            println!("solutions = {}", self.solutions.len());
        }

        return (best_move_path, bound);
    }

    // adapted from https://en.wikipedia.org/wiki/Iterative_deepening_A*
    fn search(&mut self, bound: usize) -> usize {
        let node: &Node = self.path.last().unwrap();  // End node will always exist.
        let f_cost = node.g + node.h;  // estimated cost of the cheapest path (root..node..goal)
        
        /*
            println!("-------------------------------"); 
            println!("trying:");
            node.map.print();
            println!("player position -> xy:{},{}", &node.player.x, &node.player.y);
            println!("-------------------------------"); 
        */
        
        self.rundat.nodes_checked += 1;

        // base cases
        if f_cost > bound { 
            return f_cost;  // end current dls
        } else if self.is_goal(node) {
            self.solutions.push(self.path.clone());
            return f_cost;  // this number doesn't matter.
        }

        let mut min: usize = std::usize::MAX; // infinity
        for succ in self.successors() {
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
                let new_f = self.search(bound);  // recursion
                if new_f < min {
                    min = new_f;
                }
                
                // hitting this line means that none of this node's children are the goal. (within current bound)
                self.path.pop();
            }
        }
        return min;
    }

    // currently just returns solution as string.
    pub fn solve(&mut self) -> String {
        // run ida_star on the puzzle.
        let (path, cost) = self.ida_star();
        if self.debug {
            println!("-------- Stats 2: --------");
            println!("path cost: {}", cost);
            println!("path len: {}", path.len());
            self.rundat.print();
        }

        if path.len() == 0 {
            return "no solution".to_string();
        }
        Action::to_string(&path)
    }

}