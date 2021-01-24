use byteorder::{BigEndian, ByteOrder, LittleEndian};
use pager::{Page, Pager, PAGE_SIZE};
use prettytable::{Cell, Row as PrettyRow, Table as PrettyTable};
use std::ops::{Index, Range};
const ROW_SIZE: usize = 4 + 32 + 256;
const ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;

const NODE_TYPE_SIZE: usize = 1;
const NODE_TYPE_OFFSET: usize = 0;
const IS_ROOT_SIZE: usize = 1;
const IS_ROOT_OFFSET: usize = NODE_TYPE_SIZE;
const PARENT_POINTER_SIZE: usize = 4;
const PARENT_POINTER_OFFSET: usize = IS_ROOT_OFFSET + IS_ROOT_SIZE;
const COMMON_NODE_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;

// Leaf node header layout
const LEAF_NODE_NUM_CELLS_SIZE: usize = 4;
const LEAF_NODE_NUM_CELLS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const LEAF_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + LEAF_NODE_NUM_CELLS_SIZE;

// Lead node body layout
const LEAF_NODE_KEY_SIZE: usize = 4;
const LEAF_NODE_KEY_OFFSET: usize = 0;
const LEAF_NODE_VALUE_SIZE: usize = ROW_SIZE;
const LEAF_NODE_VALUE_OFFSET: usize = LEAF_NODE_KEY_OFFSET + LEAF_NODE_KEY_SIZE;
const LEAF_NODE_CELL_SIZE: usize = LEAF_NODE_KEY_SIZE + LEAF_NODE_VALUE_SIZE;
const LEAF_NODE_SPACE_FOR_CELLS: usize = PAGE_SIZE - LEAF_NODE_HEADER_SIZE;
const LEAF_NODE_MAX_CELLS: usize = LEAF_NODE_SPACE_FOR_CELLS / LEAF_NODE_CELL_SIZE;

pub const TABLE_MAX_PAGES: usize = 100;
pub const TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;

pub enum NodeType {
    NodeInternal,
    NodeLeaf,
}

pub fn leaf_node_num_cells(node: &Page) -> u32 {
    return LittleEndian::read_u32(node.index(Range {
        start: LEAF_NODE_NUM_CELLS_OFFSET,
        end: LEAF_NODE_NUM_CELLS_OFFSET + LEAF_NODE_NUM_CELLS_SIZE,
    }));
}

pub fn leaf_node_cell(
    node_from: &Page,
    from_cell_num: usize,
    node_to: &mut Page,
    to_cell_num: usize,
) {
    let from_offset: usize = LEAF_NODE_HEADER_SIZE + (from_cell_num * LEAF_NODE_CELL_SIZE);
    let to_offset: usize = LEAF_NODE_HEADER_SIZE + (to_cell_num * LEAF_NODE_CELL_SIZE);
    for i in 0..LEAF_NODE_CELL_SIZE {
        node_to[i + to_offset] = node_from[i + from_offset]
    }
}

pub fn leaf_node_value(node: &Page, cell_num: usize) -> Vec<u8> {
    let offset: usize =
        LEAF_NODE_HEADER_SIZE + (cell_num * LEAF_NODE_CELL_SIZE) + LEAF_NODE_KEY_SIZE;
    let mut bytes: Vec<u8> = vec![0; LEAF_NODE_VALUE_SIZE];
    bytes.clone_from_slice(node.index(Range {
        start: offset,
        end: offset + LEAF_NODE_VALUE_SIZE,
    }));
    return bytes;
}

pub fn leaf_node_key(node: &Page, cell_num: u32) -> u32 {
    let offset = LEAF_NODE_HEADER_SIZE + LEAF_NODE_CELL_SIZE * (cell_num as usize);
    let mut bytes = vec![0; LEAF_NODE_KEY_SIZE];

    bytes.clone_from_slice(node.index(Range {
        start: offset,
        end: offset + LEAF_NODE_KEY_SIZE,
    }));

    return LittleEndian::read_u32(bytes.as_slice());
}

// Cursor abstraction that points to the end-of and start-of the Table.
pub struct Cursor<'a> {
    pub table: &'a mut Table,
    pub row_num: usize,
    pub end_of_table: bool, // Indicates a position one past the last element
}

#[allow(unused_mut, unused_variables, dead_code)]
impl<'a> Cursor<'a> {
    pub fn is_end_of_table(&self) -> bool {
        self.table.num_row == 0
    }
    pub fn end_of_table(&mut self) -> Cursor {
        let cursor = Cursor {
            table: self.table,
            row_num: 0,
            end_of_table: false,
        };

        cursor
    }

