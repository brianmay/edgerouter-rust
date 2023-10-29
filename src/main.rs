use edgerouter_rust::{
    parser::parse_json_file,
    types::{serialize_jsonvalue, JSONValue},
};
use std::fs;

fn main() {
    let unparsed_file = fs::read_to_string("data.json").expect("cannot read file");

    let json: JSONValue = parse_json_file(&unparsed_file).expect("unsuccessful parse");

    println!("{}", serialize_jsonvalue(&json));
}
