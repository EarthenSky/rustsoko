use std::fs::File;
use std::io::Write;
use std::process;

use rand::prelude::*;

use crate::util;
use crate::types::{TileMatrix, Point2D, BitMatrix};

// Generate levels based on:
// 1. dimensions -> H x W
// 2. complexity -> Goals and boxes
// 3. boxes cannot be placed in 'simple deadlocks'

fn init_puzzle(width: usize, height: usize) -> Vec<char> {
    let mut puzzle_vec: Vec<char> = Vec::new();

    for y in 0..height {
        for x in 0..width {
            if x == 0 || x == width-1 || y == 0 || y == height-1 {
                puzzle_vec.push('#');
            } else {
                puzzle_vec.push(' ');
            }
            
        }
        puzzle_vec.push('\n');
    }
    puzzle_vec
}

pub fn make_sok(file_name: &str, width: usize, height: usize, batch_num: usize, goal_num: usize, wall_num: usize) {
    let mut rng = rand::thread_rng();
    let mut file_string = format!("Date of Last Change:\n\nSet: {}\nCopyright: Geb\nEmail:\nHomepage:\n\n\
                                   This sokoban puzzle set was automatically generated\n\n", file_name);
    let inner_spaces = (width - 2) * (height - 2);

    let mut i = 0;
    while i < batch_num {
        let mut puzzle_vec: Vec<char> = init_puzzle(width, height);

        let mut goals: Vec<Point2D> = Vec::new();
        let mut goals_added = 0;
        while goals_added < goal_num {
            let index = rng.gen_range(0, inner_spaces);
            let (x, y) = ((index % (width - 2)) + 1, (index / (width - 2)) + 1);
            let index = x + y * (width + 1);
            if puzzle_vec[index] == ' ' {
                puzzle_vec[index] = '.';
                goals_added += 1;
                goals.push( Point2D::new(x, y) );
            }
        }

        let mut walls_added = 0;
        while walls_added < wall_num {
            let index = rng.gen_range(0, inner_spaces);
            let (x, y) = ((index % (width - 2)) + 1, (index / (width - 2)) + 1);
            let index = x + y * (width + 1);
            if puzzle_vec[index] == ' ' {
                puzzle_vec[index] = '#';
                walls_added += 1;
            }
        }

        // Check all locations which can be pulled to.
        let mut puzzle_string = String::new();
        for ch in &puzzle_vec {
            puzzle_string.push(*ch);
        }
        let tile_map = TileMatrix::from_string_bare(&puzzle_string[..]);
        let good_spaces: BitMatrix = util::find_simple_deadlocks(&tile_map, &goals);

        // check if there are enough spaces for the crates to go in.
        let mut crate_spaces = 0;
        for y in 0..height {
            for x in 0..width {
                if good_spaces.get(Point2D::new(x, y)).unwrap()
                   && puzzle_vec[x + y * (width+1)] == ' ' {
                    crate_spaces += 1;
                }
            }
        }
        
        if crate_spaces < goal_num {
            println!("Could not assign crates, retrying current puzzle...");
            continue;
        }

        let mut crates_added = 0;
        while crates_added < goal_num {
            let index = rng.gen_range(0, inner_spaces);
            let (x, y) = ((index % (width - 2)) + 1, (index / (width - 2)) + 1);
            let index = x + y * (width + 1);
            if puzzle_vec[index] == ' ' && good_spaces.get(Point2D::new(x, y)).unwrap() {
                puzzle_vec[index] = '$';
                crates_added += 1;
            }
        }

        let mut player_added = false;
        while !player_added {
            let index = rng.gen_range(0, inner_spaces);
            let (x, y) = ((index % (width - 2)) + 1, (index / (width - 2)) + 1);
            let index = x + y * (width + 1);
            if puzzle_vec[index] == ' ' {
                puzzle_vec[index] = '@';
                player_added = true;
            } else if puzzle_vec[index] == '.' {
                puzzle_vec[index] = '+';
                player_added = true;
            }
        }

        // add to string
        file_string.push_str(&format!("{}\n", i+1));
        for ch in puzzle_vec {
            file_string.push(ch);
        }
        file_string.push_str("\n\n");

        i += 1;
    }

    // write string to file.
    let mut file = match File::create( format!("{}.sok", file_name) ) {
        Ok(f) => f,
        Err(_) => {
            println!("Error: file unable to be created.");
            process::exit(1);
        },
    };
    match file.write_all(file_string.as_bytes()) {
        Ok(_) => {
            println!("File written!");
        },
        Err(_) => {
            println!("Error: file unable to be written.");
            process::exit(1);
        },
    };
}