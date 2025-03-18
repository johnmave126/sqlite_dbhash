use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, ensure};
use rusqlite::Connection;
use sqlite_dbhash::{Selection, dbhash};

pub enum Step {
    /// The test harness executes the provided SQL and performs
    /// a full database comparison with the original dbhash program.
    Sql(&'static str),
    /// The test harness performs a comparison specified by
    /// the table pattern and what to compare.
    Compare {
        table_pattern: Option<&'static str>,
        selection: Selection,
    },
}

impl Step {
    /// Constructor to execute the provided `query` for a step
    pub fn sql(query: &'static str) -> Self {
        Self::Sql(query)
    }

    /// Constructor to perform custom comparison for a step
    pub fn compare(table_pattern: Option<&'static str>, selection: Selection) -> Self {
        Self::Compare {
            table_pattern,
            selection,
        }
    }
}

/// Start a test with steps from `steps`. A temporary database with `name`
/// will be created for the test.
pub fn run_tests<S>(name: &str, steps: S)
where
    S: IntoIterator<Item = Step>,
{
    try_run_tests(name, steps).expect("failed to finish test")
}

/// Start a test with error propagation.
fn try_run_tests<S>(name: &str, steps: S) -> anyhow::Result<()>
where
    S: IntoIterator<Item = Step>,
{
    let db_file = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}.db"));
    fs::remove_file(&db_file).or_else(|e| {
        if matches!(e.kind(), std::io::ErrorKind::NotFound) {
            Ok(())
        } else {
            Err(e)
        }
    })?;

    for (i, step) in steps.into_iter().enumerate() {
        eprintln!("Executing step {i}");

        match step {
            Step::Sql(sql) => {
                let conn = Connection::open(&db_file)
                    .with_context(|| format!("failed to open {}", db_file.display()))?;
                conn.execute_batch(sql).context("failed to run sql")?;
                conn.close()
                    .map_err(|(_, e)| e)
                    .context("failed to close database connection")?;
                full_compare_dbhash(&db_file)?;
            }
            Step::Compare {
                table_pattern,
                selection,
            } => {
                assert_eq!(
                    StockHasher::dbhash(&db_file, table_pattern, selection)?,
                    LibHasher::dbhash(&db_file, table_pattern, selection)?
                );
            }
        }
    }

    Ok(())
}

/// Compare the whole database hash between library and the ground truth.
fn full_compare_dbhash(db_file: &Path) -> anyhow::Result<()> {
    assert_eq!(
        StockHasher::dbhash(&db_file, None, Selection::SchemaOnly)?,
        LibHasher::dbhash(&db_file, None, Selection::SchemaOnly)?
    );
    assert_eq!(
        StockHasher::dbhash(&db_file, None, Selection::ContentOnly)?,
        LibHasher::dbhash(&db_file, None, Selection::ContentOnly)?
    );
    assert_eq!(
        StockHasher::dbhash(&db_file, None, Selection::SchemaAndContent)?,
        LibHasher::dbhash(&db_file, None, Selection::SchemaAndContent)?
    );

    Ok(())
}

/// Represent a type that can hash a database
pub trait DbHasher {
    fn dbhash(
        db_file: &Path,
        table_pattern: Option<&str>,
        selection: Selection,
    ) -> anyhow::Result<String>;
}

/// The dbhash executable from sqlite codebase
pub struct StockHasher;

impl DbHasher for StockHasher {
    fn dbhash(
        db_file: &Path,
        table_pattern: Option<&str>,
        selection: Selection,
    ) -> anyhow::Result<String> {
        let mut cmd = Command::new(env!("DBHASH_PATH"));

        if let Some(table_pattern) = table_pattern {
            cmd.args(["--like", table_pattern]);
        }

        match selection {
            Selection::SchemaAndContent => (),
            Selection::SchemaOnly => {
                cmd.arg("--schema-only");
            }
            Selection::ContentOnly => {
                cmd.arg("--without-schema");
            }
        }

        cmd.arg(db_file);

        let output = cmd
            .output()
            .with_context(|| format!("failed to run dbhash on {}", db_file.display()))?;

        ensure!(
            output.status.success(),
            "dbhash exit with status: {}",
            output.status
        );

        String::from_utf8(output.stdout)
            .context("failed to read output as a string")?
            .split_ascii_whitespace()
            .next()
            .map(ToOwned::to_owned)
            .ok_or(anyhow::anyhow!("dbhash outputs empty content"))
    }
}

/// The library implementation of dbhash
pub struct LibHasher;

impl DbHasher for LibHasher {
    fn dbhash(
        db_file: &Path,
        table_pattern: Option<&str>,
        selection: Selection,
    ) -> anyhow::Result<String> {
        let mut conn = Connection::open(db_file)
            .with_context(|| format!("failed to open {}", db_file.display()))?;
        dbhash(&mut conn, table_pattern, selection)
            .map(hex::encode)
            .map_err(Into::into)
    }
}
