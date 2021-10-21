#![feature(array_windows)]

use std::path::PathBuf;

use anyhow::Context;
use structopt::StructOpt;

extern crate tinytemplate;

use tinytemplate::TinyTemplate;
use tinytemplate::format_unescaped;
use serde::Serialize;

#[derive(Serialize)]
struct Mapping<'a> {
    nonterm: &'a String,
    result: Vec<&'a String>,
}

#[derive(Serialize)]
struct Contexta<'a>{
    firstmappings: Vec<Mapping<'a>>,
    followmappings: Vec<Mapping<'a>>,
}

mod grammar;

#[derive(structopt::StructOpt)]
struct Config {
    #[structopt(long)]
    java: bool,
    file_name: PathBuf,
}

static TEMPLATE : &'static str = include_str!("template.java");

fn main() -> anyhow::Result<()> {
    let config = Config::from_args();

    let content = std::fs::read_to_string(config.file_name).context("Couldn't load file.")?;

    let grammar = grammar::Grammar::parse(&content);

    grammar.integrity_check()?;

    let mut first_sets = grammar.calc_first_sets();
    let follow_sets = grammar.calc_follow_sets(&mut first_sets);

    if !config.java {
        println!("####################### FIRST SETS ##############################");

        for non_term in grammar.rules.keys() {
            let first_set = &first_sets[non_term];
            let mut first_set: Vec<_> = first_set.into_iter().collect();
            first_set.sort();

            let first_set: Vec<_> = first_set.into_iter().map(|s| format!("{:?}", s)).collect();

            println!("{} = {}", non_term, first_set.join(" "))
        }


        println!("####################### FOLLOW SETS ##############################");

        for non_term in grammar.rules.keys() {
            let follow_set = &follow_sets[non_term];
            let mut follow_set: Vec<_> = follow_set.into_iter().collect();
            follow_set.sort();

            let follow_set: Vec<_> = follow_set.into_iter().map(|s| format!("{:?}", s)).collect();

            println!("{} = {}", non_term, follow_set.join(" "))
        }
    } else {
        let mut tt = TinyTemplate::new();
        tt.add_template("java", TEMPLATE)?;
        tt.set_default_formatter(&format_unescaped);
        
        let mut nonterms: Vec<&String> = grammar.rules.keys().into_iter().collect();
        nonterms.sort();

        let firstmappings = nonterms.iter().map(|&nonterm| {
            let mut first_set: Vec<_> = (&first_sets[nonterm]).into_iter().collect();
            first_set.sort();
            Mapping { nonterm, result: first_set }
        }).collect();

        let followmappings = nonterms.iter().map(|&nonterm| {
            let mut result: Vec<_> = (&follow_sets[nonterm]).into_iter().collect();
            result.sort();
            Mapping { nonterm, result }
        }).collect();

        let context = Contexta {
            firstmappings,
            followmappings
        };

        let rendered = tt.render("java", &context)?;
        println!("{}", rendered);
    }

    Ok(())
}
