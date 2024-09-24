#![allow(warnings)]

use vector_match::generate_match; // replace `vector_match` with your crate's name

fn main() {
    let v = vec!['a', 'b', 'c', 'd'];

    generate_match! {
        match v {
            [1, 2] => {
                println!("No match");
            }
            _ => {
                println!("No match");
            }
            _ => {
                println!("No match");
            }
        }
    }
}
