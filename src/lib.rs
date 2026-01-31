pub const COLUMN_USERNAME_SIZE: usize = 32;
pub const COLUMN_EMAIL_SIZE: usize = 255;
pub const PAGE_SIZE: usize = 4096;
pub const TABLE_MAX_PAGES: usize = 100;

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub id: u32,
    pub username: String,
    pub email: String,
}

const ID_SIZE: usize = 4;
const USERNAME_SIZE: usize = COLUMN_USERNAME_SIZE;
const EMAIL_SIZE: usize = COLUMN_EMAIL_SIZE;
const ID_OFFSET: usize = 0;
const USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
const EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;
pub const ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

pub const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
pub const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

pub struct Table {
    pub num_rows: usize,
    pages: Vec<Option<Box<[u8; PAGE_SIZE]>>>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            num_rows: 0,
            pages: vec![None; TABLE_MAX_PAGES],
        }
    }

    pub fn row_slot(&mut self, row_num: usize) -> &mut [u8] {
        let page_num = row_num / ROWS_PER_PAGE;
        let row_offset = row_num % ROWS_PER_PAGE;
        let byte_offset = row_offset * ROW_SIZE;

        if self.pages[page_num].is_none() {
            self.pages[page_num] = Some(Box::new([0; PAGE_SIZE]));
        }

        let page = self.pages[page_num].as_mut().unwrap();
        &mut page[byte_offset..byte_offset + ROW_SIZE]
    }
}

pub fn serialize_row(row: &Row, destination: &mut [u8]) {
    destination[ID_OFFSET..ID_OFFSET + ID_SIZE].copy_from_slice(&row.id.to_le_bytes());

    let mut username_bytes = [0u8; USERNAME_SIZE];
    let username_data = row.username.as_bytes();
    let username_len = username_data.len().min(USERNAME_SIZE);
    username_bytes[..username_len].copy_from_slice(&username_data[..username_len]);
    destination[USERNAME_OFFSET..USERNAME_OFFSET + USERNAME_SIZE].copy_from_slice(&username_bytes);

    let mut email_bytes = [0u8; EMAIL_SIZE];
    let email_data = row.email.as_bytes();
    let email_len = email_data.len().min(EMAIL_SIZE);
    email_bytes[..email_len].copy_from_slice(&email_data[..email_len]);
    destination[EMAIL_OFFSET..EMAIL_OFFSET + EMAIL_SIZE].copy_from_slice(&email_bytes);
}

pub fn deserialize_row(source: &[u8]) -> Row {
    let id = u32::from_le_bytes([source[0], source[1], source[2], source[3]]);

    let username_bytes = &source[USERNAME_OFFSET..USERNAME_OFFSET + USERNAME_SIZE];
    let username_end = username_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(USERNAME_SIZE);
    let username = String::from_utf8_lossy(&username_bytes[..username_end]).to_string();

    let email_bytes = &source[EMAIL_OFFSET..EMAIL_OFFSET + EMAIL_SIZE];
    let email_end = email_bytes
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(EMAIL_SIZE);
    let email = String::from_utf8_lossy(&email_bytes[..email_end]).to_string();

    Row {
        id,
        username,
        email,
    }
}

#[derive(Debug)]
pub enum StatementType {
    Insert,
    Select,
}

#[derive(Debug)]
pub struct Statement {
    pub statement_type: StatementType,
    pub row_to_insert: Option<Row>,
}

pub enum PrepareResult {
    Success(Statement),
    UnrecognizedStatement,
    SyntaxError,
    StringTooLong,
    NegativeId,
}

pub enum ExecuteResult {
    Success,
}

pub enum MetaCommandResult {
    Success,
    UnrecognizedCommand,
}

pub fn do_meta_command(input: &str) -> MetaCommandResult {
    if input == ".exit" {
        std::process::exit(0);
    } else {
        MetaCommandResult::UnrecognizedCommand
    }
}

pub fn prepare_statement(input: &str) -> PrepareResult {
    if input.starts_with("select") {
        PrepareResult::Success(Statement {
            statement_type: StatementType::Select,
            row_to_insert: None,
        })
    } else if input.starts_with("insert") {
        let parts = input.split_whitespace().collect::<Vec<_>>();

        if parts.len() != 4 {
            return PrepareResult::UnrecognizedStatement;
        }

        let id = match parts[1].parse::<u32>() {
            Ok(id) => id,
            Err(_) => return PrepareResult::SyntaxError,
        };

        if parts[2].len() > COLUMN_USERNAME_SIZE || parts[3].len() > COLUMN_EMAIL_SIZE {
            return PrepareResult::StringTooLong;
        }

        let row = Row {
            id,
            username: parts[2].to_string(),
            email: parts[3].to_string(),
        };

        PrepareResult::Success(Statement {
            statement_type: StatementType::Insert,
            row_to_insert: Some(row),
        })
    } else {
        PrepareResult::UnrecognizedStatement
    }
}

pub fn execute_statement(statement: &Statement, table: &mut Table) -> ExecuteResult {
    match statement.statement_type {
        StatementType::Insert => {
            if table.num_rows >= TABLE_MAX_ROWS {
                println!("Error: Table full.");
                return ExecuteResult::Success;
            }

            let row = statement.row_to_insert.as_ref().unwrap();
            let slot = table.row_slot(table.num_rows);
            serialize_row(row, slot);
            table.num_rows += 1;
        }
        StatementType::Select => {
            for i in 0..table.num_rows {
                let slot = table.row_slot(i);
                let row = deserialize_row(slot);
                println!("({}, {}, {})", row.id, row.username, row.email);
            }
        }
    }
    ExecuteResult::Success
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_row_serde() {
        let mut table = Table::new();

        let row = Row {
            id: 1,
            username: "john".to_string(),
            email: "john@test.com".to_string(),
        };

        let slot = table.row_slot(0);
        serialize_row(&row, slot);
        let deser_row = deserialize_row(slot);

        assert_eq!(row, deser_row);
    }
}
