use std::fs::File;

extern crate clap;
use clap::{Arg, ArgGroup, App, SubCommand, AppSettings};

mod util;
mod types;
mod level_reader;
mod ida_star_solver;

use ida_star_solver::{IDAStarSolver, heuristic};

fn main() {
    // init cli input method
    let matches = App::new("CMPT 310 Sokoban Solver -- sokosolver")
        .version("1.0")
        .author("EarthenSky - Geb")
        .about("Implements various push optimal solving methods for sokoban puzzle")
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(Arg::with_name("silent")
            .short("s")
            .long("silent")
            .help("Only reuturns runtime statistics in csv format. ## TODO: this"))
        .arg(Arg::with_name("INPUT")
            .required(true)
            .index(1)
            .help("Path to sokoban puzzle file to solve"))
        .subcommand(
            SubCommand::with_name("idas")
            .about("Uses the IDA* algorithm to do a tree search on the problem")
            .arg(Arg::with_name("profile")
                .long("profile")
                .help("Runs the given search through a profiler and returns a flamegraph."))
            .arg(Arg::with_name("closest-box")
                .long("closest-box")
                .help("This heuristic is the distance from the closest box to the goal. It is admissible."))
            .group(ArgGroup::with_name("heuristic")
                .required(true)
                .arg("closest-box"))
        )
        .get_matches();

    let is_silent = matches.is_present("silent");
    let filepath = matches.value_of("INPUT").unwrap();
    let puzzle = level_reader::read_puzzle(filepath, !is_silent);

    if let Some(matches) = matches.subcommand_matches("idas") {
        let mut solver: Option<IDAStarSolver> = None;

        // clap assures that there will be exactly one heuristic.
        if matches.is_present("closest-box") {
            // add the cloest-box heuristic
            solver = Some(IDAStarSolver::new(puzzle, heuristic::closest_box, !is_silent));
        }

        if matches.is_present("profile") {
            // Profiling execution
            let guard = pprof::ProfilerGuard::new(100).unwrap();
            if let Some(mut s) = solver {
                let solution = s.solve();
                if !is_silent {
                    print!("Optimal solution is: ");
                }
                println!("{}", solution);
            } else {
                println!("Command Error: A heuristic must be stated. ex: --closest-box");
            }
            if let Ok(report) = guard.report().build() {
                let file = File::create("flamegraph.svg").unwrap();
                report.flamegraph(file).unwrap();

                println!("report: {}", &report);
            };

        } else {
            // Regular execution
            if let Some(mut s) = solver {
                let solution = s.solve();
                if !is_silent {
                    print!("Optimal solution is: ");
                }
                println!("{}", solution);
            } else {
                println!("Command Error: A heuristic must be stated. ex: --closest-box");
            }
        }
    }    
}