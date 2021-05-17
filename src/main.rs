use fundot::evaluator::Evaluator;
use fundot::object::Object;
use std::io::{self, prelude::*};

fn main() {
    let evaluator = Evaluator::new();
    loop {
        let mut input = String::new();
        print!(">>> ");
        io::stdout().flush().expect("Failed to flush output");
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let obj = input
            .parse::<Object>()
            .expect("Failed to parse string as object");
        println!("{}", evaluator.eval(&obj));
    }
}
