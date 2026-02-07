use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom, Write};

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

pub const ROOT_PAGE_NUM: usize = 0;

// node page layout:
//   [0]     node_type   (1 byte)
//   [1]     is_root     (1 byte)
//   [2..5]  parent_ptr  (4 bytes)
//   [6..9]  num_cells   (4 bytes, leaf only)
//   [10..]  cells       (key + value each)
const NODE_TYPE_SIZE: usize = 1;
const NODE_TYPE_OFFSET: usize = 0;
const IS_ROOT_SIZE: usize = 1;
const IS_ROOT_OFFSET: usize = NODE_TYPE_SIZE;
const PARENT_POINTER_SIZE: usize = 4;
const COMMON_NODE_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;

const LEAF_NODE_NUM_CELLS_SIZE: usize = 4;
const LEAF_NODE_NUM_CELLS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const LEAF_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + LEAF_NODE_NUM_CELLS_SIZE;

const LEAF_NODE_KEY_SIZE: usize = 4;
const LEAF_NODE_VALUE_OFFSET: usize = LEAF_NODE_KEY_SIZE;
const LEAF_NODE_VALUE_SIZE: usize = ROW_SIZE;
const LEAF_NODE_CELL_SIZE: usize = LEAF_NODE_KEY_SIZE + LEAF_NODE_VALUE_SIZE;
pub const LEAF_NODE_MAX_CELLS: usize = (PAGE_SIZE - LEAF_NODE_HEADER_SIZE) / LEAF_NODE_CELL_SIZE;

#[allow(dead_code)]
enum NodeType {
    Leaf,
    Internal,
}

pub struct Pager {
    file: File,
    file_length: u64,
    num_pages: usize,
    pages: Vec<Option<Box<[u8; PAGE_SIZE]>>>,
}

impl Pager {
    fn new(filename: &str) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(filename)?;

        let file_length = file.metadata()?.len();

        if file_length != 0 && file_length % PAGE_SIZE as u64 != 0 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "db file is not a whole number of pages",
            ));
        }

        let num_pages = file_length / PAGE_SIZE as u64;

        Ok(Pager {
            file,
            file_length,
            num_pages: num_pages as usize,
            pages: vec![None; TABLE_MAX_PAGES],
        })
    }

    pub fn get_page(&mut self, page_num: usize) -> Result<&mut [u8; PAGE_SIZE]> {
        if self.pages[page_num].is_none() {
            let mut page = Box::new([0; PAGE_SIZE]);

            if (page_num as u64) < (self.file_length / PAGE_SIZE as u64) {
                self.file
                    .seek(SeekFrom::Start((page_num * PAGE_SIZE) as u64))?;
                self.file.read_exact(&mut page[..])?;
            }
            self.pages[page_num] = Some(page);

            if page_num >= self.num_pages {
                self.num_pages = page_num + 1;
            }
        }

        Ok(self.pages[page_num].as_mut().unwrap())
    }

    fn flush(&mut self, page_num: usize) -> Result<()> {
        if let Some(page) = &self.pages[page_num] {
            self.file
                .seek(SeekFrom::Start((page_num * PAGE_SIZE) as u64))?;
            self.file.write_all(&page[..])?;
        }
        Ok(())
    }
}

pub struct Table {
    pub root_page_num: usize,
    pub pager: Pager,
}

pub struct Cursor<'a> {
    pub table: &'a mut Table,
    pub page_num: usize,
    pub cell_num: usize,
    pub end_of_table: bool,
}

impl<'a> Cursor<'a> {
    pub fn table_start(table: &'a mut Table) -> Result<Self> {
        let page_num = table.root_page_num;
        let num_cells = {
            let root_page = table.pager.get_page(table.root_page_num)?;
            leaf_node_num_cells(root_page)
        };

        Ok(Cursor {
            table,
            page_num,
            cell_num: 0,
            end_of_table: num_cells == 0,
        })
    }

    pub fn table_end(table: &'a mut Table) -> Result<Self> {
        let page_num = table.root_page_num;
        let num_cells = {
            let root_page = table.pager.get_page(table.root_page_num)?;
            leaf_node_num_cells(root_page)
        };

        Ok(Cursor {
            table,
            page_num,
            cell_num: num_cells as usize,
            end_of_table: true,
        })
    }

    pub fn value(&mut self) -> Result<&mut [u8]> {
        let page = self.table.pager.get_page(self.page_num)?;
        Ok(leaf_node_value(page, self.cell_num as u32))
    }

    pub fn advance(&mut self) -> Result<()> {
        self.cell_num += 1;

        let num_cells = {
            let page = self.table.pager.get_page(self.page_num)?;
            leaf_node_num_cells(page)
        };

        if self.cell_num >= num_cells as usize {
            self.end_of_table = true;
        }

        Ok(())
    }
}

pub fn db_open(filename: &str) -> Result<Table> {
    let mut pager = Pager::new(filename)?;

    if pager.num_pages == 0 {
        let page = pager.get_page(0)?;
        initialize_leaf_node(page);
    }

    Ok(Table {
        root_page_num: ROOT_PAGE_NUM,
        pager,
    })
}

