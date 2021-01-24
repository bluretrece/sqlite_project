#[allow(dead_code)]
#[macro_use]
extern crate text_io;
extern crate byteorder;
extern crate termcolor;
extern crate prettytable;
mod db;
mod cli;
mod pager;

use db::Table;

// TODO
//
// Implement better guidelines for user using Clap crate.

fn main() {
    let mut table: Table = Table::new();
    let exit_code = cli::run(&mut table);
    table.close();
    std::process::exit(exit_code);
}
