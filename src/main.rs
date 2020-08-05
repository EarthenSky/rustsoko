extern crate clap;
use clap::{Arg, ArgGroup, App, SubCommand, AppSettings};

mod types;
mod level_reader;

fn main() {
    // init cli input method
    let matches = App::new("CMPT 310 Sokoban Solver -- sokosolver")
        .version("1.0")
        .author("Geb")
        .about("Implements various push optimal solving methods for sokoban puzzle")
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("Shows debug information ## TODO"))
        .arg(Arg::with_name("INPUT")
            .required(true)
            .index(1)
            .help("Path to sokoban puzzle file to solve"))
        .subcommand(
            SubCommand::with_name("idas")
            .about("Uses the IDA* algorithm to do a tree search on the problem")
            .arg(Arg::with_name("closest-box")
                .long("closest-box")
                .help("This heuristic is the distance from the closest box to the goal. It is admissible."))
            .group(ArgGroup::with_name("heuristic")
                .required(true)
                .arg("closest-box"))
        )
        .get_matches();

    let filepath = matches.value_of("INPUT").unwrap();
    let puzzle = level_reader::read_puzzle(filepath, true);

    if let Some(matches) = matches.subcommand_matches("idas") {
        // TODO: init IDA*

        // configure heuristic
        if matches.is_present("closest-box") {
            // TODO: add the cloest-box heuristic

        }
    }    

}