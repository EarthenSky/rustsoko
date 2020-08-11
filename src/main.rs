use std::fs::File;
use std::process;

extern crate clap;
use clap::{Arg, ArgGroup, App, SubCommand, AppSettings, ArgMatches};

mod util;
mod types;
mod level_reader;
mod ida_star_solver;
mod level_generator;

use ida_star_solver::{IDAStarSolver, heuristic};
use types::{TileMatrix};

fn main() {
    // init cli input method
    let matches = App::new("CMPT 310 Sokoban Solver -- rustsoko")
        .version("1.0")
        .author("EarthenSky - Geb")
        .about("Implements various push optimal solving methods for sokoban puzzle.\nCan read individual puzzles & .sok files.\n\n \
                Type \"rustsoko solve --help\" or \"rustsoko puzzle-gen --help\" for more information on subcommands.")
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(Arg::with_name("silent")
            .short("s")
            .long("silent")
            .help("Returns puzzle stats in csv format -> time_elapsed,nodes_checked,solutions,pushes,moves,solution_string"))
        .subcommand(
            SubCommand::with_name("solve")
            .about("Uses IDA* to do a tree search on the problem. Puzzles will be returned 'unsolved' if they take more than 300s and don't find a solution.")
            .arg(Arg::with_name("INPUT")
                .required(true)
                .index(1)
                .help("Path to sokoban puzzle file to solve"))
            .arg(Arg::with_name("profile")
                .long("profile")
                .help("Runs the given search through a profiler and returns a flamegraph. \
                       Note: does not work on windows-wsl version of linux."))
            .arg(Arg::with_name("deadlock-hashing")
                .long("deadlock-hashing")
                .help("Hashes deadlocked positions so that IDA* search can ignore the children deadlocked positions after secondary iterations."))
            .arg(Arg::with_name("greedy-perfect-match")
                .long("greedy-perfect-match")
                .help("This heuristic estimates the calculation of a perfect match of a bipartite graph between the goals and the crates, falling back to closest-box for initially unsatisfied nodes. It is admissible."))
            .arg(Arg::with_name("closest-box")
                .long("closest-box")
                .help("This heuristic is the distance from the closest box to the goal. It is admissible."))
            .arg(Arg::with_name("goal-count")
                .long("goal-count")
                .help("This heuristic uses the number of goals which are not covered, which is faster but also less intelligent. It is admissible."))
            .group(ArgGroup::with_name("heuristic")
                .required(true)
                .arg("closest-box")
                .arg("goal-count")
                .arg("greedy-perfect-match"))
        )
        .subcommand(
            SubCommand::with_name("puzzle-gen")
            .about("Generates a .sok file filled with randomly generated puzzles, many of which may be unsolvable.")
            .arg(Arg::with_name("OUTPUT")
                .required(true)
                .index(1)
                .help("filename of .sok file. Extension is added automatically. ex: \"test_files\" -> test_files.sok"))
            .arg(Arg::with_name("width")
                .required(true)
                .index(2))
            .arg(Arg::with_name("height")
                .required(true)
                .index(3))
            .arg(Arg::with_name("batch-size")
                .required(true)
                .index(4)
                .help("How many puzzles to put in the .sok file"))
            .arg(Arg::with_name("goal-number")
                .required(true)
                .index(5)
                .help("How many goals to include in the puzzle"))
            .arg(Arg::with_name("wall-number")
                .required(true)
                .index(6)
                .help("How many walls to include in the puzzle"))
        )
        .get_matches();

    let is_silent = matches.is_present("silent");
    
    if let Some(matches) = matches.subcommand_matches("solve") {
        let filepath = matches.value_of("INPUT").unwrap();

        // Load file
        let mut puzzles: Vec<TileMatrix>;
        let mut is_dot_sok = false;
        match level_reader::get_extension_from_filename(filepath) {
            Some(s) => if s == "sok" {
                puzzles = level_reader::read_sok(filepath, !is_silent);
                is_dot_sok = true;
            } else {  // any other file extension -- like .txt -- is allowed
                puzzles = vec![level_reader::read_puzzle(filepath, !is_silent)];
            },
            None => {
                puzzles = vec![level_reader::read_puzzle(filepath, !is_silent)];
            },
        } 

        let mut deadlock_hashing: bool = false;
        if matches.is_present("deadlock-hashing") {
            deadlock_hashing = true;
        }

        if !is_dot_sok {
            do_normal_solve(puzzles.pop().unwrap(), is_silent, deadlock_hashing, matches);
        } else {
            do_batch_solve(puzzles, is_silent, deadlock_hashing, matches);
        }
    } else if let Some(matches) = matches.subcommand_matches("puzzle-gen") { 
        let file_name = matches.value_of("OUTPUT").unwrap();
        if file_name.contains("/") {
            println!("Command Error: invalid filename -> cannot contain /");
            process::exit(1);
        }
        let width = usize_parse( matches.value_of("width").unwrap(), "width" );
        let height = usize_parse( matches.value_of("height").unwrap(), "height" );
        let batch_num = usize_parse( matches.value_of("batch-size").unwrap(), "batch-size" );
        let goal_num = usize_parse( matches.value_of("goal-number").unwrap(), "goal-number" );
        let wall_num = usize_parse( matches.value_of("wall-number").unwrap(), "wall-number" );

        if width <= 3 {
            println!("Command Error: width too small -> width must be 4 or larger");
            process::exit(1);
        } else if height <= 3 {
            println!("Command Error: height too small -> width must be 4 or larger");
            process::exit(1);
        } else if batch_num == 0 {
            println!("Command Error: batch-size must be non-zero");
            process::exit(1);
        } else if goal_num + wall_num > ((width * height) - ((2*width + 2*height) - 4)) / 2 {
            println!("Command Error: too many goal & wall spaces. goals + walls must be less than empty_spaces / 2.");
            process::exit(1);
        }

        level_generator::make_sok(file_name, width, height, batch_num, goal_num, wall_num);
    }
}

