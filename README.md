# Mini SQLite Port

This project was made for educational purposes only, in order to demonstrate the efficiency of an hypothetical database management system written in Rust. A variety of features are still unimplemented in this prototype, additionally I do not give any guarantees that its API is stable enough. As consequence the use in production or in intensive data applications of this program is not recommended.

This prototype supports the current operations:

 1. INSERT: Insert data in a new table. If not exists, a .db file is created by default.
 2. SELECT: Prints the data from the current table, if it exist.
 3. DUMP File

# TODO
  - [x] Persistence.
  - [x] Cursor abstraction to move around the table.
 - [x] Dump file support.
 - [x] Table pretty-printing.
 - [ ] A better error handling.
 - [ ] B-Tree for indexing.