pub fn db_close(table: &mut Table) -> Result<()> {
    for i in 0..table.pager.num_pages {
        table.pager.flush(i)?;
    }

    Ok(())
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

// --- leaf node accessors ---

fn leaf_node_num_cells(node: &[u8; PAGE_SIZE]) -> u32 {
    u32::from_le_bytes(
        node[LEAF_NODE_NUM_CELLS_OFFSET..LEAF_NODE_NUM_CELLS_OFFSET + LEAF_NODE_NUM_CELLS_SIZE]
            .try_into()
            .unwrap(),
    )
}

fn set_leaf_node_num_cells(node: &mut [u8; PAGE_SIZE], num_cells: u32) {
    node[LEAF_NODE_NUM_CELLS_OFFSET..LEAF_NODE_NUM_CELLS_OFFSET + LEAF_NODE_NUM_CELLS_SIZE]
        .copy_from_slice(&num_cells.to_le_bytes());
}

fn leaf_node_cell_offset(cell_num: u32) -> usize {
    LEAF_NODE_HEADER_SIZE + (cell_num as usize * LEAF_NODE_CELL_SIZE)
}

fn leaf_node_cell(node: &mut [u8; PAGE_SIZE], cell_num: u32) -> &mut [u8] {
    let offset = leaf_node_cell_offset(cell_num);
    &mut node[offset..offset + LEAF_NODE_CELL_SIZE]
}

fn leaf_node_key(node: &[u8; PAGE_SIZE], cell_num: u32) -> u32 {
    let offset = leaf_node_cell_offset(cell_num);
    let cell = &node[offset..offset + LEAF_NODE_CELL_SIZE];

    u32::from_le_bytes(cell[..LEAF_NODE_KEY_SIZE].try_into().unwrap())
}

fn leaf_node_value(node: &mut [u8; PAGE_SIZE], cell_num: u32) -> &mut [u8] {
    let offset = leaf_node_cell_offset(cell_num) + LEAF_NODE_VALUE_OFFSET;
    &mut node[offset..offset + LEAF_NODE_VALUE_SIZE]
}

fn initialize_leaf_node(node: &mut [u8; PAGE_SIZE]) {
    node[NODE_TYPE_OFFSET] = NodeType::Leaf as u8;
    node[IS_ROOT_OFFSET] = 1;
    set_leaf_node_num_cells(node, 0);
}

// insert a cell at the end of the leaf node: write key, serialize value, bump num_cells
pub fn leaf_node_insert(cursor: &mut Cursor, key: u32, value: &Row) -> Result<()> {
    let page = cursor.table.pager.get_page(cursor.page_num)?;
    let num_cells = leaf_node_num_cells(page);
    let cell = leaf_node_cell(page, cursor.cell_num as u32);

    cell[..LEAF_NODE_KEY_SIZE].copy_from_slice(&key.to_le_bytes());
    serialize_row(value, &mut cell[LEAF_NODE_VALUE_OFFSET..]);

    set_leaf_node_num_cells(page, num_cells + 1);

    Ok(())
}

// --- debug meta commands ---

pub fn print_constants() {
    println!("ROW_SIZE: {}", ROW_SIZE);
    println!("LEAF_NODE_HEADER_SIZE: {}", LEAF_NODE_HEADER_SIZE);
    println!("LEAF_NODE_CELL_SIZE: {}", LEAF_NODE_CELL_SIZE);
    println!("PAGE_SIZE: {}", PAGE_SIZE);
    println!(
        "LEAF_NODE_SPACE_FOR_CELLS: {}",
        PAGE_SIZE - LEAF_NODE_HEADER_SIZE
    );
    println!("LEAF_NODE_MAX_CELLS: {}", LEAF_NODE_MAX_CELLS);
}

pub fn print_btree(table: &mut Table) -> Result<()> {
    let root = table.pager.get_page(table.root_page_num)?;
    print_leaf_node(root);
    Ok(())
}

fn print_leaf_node(node: &mut [u8; PAGE_SIZE]) {
    let num_cells = leaf_node_num_cells(node);
    println!("*---*");
    for i in 0..num_cells {
        let key = leaf_node_key(node, i);
        println!("  - {}: {}", i, key);
    }
    println!("*---*");
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
    Exit,
    PrintConstants,
    PrintBtree,
    UnrecognizedCommand,
}

pub fn do_meta_command(input: &str) -> MetaCommandResult {
    match input {
        ".exit" => MetaCommandResult::Exit,
        ".constants" => MetaCommandResult::PrintConstants,
        ".btree" => MetaCommandResult::PrintBtree,
        _ => MetaCommandResult::UnrecognizedCommand,
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

pub fn execute_statement(statement: &Statement, table: &mut Table) -> Result<ExecuteResult> {
    match statement.statement_type {
        StatementType::Insert => {
            let num_cells = {
                let page = table.pager.get_page(table.root_page_num)?;
                leaf_node_num_cells(page)
            };

            if num_cells >= LEAF_NODE_MAX_CELLS as u32 {
                println!("Error: leaf node full.");
                return Ok(ExecuteResult::Success);
            }

            let row = statement.row_to_insert.as_ref().unwrap();
            let mut cursor = Cursor::table_end(table)?;
            leaf_node_insert(&mut cursor, row.id, row)?;
        }
        StatementType::Select => {
            let mut cursor = Cursor::table_start(table)?;
            while !cursor.end_of_table {
                let slot = cursor.value()?;
                let row = deserialize_row(slot);

                println!("({}, {}, {})", row.id, row.username, row.email);

                cursor.advance()?;
            }
        }
    }
    Ok(ExecuteResult::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_serialization() {
        let row = Row {
            id: 1,
            username: "john".to_string(),
            email: "john@test.com".to_string(),
        };

        let mut buffer = [0u8; ROW_SIZE];
        serialize_row(&row, &mut buffer);
        let deser_row = deserialize_row(&buffer);

        assert_eq!(row, deser_row);
    }
}
