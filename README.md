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

single hardcoded table with fixed schema (id, username, email):
- insert and select operations
- b-tree storage: structured leaf nodes with header + cells (key + serialized row)
- cursor abstraction for table traversal
- page-based i/o (4kb pages), current limit: 13 rows per leaf node
- data persists to disk, survives restarts
- validates string lengths (32 for username, 255 for email)
- meta commands: .exit, .constants, .btree
- error handling via Result types
- lib/main split for testing

the schema is intentionally fixed to focus on learning storage engine internals. schema management (CREATE TABLE, multiple tables, dynamic types) will be added later as extensions.

## progress

### tutorial parts (1-14)
- [x] part 1: repl
- [x] part 2: sql compiler and vm
- [x] part 3: in-memory table with insert/select
- [x] part 4: testing infrastructure and input validation
- [x] part 5: persistence to disk
- [x] part 6: cursor abstraction
- [x] part 7: introduction to b-trees (conceptual)
- [x] part 8: b-tree leaf node format
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
