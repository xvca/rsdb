# rsdb

a sqlite clone in rust, following https://cstack.github.io/db_tutorial/

learning project to understand database internals by building one from scratch.

## building and running

```bash
cargo build
cargo run
cargo test
```

## current status

currently implements a basic in-memory database with a single hardcoded table schema:
- **table**: rows with (id: u32, username: string, email: string)
- **operations**: insert and select (no where clauses yet)
- **storage**: page-based in-memory storage (4kb pages, up to 100 pages)
- **validation**: rejects strings exceeding max lengths (32 for username, 255 for email)
- **testing**: unit tests for serialization, integration tests for end-to-end functionality
- **architecture**: clean lib/main split for reusable components

the schema is intentionally fixed to focus on learning storage engine internals. schema management (CREATE TABLE, multiple tables, dynamic types) will be added later as extensions.

## progress

### tutorial parts (1-14)
- [x] part 1: repl
- [x] part 2: sql compiler and vm
- [x] part 3: in-memory table with insert/select
- [x] part 4: testing infrastructure and input validation
- [ ] part 5: persistence to disk
- [ ] part 6: cursor abstraction
- [ ] part 7: b-tree leaf node format
- [ ] part 8: b-tree leaf node binary search
- [ ] part 9: binary search and duplicate keys
- [ ] part 10: splitting a leaf node
- [ ] part 11: recursively searching b-tree
- [ ] part 12: scanning multi-level b-tree
- [ ] part 13: updating parent node after split
- [ ] part 14: splitting internal nodes

### extension
- [ ] schema support (CREATE TABLE)
- [ ] multiple tables
- [ ] data types beyond fixed strings
- [ ] WHERE clauses
- [ ] JOIN operations
- [ ] secondary indexes
- [ ] transactions
