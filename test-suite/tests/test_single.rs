use sqlite_dbhash::Selection;

use crate::harness::{Step, run_tests};

mod harness;

#[test]
pub fn test_single_table_normal() {
    run_tests(
        "test_single_table_normal",
        [
            Step::sql(
                "
                CREATE TABLE t
                (
                    intval INT,
                    textval TEXT,
                    realval REAL,
                    blobval BLOB,
                    numericval NUMERIC
                );
                ",
            ),
            Step::sql(
                "
                INSERT INTO t
                VALUES
                    (0, 'a', 0.0, x'00', 0.0);
                ",
            ),
            Step::sql(
                "
                INSERT INTO t
                VALUES
                    (1, 'bbb', 3.14, x'0a0b0c0d', -2.72),
                    (42, 'áÁñçéá', 1e999, x'', -1e999);
                ",
            ),
            Step::sql(
                "
                INSERT INTO t
                VALUES
                    (NULL, NULL, NULL, NULL, NULL),
                    (-100, 'NaN', 'NaN', x'', 'NaN');
                ",
            ),
            Step::compare(Some("t"), Selection::SchemaAndContent),
            Step::compare(Some("t%"), Selection::SchemaAndContent),
            Step::compare(Some("%t"), Selection::SchemaAndContent),
        ],
    );
}

#[test]
pub fn test_single_table_weird_name() {
    run_tests(
        "test_single_table_weird_name",
        [
            Step::sql(
                r#"
                CREATE TABLE "weird.table+name""!áÁñçéá"
                (
                    intval INT,
                    textval TEXT,
                    realval REAL,
                    blobval BLOB,
                    numericval NUMERIC
                );
                "#,
            ),
            Step::sql(
                r#"
                INSERT INTO "weird.table+name""!áÁñçéá"
                VALUES
                    (0, 'a', 0.0, x'00', 0.0);
                "#,
            ),
            Step::compare(Some("weird%"), Selection::SchemaAndContent),
            Step::compare(Some("%table%"), Selection::SchemaAndContent),
            Step::compare(Some(r#"%"%"#), Selection::SchemaAndContent),
        ],
    );
}

#[test]
pub fn test_single_table_with_indices() {
    run_tests(
        "test_single_table_with_indices",
        [
            Step::sql(
                "
                CREATE TABLE t
                (
                    intval INT NOT NULL,
                    textval TEXT NOT NULL
                );
                ",
            ),
            Step::sql("CREATE UNIQUE INDEX idx_t ON t (intval);"),
            Step::sql(
                "
                INSERT INTO t
                VALUES
                    (0, 'a');
                ",
            ),
            Step::sql(
                "
                INSERT INTO t
                VALUES
                    (1, 'bbb'),
                    (42, 'áÁñçéá');
                ",
            ),
            Step::sql("CREATE INDEX text_t ON t (textval, intval);"),
        ],
    );
}
