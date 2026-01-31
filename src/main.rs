use std::io::{self, Write};

#[derive(Debug)]
enum StatementType {
    Insert,
    Select,
}

#[derive(Debug)]
struct Statement {
    statement_type: StatementType,
}

enum PrepareResult {
    Success(Statement),
    UnrecognizedStatement,
}

enum ExecuteResult {
    Success,
}

enum MetaCommandResult {
    Success,
    UnrecognizedCommand,
}

fn do_meta_command(input: &str) -> MetaCommandResult {
    if input == ".exit" {
        std::process::exit(0);
    } else {
        MetaCommandResult::UnrecognizedCommand
    }
}

fn prepare_statement(input: &str) -> PrepareResult {
    if input.starts_with("select") {
        PrepareResult::Success(Statement {
            statement_type: StatementType::Select,
        })
    } else if input.starts_with("insert") {
        PrepareResult::Success(Statement {
            statement_type: StatementType::Insert,
        })
    } else {
        PrepareResult::UnrecognizedStatement
    }
}

fn execute_statement(statement: &Statement) -> ExecuteResult {
    match statement.statement_type {
        StatementType::Insert => {
            println!("run insert here");
        }
        StatementType::Select => {
            println!("run select here");
        }
    }
    ExecuteResult::Success
}

fn main() {
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
                execute_statement(&statement);
                println!("executed.");
            }
            PrepareResult::UnrecognizedStatement => {
                println!("unrecognized keyword at start of '{}'.", input);
            }
        }
    }
}
