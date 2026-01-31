use rsdb::{do_meta_command, execute_statement, prepare_statement};
use rsdb::{MetaCommandResult, PrepareResult, Table};
use std::io::{self, Write};

fn main() {
    let mut table = Table::new();

    loop {
        print!("db > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("failed to read line");

        let input = input.trim();

        if input.starts_with('.') {
            match do_meta_command(input) {
                MetaCommandResult::Success => continue,
                MetaCommandResult::UnrecognizedCommand => {
                    println!("unrecognized command: {}", input);
                    continue;
                }
            }
        }

        match prepare_statement(input) {
            PrepareResult::Success(statement) => {
                execute_statement(&statement, &mut table);
                println!("executed.");
            }
            PrepareResult::UnrecognizedStatement => {
                println!("unrecognized keyword at start of '{}'.", input);
            }
            PrepareResult::SyntaxError => {
                println!("syntax error. could not parse statement.");
            }
            PrepareResult::StringTooLong => {
                println!("string is too long.");
            }
            PrepareResult::NegativeId => {
                println!("id must be positive.");
            }
        }
    }
}
