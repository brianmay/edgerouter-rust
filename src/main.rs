use edgerouter_rust::{parser::parse_file, types::File};
use std::fs;

fn main() {
    let unparsed_file = fs::read_to_string("config.boot").expect("cannot read file");

    let json: File = parse_file(&unparsed_file).expect("unsuccessful parse");

    println!("{}", json.serialize());
}