    pub fn table_end(&self) -> bool {
        self.row_num >= self.table.num_row
    }

    pub fn cursor_advance(&mut self) {
        self.row_num += 1;
        if self.table_end() {
            self.end_of_table = true;
        }
    }

    pub fn get_row(&mut self) -> Row {
        let value = self.cursor_value();

        Row::deserialize(value)
    }

    pub fn cursor_value(&mut self) -> Vec<u8> {
        let row_num = self.row_num as u32;
        let page_num = row_num / ROWS_PER_PAGE as u32;
        // NODE
        let page: &mut Page = self.table.pager.page_to_write(page_num as usize);

        return return_cursor_value(&page, page_num as usize, row_num);
    }
}

pub fn return_cursor_value(page: &Page, cell_num: usize, row_num: u32) -> Vec<u8> {
    let row_offset: u32 = row_num % ROWS_PER_PAGE as u32;
    let mut byte_offset: Vec<u8> = vec![0; ROW_SIZE];

    byte_offset.clone_from_slice(page.index(Range {
        start: row_offset as usize,
        end: (row_offset + ROW_SIZE as u32) as usize,
    }));

    return byte_offset;
}
pub struct Table {
    pub pager: Pager,
    pub num_row: usize,
}

impl Table {
    pub fn new() -> Table {
        let pager = Pager::new();
        // let num_row = (pager.file.metadata().unwrap().len() / ROW_SIZE as u64) as usize;
        let num_row = if pager.num_pages == 0 {
            0
        } else {
            if pager.num_pages == 1 {
                1
            } else {
                1000
            }
        };

        Table { pager, num_row }
    }

    pub fn table_start(&mut self) -> Cursor {
        let cell_num = 0;
        let cursor = Cursor {
            table: self,
            row_num: 0,
            end_of_table: cell_num == 0,
        };

        cursor
    }

    pub fn close(&mut self) {
        self.pager.close();
    }

    pub fn insert_row(&mut self, row: Row) {
        let bytes = row.serialize();

        let page_index = self.num_row / ROWS_PER_PAGE;
        let page = self.pager.page_to_write(page_index);
        let row_index = (self.num_row % ROWS_PER_PAGE) * ROW_SIZE;
        for (i, byte) in bytes.into_iter().enumerate() {
            page[row_index + i] = byte;
        }
        self.num_row += 1;
    }

    pub fn read_row(&mut self, num_row: usize) -> Row {
        let page_index = num_row / ROWS_PER_PAGE;
        let page = self.pager.page_to_read(page_index);
        let row_index = (num_row % ROWS_PER_PAGE) * ROW_SIZE;
        let bytes = page[row_index..row_index + ROW_SIZE].to_vec();

        Row::deserialize(bytes)
    }

    pub fn print_table(&mut self) {
        let mut table = PrettyTable::new();
        table.add_row(PrettyRow::new(vec![
            Cell::new("ID"),
            Cell::new("USERNAME"),
            Cell::new("EMAIL"),
        ]));

        table.printstd();
        for i in 0..self.num_row {
            let row = self.read_row(i);
            row.print_row();
        }
    }
}

pub struct Row {
    pub id: u32,
    pub username: String,
    pub email: String,
}

impl Row {
    fn print_row(&self) {
        println!("| {} | {} | {} |", self.id, self.username, self.email);
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buf = vec![0; ROW_SIZE];
        BigEndian::write_u32(&mut buf, self.id);
        Row::write_string(&mut buf, 4, 32, &self.username);
        Row::write_string(&mut buf, 36, 256, &self.email);

        buf
    }

    fn deserialize(buf: Vec<u8>) -> Row {
        let id = BigEndian::read_u32(&buf);
        let username = Row::read_string(&buf, 4, 32);
        let email = Row::read_string(&buf, 36, 256);
        Row {
            id,
            username,
            email,
        }
    }

    fn write_string(buf: &mut Vec<u8>, pos: usize, max_len: usize, s: &String) {
        let bytes = s.as_bytes().to_owned();

        let mut i = 0;
        for b in bytes {
            buf[pos + i] = b;
            i += 1;
        }
        while i < max_len {
            buf[pos + i] = 0;
            i += 1;
        }
    }

    fn read_string(buf: &Vec<u8>, pos: usize, max_len: usize) -> String {
        let mut end = pos;
        while end < max_len + pos && buf[end] != 0 {
            end += 1;
        }
        let bytes = buf[pos..end].to_vec();

        String::from_utf8(bytes).unwrap()
    }
}
