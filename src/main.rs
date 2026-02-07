use rsdb::{MetaCommandResult, PrepareResult};
use rsdb::{db_close, db_open, do_meta_command, execute_statement, prepare_statement, print_btree, print_constants};
use std::env;
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("must supply a database filename.");
        std::process::exit(1);
    }

    let filename = &args[1];
    let mut table = match db_open(filename) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error opening database: {}", e);
            std::process::exit(1);
        }
    };

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
                MetaCommandResult::Exit => break,
                MetaCommandResult::PrintConstants => {
                    print_constants();
                    continue;
                }
                MetaCommandResult::PrintBtree => {
                    if let Err(e) = print_btree(&mut table) {
                        println!("error: {}", e);
                    }
                    continue;
                }
                MetaCommandResult::UnrecognizedCommand => {
                    println!("unrecognized command: {}", input);
                    continue;
                }
            }
        }

        match prepare_statement(input) {
            PrepareResult::Success(statement) => match execute_statement(&statement, &mut table) {
                Ok(_) => println!("executed."),
                Err(e) => println!("error executing statement: {}", e),
            },
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

    if let Err(e) = db_close(&mut table) {
        eprintln!("error closing database: {}", e);
        std::process::exit(1);
    }
}
