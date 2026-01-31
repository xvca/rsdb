use std::io::{self, Write};

fn main() {
    loop {
        print!("db > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read line");

        let input = input.trim();

        match input {
            ".exit" => break,
            _ => eprintln!("unrecognized command: {}", input),
        }
    }
}
