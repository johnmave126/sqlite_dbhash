//! A Rust port of dbhash utility from sqlite repository so that
//! it can be used as a library function instead of a standalone
//! program.
//!
//! See a full exmaple in [`dbhash`].
use std::cell::OnceCell;

use rusqlite::{Connection, Rows, types::ValueRef};
use sha1::{Digest, Sha1};
#[cfg(feature = "tracing")]
use tracing::{Level, span, trace};

/// Specify what to hash, imitating the function of
/// command-line arguments `--schema-only` and `without-schema`
/// in the original dbhash utility program.
#[derive(Clone, Copy)]
pub enum Selection {
    /// Hash both schema and table content
    SchemaAndContent,
    /// Only hash the schema, equivalent to `--schema-only`
    SchemaOnly,
    /// Only hash the table content, equivalent to `--without-schema`
    ContentOnly,
}

/// Compute the SHA1 hash of database through a database connection `conn`
/// obtained from `rusqlite`.
///
/// If `table_pattern` is `None`, the whole database will be hashed. Otherwise,
/// only the tables whose name is LIKE `table_pattern` will be hashed. This is
/// equivalent to `--like PATTERN` argument in the original dbhash utility program.
///
/// Whether to hash schema or content is determined by `selection`.
/// # Examples
/// ```no_run
/// # use sqlite_dbhash::{dbhash, Selection};
/// # use rusqlite::{Connection, Result};
/// fn main() -> Result<()> {
///     let conn = Connection::open("my_db.db")?;
///     // Equivalent to `dbhash my_db.db` (except for the output format)
///     println!("{:02x?}", dbhash(&conn, None, Selection::SchemaAndContent)?);
///     // Equivalent to `dbhash --like "prefix% --schema-only my_db.db"`
///     println!("{:02x?}", dbhash(&conn, Some("prefix%"), Selection::SchemaOnly)?);
///     // Equivalent to `dbhash --without-schema my_db.db`
///     println!("{:02x?}", dbhash(&conn, None, Selection::ContentOnly)?);
///     Ok(())
/// }
/// ```
pub fn dbhash(
    conn: &Connection,
    table_pattern: Option<&str>,
    selection: Selection,
) -> rusqlite::Result<[u8; 20]> {
    #[cfg(feature = "tracing")]
    let _span = span!(Level::TRACE, "dbhash").entered();

    let mut hasher = Sha1::new();
    if matches!(
        selection,
        Selection::SchemaAndContent | Selection::ContentOnly
    ) {
        hash_content(&mut hasher, conn, table_pattern)?;
    }

    if matches!(
        selection,
        Selection::SchemaAndContent | Selection::SchemaOnly
    ) {
        hash_schema(&mut hasher, conn, table_pattern)?;
    }

    Ok(hasher.finalize().into())
}

/// Hash the content of tables specified by `table_pattern`.
fn hash_content(
    hasher: &mut Sha1,
    conn: &Connection,
    table_pattern: Option<&str>,
) -> rusqlite::Result<()> {
    // Find all tables matching the `table_pattern`.
    let mut table_names_stmt;
    let mut table_names = match table_pattern {
        Some(pattern) => {
            table_names_stmt = conn.prepare(
                "SELECT name FROM sqlite_schema
                  WHERE type = 'table'
                    AND sql NOT LIKE 'CREATE VIRTUAL%%'
                    AND name NOT LIKE 'sqlite_%%'
                    AND name LIKE ?1
                  ORDER BY name COLLATE nocase",
            )?;
            table_names_stmt.query([pattern])?
        }
        None => {
            table_names_stmt = conn.prepare(
                "SELECT name FROM sqlite_schema
                  WHERE type = 'table'
                    AND sql NOT LIKE 'CREATE VIRTUAL%%'
                    AND name NOT LIKE 'sqlite_%%'
                  ORDER BY name COLLATE nocase",
            )?;
            table_names_stmt.query([])?
        }
    };

    while let Some(row) = table_names.next()? {
        let name = row.get_ref(0)?.as_str()?;

        // optional tracing
        #[cfg(feature = "tracing")]
        let _span = span!(Level::TRACE, "hash table content", table = name).entered();

        // Escape each double-quote into two double-quotes
        let quoted_name = name.replace('"', r#""""#);

        let mut select_all_stmt = conn.prepare(&format!(r#"SELECT * FROM "{quoted_name}""#))?;
        hash_query(hasher, select_all_stmt.query([])?)?;
    }

    Ok(())
}

/// Hash the schema of tables specified by `table_pattern`.
fn hash_schema(
    hasher: &mut Sha1,
    conn: &Connection,
    table_pattern: Option<&str>,
) -> rusqlite::Result<()> {
    #[cfg(feature = "tracing")]
    let _span = span!(Level::TRACE, "hash schema").entered();

    let mut table_info_stmt;
    let table_infos = match table_pattern {
        Some(pattern) => {
            table_info_stmt = conn.prepare(
                "SELECT type, name, tbl_name, sql FROM sqlite_schema
                  WHERE tbl_name LIKE ?1
                  ORDER BY name COLLATE nocase",
            )?;
            table_info_stmt.query([pattern])?
        }
        None => {
            table_info_stmt = conn.prepare(
                "SELECT type, name, tbl_name, sql FROM sqlite_schema
                  ORDER BY name COLLATE nocase",
            )?;
            table_info_stmt.query([])?
        }
    };

    hash_query(hasher, table_infos)
}

/// Hash the result of one query
fn hash_query(hasher: &mut Sha1, mut rows: Rows<'_>) -> rusqlite::Result<()> {
    let column_count_cell = OnceCell::new();

    while let Some(row) = rows.next()? {
        // Need to lazily get column count here after stepping at least once
        // to handle shcema change between creation of statement and the execution
        // of the statement
        let column_count = column_count_cell.get_or_init(|| row.as_ref().column_count());
        for i in 0..*column_count {
            let val = row.get_ref(i)?;
            match val {
                ValueRef::Null => {
                    hasher.update(b"0");
                    #[cfg(feature = "tracing")]
                    trace!("NULL");
                }
                ValueRef::Integer(value) => {
                    let bytes = value.to_be_bytes();
                    hasher.update(b"1");
                    hasher.update(bytes);
                    #[cfg(feature = "tracing")]
                    trace!("INT {value}");
                }
                ValueRef::Real(value) => {
                    let bytes = value.to_be_bytes();
                    hasher.update(b"2");
                    hasher.update(bytes);
                    #[cfg(feature = "tracing")]
                    trace!("FLOAT {value}");
                }
                ValueRef::Text(value) => {
                    hasher.update(b"3");
                    hasher.update(value);
                    #[cfg(feature = "tracing")]
                    trace!("TEXT {text}", text = String::from_utf8_lossy(value));
                }
                ValueRef::Blob(value) => {
                    hasher.update(b"4");
                    hasher.update(value);
                    #[cfg(feature = "tracing")]
                    trace!("BLOB ({len} bytes)", len = value.len());
                }
            }
        }
    }

    Ok(())
}
