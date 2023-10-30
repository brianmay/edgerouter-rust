//! Test program for edgerouter-rust
#![warn(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::unwrap_used)]
// #![deny(clippy::expect_used)]

use edgerouter_rust::{parser::parse_file, types::File};
use std::fs;

fn main() {
    let unparsed_file = fs::read_to_string("config.boot").expect("cannot read file");

    let json: File = parse_file(&unparsed_file).expect("unsuccessful parse");

    println!("{}", json.serialize());
}