fn do_normal_solve(puzzle: TileMatrix, is_silent: bool, deadlock_hashing: bool, matches: &ArgMatches) {
    let mut solver: Option<IDAStarSolver> = None;

    // clap assures that there will be exactly one heuristic.
    if matches.is_present("closest-box") {
        solver = Some(IDAStarSolver::new(puzzle, heuristic::closest_box, deadlock_hashing, !is_silent));
    } else if matches.is_present("goal-count") {
        solver = Some(IDAStarSolver::new(puzzle, heuristic::goal_count, deadlock_hashing, !is_silent));
    } else if matches.is_present("greedy-perfect-match") {
        solver = Some(IDAStarSolver::new(puzzle, heuristic::greedy_perfect_match, deadlock_hashing, !is_silent));
    }

    if matches.is_present("profile") {
        // Profile execution
        let guard = pprof::ProfilerGuard::new(100).unwrap();
        execute_solver(solver, is_silent);
        if let Ok(report) = guard.report().build() {
            let file = File::create("flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
            println!("Flamegraph Generated");
        };

    } else {
        execute_solver(solver, is_silent);
    }
}

fn do_batch_solve(mut puzzles: Vec<TileMatrix>, is_silent: bool, deadlock_hashing: bool, matches: &ArgMatches) {
    let mut i = 0;
    for puzzle in puzzles.drain(..) {
        let mut solver: Option<IDAStarSolver> = None;
        if !is_silent {
            println!("======================================================");
            println!("Starting puzzle {}:", i+1);
            puzzle.print();
        }

        // clap assures that there will be exactly one heuristic.
        if matches.is_present("closest-box") {
            solver = Some(IDAStarSolver::new(puzzle, heuristic::closest_box, deadlock_hashing, !is_silent));
        } else if matches.is_present("goal-count") {
            solver = Some(IDAStarSolver::new(puzzle, heuristic::goal_count, deadlock_hashing, !is_silent));
        } else if matches.is_present("greedy-perfect-match") {
            solver = Some(IDAStarSolver::new(puzzle, heuristic::greedy_perfect_match, deadlock_hashing, !is_silent));
        }

        if matches.is_present("profile") {
            // Profile execution
            // when profiling, will create N flame graphs.
            let guard = pprof::ProfilerGuard::new(100).unwrap();
            execute_solver(solver, is_silent);
            if let Ok(report) = guard.report().build() {
                let file = File::create( format!("flamegraph{}.svg", i+1) ).unwrap();
                report.flamegraph(file).unwrap();
                println!("Flamegraph Generated");
            };

        } else {
            execute_solver(solver, is_silent);
        }

        i += 1;
    }
}

fn execute_solver(solver: Option<IDAStarSolver>, is_silent: bool) {
    if let Some(mut s) = solver {
        let solution = s.solve();
        if !is_silent {
            print!("Optimal solution is: ");
        }
        println!("{}", solution);
    } else {
        println!("Command Error: A heuristic must be stated. ex: --closest-box");
        process::exit(1);
    }
}

fn usize_parse(s: &str, error_kind: &str) -> usize {
    match s.parse::<usize>() {
        Ok(num) => num,
        Err(_) => {
            println!("Command Error: invalid {} -> must be integer", error_kind);
            process::exit(1);
        }
    }
}