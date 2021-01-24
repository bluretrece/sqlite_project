use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use db::TABLE_MAX_PAGES;

pub const PAGE_SIZE: usize = 4096;

pub type Page = Vec<u8>;

#[derive(Debug)]
pub struct Pager {
    pub file: File,
    pub file_length: usize,
    pub pages: Vec<Option<Page>>,
    pub num_pages: usize,
}

impl Pager {
    pub fn new() -> Pager {
        let mut file = OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .open("database.db")
            .unwrap();
        let pages = vec![None; TABLE_MAX_PAGES];
        let num_pages = (file.metadata().unwrap().len() / PAGE_SIZE as u64) as usize;
        let file_length = file.metadata().unwrap().len() as usize;
        Pager {
            file,
            file_length,
            pages,
            num_pages,
        }
    }

    pub fn close(&mut self) {
        for i in 0..self.num_pages {
            self.flush(i)
        }
    }

    pub fn page_to_read(&mut self, page_index: usize) -> &Page {
        if page_index > TABLE_MAX_PAGES {
            panic!("Reached EOF");
        }
        if self.pages[page_index] == None {
            self.load(page_index);
        }
        self.pages[page_index].as_ref().unwrap()
    }

    /// Gets a mutable reference to a Page.
    pub fn page_to_write(&mut self, page_index: usize) -> &mut Page {
        if page_index > TABLE_MAX_PAGES {
            panic!("Reached EOF"); // TODO Properly handle error types.
        }

        if self.pages[page_index].is_none() {
            let page = vec![0; PAGE_SIZE];
            self.pages[page_index] = Some(page);
            self.num_pages += 1;
                
            /// If page index is LT the number of pages in 
            /// the file. Start reading from there. 
            if (page_index <= self.num_pages) {
                let offset = page_index * PAGE_SIZE;
                self.file.seek(SeekFrom::Start(offset as u64)).unwrap();
                let mut buf = vec![0; PAGE_SIZE];
                self.file.read(buf.as_mut_slice()).unwrap();
                self.pages[page_index] = Some(buf);
            }


            if (page_index >= self.num_pages) {
                self.num_pages = page_index + 1;
            }
        }

        return self.pages[page_index].as_mut().unwrap();
    }

    fn load(&mut self, page_index: usize) {
        let offset = page_index * PAGE_SIZE;
        let mut buf = vec![0; PAGE_SIZE];
        self.file.seek(SeekFrom::Start(offset as u64)).unwrap();
        self.file.read(buf.as_mut_slice()).unwrap();
        self.pages[page_index] = Some(buf);
    }

    pub fn flush(&mut self, page_index: usize) {
        let offset = page_index * PAGE_SIZE;
        if let Some(ref mut page) = self.pages[page_index] {
            self.file.seek(SeekFrom::Start(offset as u64)).unwrap();
            self.file.write_all(page.as_mut_slice()).unwrap();
        }
    }
}

