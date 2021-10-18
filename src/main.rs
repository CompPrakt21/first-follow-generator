#![feature(array_windows)]

use std::path::PathBuf;

use anyhow::Context;
use structopt::StructOpt;

mod grammar;

#[derive(structopt::StructOpt)]
struct Config {
    file_name: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let config = Config::from_args();

    let content = std::fs::read_to_string(config.file_name).context("Couldn't load file.")?;

    let grammar = grammar::Grammar::parse(&content);

    grammar.integrity_check()?;

    let mut first_sets = grammar.calc_first_sets();

    println!("####################### FIRST SETS ##############################");

    for non_term in grammar.rules.keys() {
        let first_set = &first_sets[non_term];
        let mut first_set: Vec<_> = first_set.into_iter().collect();
        first_set.sort();

        let first_set: Vec<_> = first_set.into_iter().map(|s| format!("{:?}", s)).collect();

        println!("{} = {}", non_term, first_set.join(" "))
    }

    let follow_sets = grammar.calc_follow_sets(&mut first_sets);

    println!("####################### FOLLOW SETS ##############################");

    for non_term in grammar.rules.keys() {
        let follow_set = &follow_sets[non_term];
        let mut follow_set: Vec<_> = follow_set.into_iter().collect();
        follow_set.sort();

        let follow_set: Vec<_> = follow_set.into_iter().map(|s| format!("{:?}", s)).collect();

        println!("{} = {}", non_term, follow_set.join(" "))
    }

    Ok(())
}
