#![allow(warnings)]

use vector_match::generate_match; // replace `vector_match` with your crate's name

fn main() {
    let v: Vec<u32> = vec![1, 2, 3, 4];

    generate_match! {u32,
        match v {
            [..] => {
                println!("No match");
            }
            _ => {
                println!("No match");
            }
        }
    }
}
