use std::fs::File;
use std::process;

extern crate clap;
use clap::{Arg, ArgGroup, App, SubCommand, AppSettings, ArgMatches};

mod util;
mod types;
mod level_reader;
mod ida_star_solver;

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
            .help("Only reuturns solution to puzzle"))
        .arg(Arg::with_name("INPUT")
            .required(true)
            .index(1)
            .help("Path to sokoban puzzle file to solve"))
        .subcommand(
            SubCommand::with_name("solve")
            .about("Uses IDA* to do a tree search on the problem.")
            .arg(Arg::with_name("profile")
                .long("profile")
                .help("Runs the given search through a profiler and returns a flamegraph. \
                       Note: does not work on windows-wsl version of linux."))
            .arg(Arg::with_name("deadlock-hashing")
                .long("deadlock-hashing")
                .help("Hashes deadlocked positions so that IDA* search can ignore the children deadlocked positions after secondary iterations."))
            .arg(Arg::with_name("perfect-match-greedy")
                .long("perfect-match-greedy")
                .help("This heuristic estimates the calculation of a perfect match of a bipartite graph between the goals and the crates, falling back to closest-box if neccesary. It is admissible."))
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
                .arg("perfect-match-greedy"))
        )
        .subcommand(
            SubCommand::with_name("puzzle-gen")
            .about("Generates a .sok file filled with randomly generated puzzles, many of which may be unsolvable.")
        )
        .get_matches();

    let is_silent = matches.is_present("silent");
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
    
    if let Some(matches) = matches.subcommand_matches("solve") {
        let mut deadlock_hashing: bool = false;
        if matches.is_present("deadlock-hashing") {
            deadlock_hashing = true;
        }

        if !is_dot_sok {
            do_normal_solve(puzzles.pop().unwrap(), is_silent, deadlock_hashing, matches);
        } else {
            do_batch_solve(puzzles, is_silent, deadlock_hashing, matches);
        }
    }    
}

fn do_normal_solve(puzzle: TileMatrix, is_silent: bool, deadlock_hashing: bool, matches: &ArgMatches) {
    let mut solver: Option<IDAStarSolver> = None;

    // clap assures that there will be exactly one heuristic.
    if matches.is_present("closest-box") {
        solver = Some(IDAStarSolver::new(puzzle, heuristic::closest_box, deadlock_hashing, !is_silent));
    } else if matches.is_present("goal-count") {
        solver = Some(IDAStarSolver::new(puzzle, heuristic::goal_count, deadlock_hashing, !is_silent));
    } else if matches.is_present("perfect-match-greedy") {
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

// TODO: output information nicely.
fn do_batch_solve(mut puzzles: Vec<TileMatrix>, is_silent: bool, deadlock_hashing: bool, matches: &ArgMatches) {
    let mut i = 0;
    for puzzle in puzzles.drain(..) {
        let mut solver: Option<IDAStarSolver> = None;
        if !is_silent {
            println!("======================================================");
            println!("Starting puzzle {}:", i);
            puzzle.print();
        }

        // clap assures that there will be exactly one heuristic.
        if matches.is_present("closest-box") {
            solver = Some(IDAStarSolver::new(puzzle, heuristic::closest_box, deadlock_hashing, !is_silent));
        } else if matches.is_present("goal-count") {
            solver = Some(IDAStarSolver::new(puzzle, heuristic::goal_count, deadlock_hashing, !is_silent));
        } else if matches.is_present("perfect-match-greedy") {
            solver = Some(IDAStarSolver::new(puzzle, heuristic::greedy_perfect_match, deadlock_hashing, !is_silent));
        }

        if matches.is_present("profile") {
            // Profile execution
            // when profiling, will create N flame graphs.
            let guard = pprof::ProfilerGuard::new(100).unwrap();
            execute_solver(solver, is_silent);
            if let Ok(report) = guard.report().build() {
                let file = File::create( format!("flamegraph{}.svg", i) ).unwrap();
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