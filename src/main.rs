mod parser;

use parser::parse_args;

fn main() {
    let settings = parse_args();
    println!("Parsed settings:\n{:#?}", settings);
}