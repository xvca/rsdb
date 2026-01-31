# rsdb

a sqlite clone in rust, following https://cstack.github.io/db_tutorial/

learning project to understand database internals by building one from scratch.

## building and running

```bash
cargo build
cargo run
cargo test
```

## progress

### tutorial parts (1-14)
- [x] part 1: repl
- [x] part 2: sql compiler and vm
- [ ] part 3: in-memory table with insert/select
- [ ] part 4: testing infrastructure
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

### beyond the tutorial
- [ ] schema support (CREATE TABLE)
- [ ] multiple tables
- [ ] data types beyond fixed strings
- [ ] WHERE clauses
- [ ] JOIN operations
- [ ] secondary indexes
- [ ] transactions
