# sqlite_dbhash

A Rust port of dbhash utility from sqlite repository so that it can be used as a library function instead of a standalone program.

Ported directly from [dbhash.c](https://github.com/sqlite/sqlite/blob/master/tool/dbhash.c).

## Usage
```rust
use sqlite_dbhash::{dbhash, Selection};
use rusqlite::{Connection, Result};
fn main() -> Result<()> {
    let conn = Connection::open("my_db.db")?;
    // Equivalent to `dbhash my_db.db` (except for the output mat)
    println!("{:02x?}", dbhash(&conn, None, Selection::SchemaAndContent)?);
    // Equivalent to `dbhash --like "prefix% --schema-only my_db.db"
    println!("{:02x?}", dbhash(&conn, Some("prefix%"), Selection::SchemaOnly)?);
    // Equivalent to `dbhash --without-schema my_db.db`
    println!("{:02x?}", dbhash(&conn, None, Selection::ContentOnly)?);
    Ok(())
}
```

## Intentional Breakage
For the vast majority of the scenarios, the hash produced by this library agrees with the `dbhash` program from sqlite. However, the hash can be different when the `table_pattern`/`--like` parameter contains non-ASCII characters.

I consider it a bug in the sqlite implementation. Since the encoding of the arguments passed from the console is platform-dependent, and the sqlite `dbhash` implementation simply used `sqlite3_vmprintf` to interpolate the SQL statement using the raw `argv`, it could be the case that the argument is not in UTF-8 (on Windows for example), and thus `dbhash` matches no table with the pattern while it is a false negative.

## License
Inheriting sqlite blessing license as follows
```
The author disclaims copyright to this source code. In place of a legal notice, here is a blessing:

    May you do good and not evil.
    May you find forgiveness for yourself and forgive others.
    May you share freely, never taking more than you give.
```