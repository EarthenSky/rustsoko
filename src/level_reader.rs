use std::fs;
use std::process;

use std::path::Path;
use std::ffi::OsStr;

use crate::types::{TileMatrix, Tile};

pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}

fn puzzle_string_to_puzzle(puzzle_string: &str) -> TileMatrix {
    match puzzle_string.find('\n') {
        Some(v) => v,
        None => {
            println!("Error: puzzle file is malformed.\nreason: must include newline.");
            process::exit(1);
        },
    };

    // find puzzle width.
    let mut puzzle_width: usize = 0;
    let mut beg_pos: usize = 0;
    loop {
        match puzzle_string[beg_pos..].find('\n') {
            Some(v) => {
                beg_pos += v + 1;
                if v > puzzle_width {
                    puzzle_width = v;
                }
            },
            None => break,
        };
        if beg_pos > puzzle_string.len() { 
            break; 
        }
    }

    if puzzle_width == 0 {
        println!("Error: puzzle file is malformed.\nreason: newline cannot be first character.");
        process::exit(1);
    }

    let mut player_count = 0;
    let mut goal_count = 0;
    let mut crate_count = 0;

    // map string to Tile enum.
    let mut line_number = 0;
    let mut index = 0;
    let mut tile_vec: Vec<Tile> = Vec::new();
    for ch in puzzle_string.chars() {
        match ch {
            '#' => tile_vec.push(Tile::Wall),
            '@' => {
                tile_vec.push(Tile::Player); 
                player_count += 1; 
            },
            '+' => {
                tile_vec.push(Tile::PlayerGoal); 
                player_count += 1; 
                goal_count += 1;
            },
            '$' => {
                tile_vec.push(Tile::Crate);
                crate_count += 1;
            },
            '*' => {
                tile_vec.push(Tile::CrateGoal);
                goal_count += 1;
                crate_count += 1;
            },
            '.' => {
                tile_vec.push(Tile::Goal); 
                goal_count += 1;
            },
            ' ' => tile_vec.push(Tile::Floor),
            '\n' => {
                // add extra padding to the map.
                line_number += 1;
                for _ in index..(line_number * puzzle_width) {
                    tile_vec.push(Tile::Floor);
                }
                index = line_number * puzzle_width;
                index -= 1;
            },
            '\r' => index -= 1,
            _ => {
                println!("Error: puzzle file is malformed.\nreason: invalid character in puzzle file, \"{}\".\npuzzle can only contain the characters \"#@+$*. \"", ch);
                process::exit(1);
            },
        };
        index += 1;
    }

    if player_count != 1 {
        println!("Error: puzzle file is malformed.\nreason: There must be exactly 1 player tile, \"@\".");
        process::exit(1);
    }

    if crate_count != goal_count {
        println!("Error: puzzle file is malformed.\nreason: There must be the same number of goals and crates.");
        process::exit(1);
    }

    TileMatrix {
        width: puzzle_width, 
        data: tile_vec,
    }
}

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
    
    let puzzle: TileMatrix = TileMatrix::from_string(&puzzle_string[..]);

    if print_puzzle {
        println!("Successfully loaded the following puzzle:");
        puzzle.print();
    }
    puzzle
}

const HEADER_START: &str = "::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::";
pub fn read_sok(filepath: &str, verbose: bool) -> Vec<TileMatrix> {
    let file_string = match fs::read_to_string(filepath) {
        Ok(puzzle) => puzzle,
        Err(_) => {
            println!("Error: Input path does not exist, or cannot be opened.");
            process::exit(1);
        },
    };

    let mut puzzles: Vec<TileMatrix> = Vec::new();
    let mut current_puzzle_string = String::new();
    let mut current_number: usize = 1;
    let mut state = "looking_for_header";
    for line in file_string.lines() {
        match state {
            "looking_for_header" => {
                if line == HEADER_START {
                    state = "looking_for_header_end";
                } else {
                    // case: no header.
                    match line.parse::<usize>() {
                        Ok(num) => {
                            if num == current_number {
                                state = "saving_puzzle";
                            }
                        },
                        Err(_) => (),
                    }
                }
            },
            "looking_for_header_end" => {
                if line == HEADER_START {
                    state = "looking_for_puzzle_number";
                }
            },
            "looking_for_puzzle_number" => {
                match line.parse::<usize>() {
                    Ok(num) => {
                        if num == current_number {
                            state = "saving_puzzle";
                        }
                    },
                    Err(_) => (),
                }
            },
            "saving_puzzle" => {
                // check if the line starts with invalid characters (must always be "Title")
                if line.len() != 0 && "# @$.+*".contains(line.chars().nth(0).unwrap()) {  
                    current_puzzle_string.push_str( &format!("{}\n", line) );
                } else {  // case: invalid line -> current puzzle is over.
                    let cur_puzzle: TileMatrix = TileMatrix::from_string(&current_puzzle_string[..]);
                    puzzles.push(cur_puzzle);

                    state = "looking_for_puzzle_number";
                    current_number += 1;
                    current_puzzle_string = String::new();
                }
            },
            _ => (),
        }
    }

    if state == "saving_puzzle" {
        let cur_puzzle: TileMatrix = puzzle_string_to_puzzle(&current_puzzle_string[..]);
        puzzles.push(cur_puzzle);
    }

    if puzzles.len() == 0 {
        println!("Error: No puzzles were found in the supplied .sok file.");
        println!("Make sure it is formatted properly with a header & numbered puzzles starting from 1.");
        process::exit(1);
    }

    if verbose {
        println!("Successfully loaded {} sokoban puzzles.", puzzles.len());
    }
    puzzles
}
