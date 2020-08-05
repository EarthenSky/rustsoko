use std::fs;
use std::process;

use crate::types::{TileMatrix, Tile};

// reads puzzle from the given location and returns a visualizable puzzle matrix.
// Any errors which are encountered automatically exit and give user-bound error message.
pub fn read_puzzle(filepath: &str, print_puzzle: bool) -> TileMatrix {
    let puzzle_string = match fs::read_to_string(filepath) {
        Ok(puzzle) => puzzle,
        Err(_) => {
            println!("Error: Input path does not exist, or cannot be opened.");
            process::exit(1);
        },
    };
    
    let width: usize = match puzzle_string.find('\n') {
        Some(v) => v,
        None => {
            println!("Error: puzzle file is malformed.\nreason: must include newline.");
            process::exit(1);
        },
    };

    if width == 0 {
        println!("Error: puzzle file is malformed.\nreason: newline cannot be first character.");
        process::exit(1);
    }

    // map string to Tile enum.
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
            '\n' => (),
            _ => {
                println!("Error: puzzle file is malformed.\nreason: invalid character in puzzle file, \"{}\".\npuzzle can only contain the characters \"#@+$*. \"", ch);
                process::exit(1);
            },
        };
    }

    // TODO: print numbers with string.
    if print_puzzle {
        println!("Successfully loaded the following puzzle: \n{}", puzzle_string);
    }

    TileMatrix {
        width: width, 
        data: tile_vec,
    }
}