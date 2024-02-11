use std::{fs::File, io::Write};

use twee_parser::*;


pub fn main() {
    let mut story = parse_twee3(include_str!("../test-data/Test Story.twee")).unwrap().0;
    story.title = "My Story".to_string();
    File::create("example1.twee").unwrap().write_all(serialize_twee3(&story).as_bytes()).unwrap();
}
