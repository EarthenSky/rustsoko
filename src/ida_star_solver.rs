use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use std::collections::HashSet;
use std::cmp::Ordering;

use std::time::Instant;

use crate::types::{Tile, Point2D, TileMatrix, Action, RunDat, BitMatrix};
use crate::util;

const TIME_LIMIT: u64 = 300;
const PER_NODE_TIME_CHECK: usize = 10_000;  // check time once per n nodes

pub mod heuristic {
    use super::*;

    // sum the manhattan distance from the closest box to each goal.
    // This Heuristic is admissible because there is no way to push a box in fewer spaces than the manhattan distance,
    // however there are many ways to push the box in more ways. Thus h(x) <= h*(x).
    pub fn closest_box (solver: &IDAStarSolver, node: &Node) -> usize {
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
    pub fn goal_count (_solver: &IDAStarSolver, node: &Node) -> usize {
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

    // Attempts to find perfect matches, but when it fails it simply falls back on the closest box heuristic.
    pub fn greedy_perfect_match(solver: &IDAStarSolver, node: &Node) -> usize {
        let mut dis_vec: Vec<usize> = Vec::new();
        let width: usize = node.crates.len();

        // get initial settings. & step 0
        for crate_pos in &node.crates {
            for goal_pos in &solver.goals {
                let dis = util::manhattan_distance(*crate_pos, *goal_pos);
                dis_vec.push(dis);
            }
        }

        let dis_vec_clone = dis_vec.clone();

        // step 0
        for y in 0..width {
            let mut min: usize = std::usize::MAX;
            for x in 0..width {
                let dis = dis_vec[y * width + x];
                if min > dis {
                    min = dis;
                }
            }

            for x in 0..width {
                dis_vec[y * width + x] -= min;
            }
        }

        // step 0
        for x in 0..width {
            let mut min: usize = std::usize::MAX;
            for y in 0..width {
                let dis = dis_vec[y * width + x];
                if min > dis {
                    min = dis;
                }
            }

            for y in 0..width {
                dis_vec[y * width + x] -= min;
            }
        }

        // attempt assignment
        let mut distance: usize = 0;
        let mut taken: Vec<bool> = Vec::new();
        taken.resize(width, false);
        for y in 0..width {
            for x in 0..width {
                if dis_vec[y * width + x] == 0 && !taken[y] {
                    distance += dis_vec_clone[y * width + x];
                    taken[y] = true;
                    break;
                }
            }

            // just take min if can't find a best.
            if taken[y] == false {
                let mut min: usize = std::usize::MAX;
                let mut min_x: usize = 0;
                for x in 0..width {
                    if dis_vec[y * width + x] < min {
                        min = dis_vec[y * width + x];
                        min_x = x;
                    }
                }
                distance += dis_vec_clone[y * width + min_x];
            }
        }

        return distance;
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

    // Simple freezed deadlock detection, just to see how much it helps.
    pub fn is_deadlocked(&self, moved_crate: Point2D) -> bool {
        match self.map.get(moved_crate) {
            Tile::CrateGoal => {
                let _width = 3;
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

                if sur_map[0].is_freezable() && sur_map[1].is_freezable() && sur_map[3].is_freezable() && 
                   (sur_map[0].is_pure_crate() || sur_map[1].is_pure_crate() || sur_map[3].is_pure_crate()) {
                    return true;
                } else if sur_map[1].is_freezable() && sur_map[2].is_freezable() && sur_map[5].is_freezable() && 
                          (sur_map[1].is_pure_crate() || sur_map[2].is_pure_crate() || sur_map[5].is_pure_crate()) {
                    return true;
                } else if sur_map[3].is_freezable() && sur_map[6].is_freezable() && sur_map[7].is_freezable() && 
                          (sur_map[3].is_pure_crate() || sur_map[6].is_pure_crate() || sur_map[7].is_pure_crate()) {
                    return true;
                } else if sur_map[5].is_freezable() && sur_map[7].is_freezable() && sur_map[8].is_freezable() && 
                          (sur_map[5].is_pure_crate() || sur_map[7].is_pure_crate() || sur_map[8].is_pure_crate()) {
                    return true;
                }
                //return false;
            },
            Tile::Crate => {
                let _width = 3;
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

// Desc:
//   This solver works in terms of pushes, finding best moves after execution.
pub struct IDAStarSolver {
    debug: bool,
    deadlock_hashing_on: bool,
    rundat: RunDat,
    goals: Vec<Point2D>,
    path: Vec<Node>,  // current search path (acts like a stack)
    heuristic: fn(&IDAStarSolver, &Node) -> usize,  // estimated cost of the cheapest path (node..goal)
    solutions: Vec<Vec<Node>>,
    simple_deadlocks: BitMatrix,
    deadlocks: HashSet<TileMatrix>,
    timer: Instant,
    search_over: bool,
}
impl IDAStarSolver {
    pub fn new(puzzle: TileMatrix, heuristic: fn(&IDAStarSolver, &Node) -> usize, deadlock_hashing_on: bool, debug: bool) -> IDAStarSolver {
        // remove static pieces from the puzzle.
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

        let simple_deadlocks: BitMatrix = util::find_simple_deadlocks(&puzzle, &goals);

        // puzzle size is a good vector size estimate which should increase performance because IDA* doesn't particularly
        // need lots of memory. -------------------------------> vVVVv
        let mut path: Vec<Node> = Vec::with_capacity(puzzle.data.len());
        let root_node = Node::default(puzzle, crates, player.unwrap());
        path.push(root_node);

        let mut solver = IDAStarSolver {
            debug, deadlock_hashing_on, rundat: RunDat::new(), goals, path, 
            heuristic, solutions: Vec::new(), deadlocks: HashSet::new(), simple_deadlocks,
            timer: Instant::now(), search_over: false
        };
        solver.path[0].h = (solver.heuristic)(&solver, &solver.path[0]); 
        solver
    }

    fn is_simple_deadlock(&self, pos: Point2D) -> bool {
        !self.simple_deadlocks.get(pos).unwrap()
    }
    
    // if a crate is not on a goal, then it is not solved.
    fn is_goal(&self, node: &Node) -> bool {
        for tile in &node.map.data {
            if let Tile::Crate = tile {
                return false;
            }
        }
        true
    }

    // We know that crates will never be on the edge of the map.
    // Node expanding function, expand nodes ordered by g + h(node). Additionally, there is a secondary value which is
    // used to break ties.
    // Step cost is updated in here.
    fn successors(&mut self) -> Vec<Node> {
        let node: &Node = self.path.last().unwrap();

        // find nodes player can access.
        let mut walk_map = BitMatrix::new(node.map.width, node.map.data.len());
        walk_map.set(node.player, true);
        util::find_walkable_spaces(&node.map, node.player, &mut walk_map);

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
                } else if self.is_simple_deadlock(crate_end) {
                    self.rundat.nodes_skipped += 1;
                    continue;
                }

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
                        if !new_node.is_deadlocked(crate_end) {
                            self.rundat.nodes_generated += 1;
                            new_node.h = (self.heuristic)(&self, &new_node);
                            succ_vec.push(new_node);
                        } else {
                            self.rundat.nodes_deadlocked += 1;
                            continue;
                        }
                    }
                }
            }
        }

        // sort by f cost?
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

    fn ida_star(&mut self) -> (Vec<Action>, usize, usize) {
        let mut bound = self.path.last().unwrap().h; // Oh damn, this is smart.
        while self.solutions.is_empty() {
            if self.debug {
                println!("DEBUG: bound updated to {}", bound);
            }
            let new_f = self.search(bound);
            if new_f == std::usize::MAX {
                return (Vec::new(), 0, bound);
            }

            bound = new_f;
        }
        
        if self.debug {
            println!("DEBUG: starting A* ...");
        }

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

        return (best_move_path, self.solutions.len(), bound);
    }

    // adapted from https://en.wikipedia.org/wiki/Iterative_deepening_A*
    fn search(&mut self, bound: usize) -> usize {
        let node: &Node = self.path.last().unwrap();  // End node will always exist.
        let f_cost = node.g + node.h;  // estimated cost of the cheapest path (root..node..goal)
    
        if self.rundat.nodes_checked % PER_NODE_TIME_CHECK == 0 && self.timer.elapsed().as_secs() > TIME_LIMIT {
            self.search_over = true;
        }

        if self.search_over  {
            return std::usize::MAX;
        }

        self.rundat.nodes_checked += 1;

        // base cases
        if f_cost > bound { 
            return f_cost;  // end current dls
        } else if self.is_goal(node) {
            self.solutions.push(self.path.clone());
            return f_cost;  // this number doesn't matter.
        } else if self.deadlock_hashing_on && self.deadlocks.contains(&node.map) {
            return std::usize::MAX; // this means no solution will be found behind this.
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
                let node = self.path.pop().unwrap();
                if self.deadlock_hashing_on && min == std::usize::MAX {
                    self.deadlocks.insert(node.map);
                }
            }
        }
        
        return min;
    }

    // currently just returns solution as string.
    pub fn solve(&mut self) -> String {
        self.timer = Instant::now();

        // run ida_star on the puzzle.
        let (path, solutions, bound) = self.ida_star();

        if self.debug {
            println!("-------- Stats: --------");
            println!("solutions: {}", solutions);
            println!("final bound: {}", bound);
            println!("path len: {}", path.len());
            println!("time elapsed (in seconds) = {}", self.timer.elapsed().as_secs_f32());
            self.rundat.print();
        }

        if self.search_over {
            if self.debug {
                return "time elapsed".to_string();
            } else {
                return format!("{},{},{},{},{},{}", self.timer.elapsed().as_secs_f32(), self.rundat.nodes_checked, solutions, bound, path.len(), "".to_string());
            }
        }

        if path.len() == 0 {
            if self.debug {
                return "no solution".to_string();
            } else {
                return format!("{},{},{},{},{},{}", self.timer.elapsed().as_secs_f32(), self.rundat.nodes_checked, solutions, bound, path.len(), "".to_string());
            }
        }

        if self.debug {
            return Action::to_string(&path);
        } else {
            return format!("{},{},{},{},{},{}", self.timer.elapsed().as_secs_f32(), self.rundat.nodes_checked, solutions, bound, path.len(), Action::to_string(&path));
        }
    }

}