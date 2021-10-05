extern crate clap;
use clap::{Arg, App};

mod term;
mod net;

use term::*;

use std::io;
use std::io::prelude::*;
use std::fs::File;

fn main() -> io::Result<()> {
    let matches = App::new("Symmetric Interaction Calculus")
        .version("0.1.0")
        .author("Victor Maia <srvictormaia@gmail.com>")
        .about("Evaluates SIC programs")
        .arg(Arg::with_name("INPUT")
            .short("i")
            .long("input")
            .value_name("INPUT")
            .help("Input term")
            .takes_value(true))
        .arg(Arg::with_name("STATS")
            .short("s")
            .long("stats")
            .value_name("STATS")
            .help("Show stats")
            .takes_value(false))
        .arg(Arg::with_name("FILE")
            .help("Sets the input file to use")
            .required(true)
            .index(1))
        .get_matches();

    let file_name = matches.value_of("FILE").unwrap();
    let mut file = File::open(file_name)?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;

    let input : Option<Vec<u8>> = match matches.value_of("INPUT") {
            Some(term) => Some(term.as_bytes().to_vec()),
            None => None
    };

    match input {
        Some(mut input) => {
            code.extend_from_slice(b"\n:main ");
            code.append(&mut input);
        },
        None => {}
    }

    let term = from_string(&code);
    let mut net = to_net(&term);
    let stats = net::reduce(&mut net);
    let norm = from_net(&net);

    let output = to_string(&norm);

    println!("{}", String::from_utf8_lossy(&output));

    if matches.is_present("STATS") {
        println!("{:?}", stats);
    }

    Ok(())
}
